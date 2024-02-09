use crate::hash::{has_leading_zeros, Hash, Hashable};
use crate::tx::Transaction;
use serde::{Deserialize, Serialize};
use std::u32;

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockHeader {
    prev_block_hash: Hash,
    merkle_hash: Hash,
    difficulty: u32,
    nonce: u32,
}
impl BlockHeader {
    /// Creates a new block header with 0 nonce.
    /// This block header is only valid after solving it.
    pub fn new(prev_block_hash: Hash, merkle_hash: Hash, difficulty: u32) -> Self {
        BlockHeader {
            prev_block_hash,
            merkle_hash,
            difficulty,
            nonce: 0,
        }
    }

    /// Creates a new block and mines/solves it such that the hash
    /// satisfies the given difficulty.
    pub fn mine_new(prev_block_hash: Hash, merkle_hash: Hash, difficulty: u32) -> Self {
        let initial = Self::new(prev_block_hash, merkle_hash, difficulty);
        Self {
            nonce: initial.solve(),
            ..initial
        }
    }

    /// Mines the nonce needed to solve this block.
    fn solve(&self) -> u32 {
        (0..u32::MAX)
            .find(|n| {
                let new_header = Self { nonce: *n, ..*self };
                let hash = new_header.hash();
                has_leading_zeros(&hash, self.difficulty as usize)
            })
            .unwrap()
    }
}

impl Hashable for BlockHeader {
    fn hash(&self) -> Hash {
        Self::hash_bytes(&bincode::serialize(self).expect("Block should be serializable"))
    }
}
