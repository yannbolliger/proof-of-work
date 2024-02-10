use repyh_proof_of_work::{broadcast, Message, Transaction, Transactions};
use std::net::SocketAddr;
use tokio::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let nodes: Vec<SocketAddr> = std::env::args().filter_map(|s| s.parse().ok()).collect();

    broadcast(
        nodes.iter(),
        &Message::Tx(Transactions(Transaction::dummy_txs(10))),
    )
    .await?;
    println!("Done proposing transactions to {:?}", nodes);
    Ok(())
}
