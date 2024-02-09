use crate::block::Block;
use crate::hash::{B58Encode, Hashable, HASH_LENGTH};
use crate::tx::Transactions;

mod block;
mod chain;
mod hash;
mod tx;

fn main() {
    let block = Block::mine_new([0; HASH_LENGTH], 1, Transactions::genesis());
    println!(
        "{:?} matches hash: {:?}",
        block.header.nonce,
        block.hash().encode()
    )
}
