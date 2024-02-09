use repyh_proof_of_work::*;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task;
use tokio::task::JoinHandle;

pub struct Node {
    address: SocketAddr,
    peers: HashSet<SocketAddr>,
    mempool: HashMap<Hash, Transaction>,
    chain: BlockChain,
    txs_to_mine: Transactions,
    mining_task: Option<JoinHandle<()>>,
}

impl Node {
    pub fn new(address: SocketAddr, peers: &[SocketAddr]) -> Self {
        Node {
            mempool: HashMap::new(),
            chain: BlockChain::new(),
            peers: peers.iter().cloned().collect(),
            txs_to_mine: Transactions(vec![]),
            mining_task: None,
            address,
        }
    }

    pub fn handle(&mut self, message: Message) -> Option<Message> {
        match message {
            // if a new peer connects
            Message::Connect(addr) if addr == self.address => None,
            // ... and is not ourselves, add it to the peers and broadcast some known peers
            Message::Connect(addr) => self.peers.insert(addr).then(|| {
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

            // add broadcast peer addresses to addresses (except ourselves)
            Message::Addr(addrs) => {
                self.peers
                    .extend(addrs.into_iter().filter(|a| a != &self.address));
                None
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
                (!new_txs.is_empty()).then(|| {
                    Message::Tx(Transactions(new_txs.into_iter().map(|(_, t)| t).collect()))
                })
            }

            // adds a new block to chain, if valid and rebroadcasts if valid & new
            Message::NewBlock(block) => {
                let is_new = self.chain.add_block(&block);
                if is_new && self.chain.highest_block() == &block {
                    // stop previous mining task
                    if let Some(task) = &self.mining_task {
                        task.abort();
                        self.mempool
                            .extend(self.txs_to_mine.0.drain(..).map(|t| (t.hash(), t)));
                    }

                    // start mining
                    let prev_hash = self.chain.highest_block().hash();
                    self.txs_to_mine = Transactions(self.mempool.drain().map(|(_, t)| t).collect());
                    let txs = self.txs_to_mine.clone();

                    self.mining_task = Some(task::spawn_blocking(move || {
                        let mined_block = Block::mine_new(prev_hash, GLOBAL_DIFFICULTY, txs);
                        // TODO: send mined block to network and ourselves
                    }));
                }
                is_new.then_some(Message::NewBlock(block))
            }
        }
    }

    pub async fn broadcast(&self, message: Message) {
        broadcast(self.peers.iter(), message).await
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

    let mut node_state = Node::new(address, &initial_peers);

    // Announce ourselves to network
    node_state.broadcast(Message::Connect(address)).await;

    println!("Starting to process...");
    while let Ok((mut socket, _)) = listener.accept().await {
        let mut buf = [0; 1024];
        let n = socket.read(&mut buf).await.unwrap();
        let message: Message = bincode::deserialize(&buf[0..n]).unwrap();
        println!("Got {:?}", message);
        let reply = node_state.handle(message);
        if let Some(r) = reply {
            println!("Send {:?}", &r);
            node_state.broadcast(r).await;
        }
    }
}
