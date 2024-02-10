use crate::MiningCommand::{Keep, Restart, Start};
use repyh_proof_of_work::*;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::task;
use tokio::task::JoinHandle;

pub struct Node {
    address: SocketAddr,
    peers: HashSet<SocketAddr>,
    mempool: HashMap<Hash, Transaction>,
    chain: BlockChain,
}

enum MiningCommand {
    Start,
    Restart,
    Keep,
}

impl Node {
    pub fn new(address: SocketAddr, peers: &[SocketAddr]) -> Self {
        Node {
            mempool: HashMap::new(),
            chain: BlockChain::new(),
            peers: peers.iter().cloned().collect(),
            address,
        }
    }

    /// Handles the state transitions of the node in response to the given message.
    /// Returns an optional reply to broadcast back to the network and instructions whether
    /// a mining process should be (re-)started.
    fn handle(&mut self, message: Message) -> (Option<Message>, MiningCommand) {
        match message {
            // if a new peer connects
            Message::Connect(addr) if addr == self.address => (None, Keep),
            // ... and is not ourselves, add it to the peers and broadcast some known peers
            Message::Connect(addr) => (
                self.peers.insert(addr).then(|| {
                    Message::Addr(
                        self.peers
                            .iter()
                            .filter(|&a| a != &addr)
                            .take(9)
                            .chain(&[self.address])
                            .cloned()
                            .collect(),
                    )
                }),
                Keep,
            ),

            // add broadcast peer addresses to addresses (except ourselves)
            Message::Addr(addrs) => {
                self.peers
                    .extend(addrs.into_iter().filter(|a| a != &self.address));
                (None, Keep)
            }

            // add broadcast txs to mempool and rebroadcast new ones
            Message::Tx(txs) => {
                let new_txs = txs
                    .0
                    .into_iter()
                    .map(|t| (t.hash(), t))
                    .filter(|(h, _)| !self.mempool.contains_key(h))
                    .collect::<Vec<_>>();
                self.mempool.extend(new_txs.clone());
                (
                    (!new_txs.is_empty()).then(|| {
                        Message::Tx(Transactions(new_txs.into_iter().map(|(_, t)| t).collect()))
                    }),
                    Start,
                )
            }

            // adds a new block to chain, if valid and rebroadcasts if valid & new
            Message::NewBlock(block) => {
                let is_new = self.chain.add_block(&block);
                let restart = if self.chain.highest_block() == &block {
                    Restart
                } else {
                    Start
                };
                (is_new.then_some(Message::NewBlock(block)), restart)
            }
        }
    }

    pub async fn broadcast(&self, message: Message) {
        broadcast(self.peers.iter(), message).await
    }
}

pub async fn start_mining(node_state: Arc<RwLock<Node>>) {
    let (prev_hash, txs) = {
        let node = node_state.read().await;
        let prev_hash = node.chain.highest_block().hash();
        let txs = node
            .mempool
            .iter()
            .take(MAX_TXS)
            .map(|(_, t)| t.clone())
            .collect();
        (prev_hash, txs)
    };

    let task = task::spawn_blocking(move || {
        println!("start mining");
        Block::mine_new(prev_hash, GLOBAL_DIFFICULTY, Transactions(txs))
    });

    if let Ok(mined_block) = task.await {
        println!("Mined {:?}", mined_block.header);
        let mut node = node_state.write().await;
        node.chain.add_block(&mined_block);
        let mined_txs: HashSet<Hash> = mined_block
            .transactions
            .0
            .iter()
            .map(|t| t.hash())
            .collect();
        // Only keep txs that weren't included in this block
        node.mempool.retain(|h, _| !mined_txs.contains(h));
        node.broadcast(Message::NewBlock(mined_block)).await
    }
}

#[tokio::main]
async fn main() {
    let initial_peers: Vec<SocketAddr> = std::env::args().filter_map(|s| s.parse().ok()).collect();

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    println!(
        "Node started at {} with initial peers: {:?}",
        address, initial_peers
    );

    let node_state = Arc::new(RwLock::new(Node::new(address, &initial_peers)));
    let mut mining_task: Option<JoinHandle<()>> = None;

    // Announce ourselves to network
    {
        let node = node_state.read().await;
        node.broadcast(Message::Connect(address)).await;
    }

    println!("Starting to process...");
    while let Ok((mut socket, _)) = listener.accept().await {
        let mut buf = [0; 1024];
        let n = socket.read(&mut buf).await.unwrap();
        let message: Message = bincode::deserialize(&buf[0..n]).unwrap();
        println!("Got {:?}", message);

        let (reply, mining_command) = {
            let mut node = node_state.write().await;
            node.handle(message)
        };

        match mining_command {
            Restart => {
                if let Some(task) = mining_task {
                    task.abort()
                }
                mining_task = Some(task::spawn(start_mining(node_state.clone())));
            }
            Start if mining_task.is_none() => {
                mining_task = Some(task::spawn(start_mining(node_state.clone())));
            }
            _ => {}
        }

        // Send replies to the network if needed
        if let Some(r) = reply {
            println!("Send {:?}", &r);
            let node = node_state.read().await;
            node.broadcast(r).await;
        }
    }
}
