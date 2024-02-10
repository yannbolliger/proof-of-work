use crate::MiningCommand::{Keep, Restart, Start};
use repyh_proof_of_work::*;
use std::collections::{HashMap, HashSet};
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio::task;
use tokio::task::JoinHandle;

/// A full node running on the blockchain.
pub struct Node {
    /// The node's own address
    address: SocketAddr,
    /// The known network peers
    peers: HashSet<SocketAddr>,
    /// Transactions proposed for inclusion in a block
    mempool: HashMap<Hash, Transaction>,
    /// The local copy of the blockchain
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
                // filter only the unknown transactions
                // TODO: this currently only looks at the mempool. However, it should also check
                //  whether a transaction was already added/comitted to the chain.
                let new_txs = txs
                    .0
                    .into_iter()
                    .map(|t| (t.hash(), t))
                    .filter(|(h, _)| !self.mempool.contains_key(h))
                    .collect::<Vec<_>>();

                // TODO: once transactions get more integrity (like signing/ordering etc. see also
                //  [Transaction], we need to validate them here before adding them to the mempool
                self.mempool.extend(new_txs.clone());

                // rebroadcast transactions we didn't yet know about
                (
                    (!new_txs.is_empty()).then(|| {
                        Message::Tx(Transactions(new_txs.into_iter().map(|(_, t)| t).collect()))
                    }),
                    Start, // start mining if not already in process
                )
            }

            // adds a new block to chain, if valid and rebroadcasts if valid & new
            Message::NewBlock(block) => {
                let is_new = self.add_block(&block);
                // if the main chain has updated, we need to restart the mining with the
                // new highest block as parent
                let cmd = if self.chain.highest_block() == &block {
                    Restart
                } else {
                    Start // otherwise, we still start mining if we were done
                };
                (is_new.then_some(Message::NewBlock(block)), cmd)
            }
        }
    }

    /// Adds a block to the chain and if valid, removes the transactions
    /// that it includes from the mempool.
    fn add_block(&mut self, block: &Block) -> bool {
        let is_new = self.chain.add_block(block);
        if is_new {
            let txs: HashSet<Hash> = block.transactions.0.iter().map(|t| t.hash()).collect();
            self.mempool.retain(|h, _| !txs.contains(h));
        }
        is_new
    }
}

/// Start a new mining process.
async fn start_mining(node_state: Arc<RwLock<Node>>) -> io::Result<()> {
    let (prev_hash, txs) = {
        let node = node_state.read().await;
        if node.mempool.is_empty() {
            println!("No txs to mine.");
            return Ok(());
        };
        let prev_hash = node.chain.highest_block().hash();
        // Take "some" transactions from the pool
        let txs = node
            .mempool
            .iter()
            .take(MAX_TXS)
            .map(|(_, t)| t.clone())
            .collect::<Vec<_>>();
        (prev_hash, txs)
    };

    // Start the mining process (blocking because CPU-bound)
    // Note that no lock is kept during the mining.
    let mined_block = task::spawn_blocking(move || {
        Block::mine_new(prev_hash, GLOBAL_DIFFICULTY, Transactions(txs))
    })
    .await?;
    println!("Mined {:?}", mined_block.header);
    let valid = {
        let mut node = node_state.write().await;
        node.add_block(&mined_block)
    };
    if valid {
        broadcast(node_state, &Message::NewBlock(mined_block)).await
    } else {
        Ok(())
    }
}

async fn broadcast(node_state: Arc<RwLock<Node>>, message: &Message) -> io::Result<()> {
    println!("Send {:?}", &message);
    let node = node_state.read().await;
    message.broadcast(node.peers.iter()).await
}

// TODO: there is no limit (in the protocol) on the message size. The receiving will break for
//  too long messages. Implement either a limit or continuous reading on the socket to fix this.
const RCV_BUFFER_LEN: usize = 1024;

async fn accept_message(listener: &TcpListener) -> io::Result<Message> {
    let (mut socket, _) = listener.accept().await?;
    let mut buf = [0; RCV_BUFFER_LEN];
    let n = socket.read(&mut buf).await?;
    Message::try_from(&buf[..n])
}

/// By default, a node will try to bind itself to `localhost:7000`.
const DEFAULT_SOCKET: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 7000);

#[tokio::main]
async fn main() -> io::Result<()> {
    // read initial peer address from the CLI arguments
    let initial_peers: Vec<SocketAddr> = std::env::args().filter_map(|s| s.parse().ok()).collect();

    // Try to bind to default port or take random port if already in use
    let listener = match TcpListener::bind(DEFAULT_SOCKET).await {
        Ok(l) => l,
        Err(_) => TcpListener::bind("127.0.0.1:0").await?,
    };
    let address = listener.local_addr()?;
    println!(
        "Node started at {} with initial peers: {:?}",
        address, initial_peers
    );

    // The entire state of the node
    let node_state = Arc::new(RwLock::new(Node::new(address, &initial_peers)));
    let mut mining_task: Option<JoinHandle<_>> = None;

    // Announce ourselves to network
    broadcast(node_state.clone(), &Message::Connect(address)).await?;

    println!("Starting to process...");
    // the event loop
    // TODO: the event loop is currently synchronous but spawns off the mining.
    //  The next iteration should be to spawn off a task for each incoming message.
    while let Ok(message) = accept_message(&listener).await {
        println!("Got {:?}", message);

        let (reply, mining_command) = {
            let mut node = node_state.write().await;
            node.handle(message)
        };

        match mining_command {
            Restart => {
                println!("Restart mining");
                if let Some(task) = mining_task {
                    task.abort()
                }
                mining_task = Some(task::spawn(start_mining(node_state.clone())));
            }
            Start
                if mining_task.is_none()
                    || mining_task.as_ref().is_some_and(|t| t.is_finished()) =>
            {
                println!("Start mining");
                mining_task = Some(task::spawn(start_mining(node_state.clone())));
            }
            _ => {}
        }

        // Send replies to the network if needed
        if let Some(r) = reply {
            broadcast(node_state.clone(), &r).await?;
        }
    }
    Ok(())
}
