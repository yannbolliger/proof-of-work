use repyh_proof_of_work::{broadcast, Message, Transaction, Transactions};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let nodes: Vec<SocketAddr> = std::env::args().filter_map(|s| s.parse().ok()).collect();

    broadcast(
        nodes.iter(),
        Message::Tx(Transactions(Transaction::dummy_txs(10))),
    )
    .await;
    println!("Done proposing transactions to {:?}", nodes);
}
