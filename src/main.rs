use crate::block::Block;
use crate::hash::{B58Encode, Hashable, HASH_LENGTH};
use crate::tx::GENESIS_TXS;

mod block;
mod hash;
mod tx;

fn main() {
    let block = Block::mine_new([0; HASH_LENGTH], 1, GENESIS_TXS);
    println!("Genesis txs merkle hash: {:?}", GENESIS_TXS.hash());
    println!(
        "{:?} matches hash: {:?}",
        block.header.nonce,
        block.hash().encode()
    )
}
