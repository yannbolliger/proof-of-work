pub use crate::block::{Block, MAX_TXS};
pub use crate::chain::BlockChain;
pub use crate::hash::{Hash, Hashable};
pub use crate::msg::Message;
pub use crate::tx::{Transaction, Transactions};

mod block;
mod chain;
mod hash;
mod msg;
mod tx;

// TODO: Implement a difficulty based on the block height and take it into account when verifying
//   new blocks.
pub const GLOBAL_DIFFICULTY: u32 = 2;
