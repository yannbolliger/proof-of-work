use crate::block::Block;
use crate::chain::BlockChain;
use crate::hash::{Hash, Hashable};
use crate::tx::{Transaction, Transactions};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::thread;

// TODO: Implement a difficulty based on the block height and take it into account when verifying
//   new blocks.
pub const GLOBAL_DIFFICULTY: u32 = 2;

pub enum Message {
    /// A new node joins the network and announces its address
    Connect(IpAddr),

    /// Announces known/live node addresses (excluding the node's own address) to the network
    Addr(Vec<IpAddr>),

    /// Proposes transactions for inclusion into blocks
    Tx(Transactions),

    /// Announces the mining of a new block
    NewBlock(Block),
    // TODO: for now a node needs to have been around from the first mining to participate
    //   add messages for synchronising past blocks
}

pub struct Node {
    address: IpAddr,
    peers: HashSet<IpAddr>,
    mempool: HashMap<Hash, Transaction>,
    chain: BlockChain,
}

impl Node {
    pub fn new(address: IpAddr) -> Self {
        Node {
            mempool: HashMap::new(),
            chain: BlockChain::new(),
            peers: HashSet::new(),
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
                    // TODO: restart mining with the new `self.chain.highest_block()` as parent
                    // stop previous mining process, return txs to mempool

                    // start mining
                    let prev_hash = self.chain.highest_block().hash();
                    let txs = Transactions(self.mempool.drain().map(|(_, t)| t).collect());
                    thread::spawn(move || Block::mine_new(prev_hash, GLOBAL_DIFFICULTY, txs));
                }
                is_new.then_some(Message::NewBlock(block))
            }
        }
    }
}
