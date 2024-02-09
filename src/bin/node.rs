use repyh_proof_of_work::*;
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use tokio::task;
use tokio::task::JoinHandle;

pub struct Node {
    address: IpAddr,
    peers: HashSet<IpAddr>,
    mempool: HashMap<Hash, Transaction>,
    chain: BlockChain,
    txs_to_mine: Transactions,
    mining_task: Option<JoinHandle<()>>,
}

impl Node {
    pub fn new(address: IpAddr) -> Self {
        Node {
            mempool: HashMap::new(),
            chain: BlockChain::new(),
            peers: HashSet::new(),
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
            Message::Connect(addr) => self
                .peers
                .insert(addr)
                .then(|| Message::Addr(self.peers.iter().take(10).cloned().collect())),

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
}

#[tokio::main]
async fn main() {}
