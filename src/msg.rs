use crate::{Block, Transactions};
use serde::{Deserialize, Serialize};
use std::io;
use std::io::ErrorKind;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

/// The blockchain protocol™️
///
/// I.e. all the possible messages that a full node accepts and sends.
// TODO: for now, a node needs to have been around from the first mining to participate
//  because there is no way of synchronising past blocks (and the blockchain rejects
//  orphans). Implement query/sync message to get old blocks and put them into the chain.
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    /// A new node joins the network and announces its address.
    /// Recipients maybe respond with [Message::Addr].
    Connect(SocketAddr),

    /// Announces known/live node addresses to the network
    Addr(Vec<SocketAddr>),

    /// Proposes transactions for inclusion into blocks
    Tx(Transactions),

    /// Announces the mining of a new block
    NewBlock(Block),
}

impl Message {
    /// Send this message over TCP to all the given addresses.
    pub async fn broadcast<'a, I: Iterator<Item = &'a SocketAddr>>(
        &self,
        addrs: I,
    ) -> io::Result<()> {
        let bytes: Vec<u8> = self.into();
        for peer in addrs {
            let mut stream = TcpStream::connect(peer).await?;
            stream.write_all(&bytes).await?;
        }
        Ok(())
    }
}

impl TryFrom<&[u8]> for Message {
    type Error = io::Error;

    fn try_from(value: &[u8]) -> io::Result<Self> {
        bincode::deserialize(value).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
    }
}

impl From<&Message> for Vec<u8> {
    fn from(value: &Message) -> Self {
        bincode::serialize(&value).expect("can always serialize a message")
    }
}
