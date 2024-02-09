use crate::block::BlockHeader;
use crate::hash::{B58Encode, Hashable, HASH_LENGTH};

mod block;
mod hash;

fn main() {
    let header = BlockHeader::mine_new([0; HASH_LENGTH], [1; HASH_LENGTH], 2);
    println!("{:?} matches hash: {:?}", header, header.hash().encode())
}
