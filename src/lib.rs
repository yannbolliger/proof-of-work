pub use crate::block::{Block, MAX_TXS};
pub use crate::chain::BlockChain;
pub use crate::hash::{Hash, Hashable};
pub use crate::tx::{Transaction, Transactions};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

mod block;
mod chain;
mod hash;
mod tx;

// TODO: Implement a difficulty based on the block height and take it into account when verifying
//   new blocks.
pub const GLOBAL_DIFFICULTY: u32 = 2;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    /// A new node joins the network and announces its address
    Connect(SocketAddr),

    /// Announces known/live node addresses (excluding the node's own address) to the network
    Addr(Vec<SocketAddr>),

    /// Proposes transactions for inclusion into blocks
    Tx(Transactions),

    /// Announces the mining of a new block
    NewBlock(Block),
    // TODO: for now a node needs to have been around from the first mining to participate
    //   add messages for synchronising past blocks
}

pub async fn broadcast<'a, I: Iterator<Item = &'a SocketAddr>>(addrs: I, message: Message) {
    let bytes = bincode::serialize(&message).unwrap();
    for peer in addrs {
        let mut stream = TcpStream::connect(peer).await.unwrap();
        stream.write_all(&bytes).await.unwrap();
    }
}
