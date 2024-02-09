pub use crate::block::Block;
pub use crate::chain::BlockChain;
pub use crate::hash::{Hash, Hashable};
pub use crate::tx::{Transaction, Transactions};

use std::net::IpAddr;

mod block;
mod chain;
mod hash;
mod tx;

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
