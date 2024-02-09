use crate::hash::{has_leading_zeros, Hash, Hashable};
use crate::tx::Transactions;
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

    pub fn is_valid(&self) -> bool {
        has_leading_zeros(&self.hash(), self.difficulty as usize)
    }
}

impl Hashable for BlockHeader {
    fn hash(&self) -> Hash {
        Self::hash_bytes(&bincode::serialize(self).expect("Block should be serializable"))
    }
}

pub struct Block<'t> {
    header: BlockHeader,
    transactions: &'t Transactions,
}

impl<'t> Block<'t> {
    pub fn new(prev_block_hash: Hash, difficulty: u32, transactions: &'t Transactions) -> Self {
        Block {
            header: BlockHeader::new(prev_block_hash, transactions.hash(), difficulty),
            transactions,
        }
    }

    pub fn mine_new(
        prev_block_hash: Hash,
        difficulty: u32,
        transactions: &'t Transactions,
    ) -> Self {
        Block {
            header: BlockHeader::mine_new(prev_block_hash, transactions.hash(), difficulty),
            transactions,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.transactions.hash() == self.header.merkle_hash && self.header.is_valid()
    }
}

impl<'t> Hashable for Block<'t> {
    /// A block's hash is only its headers hash.
    fn hash(&self) -> Hash {
        self.header.hash()
    }
}

#[cfg(test)]
mod test {
    use crate::block::{Block, BlockHeader};
    use crate::hash::{Hash, HASH_LENGTH};
    use crate::tx::Transactions;

    const PREVIOUS_HASH: Hash = [7; HASH_LENGTH];

    #[test]
    fn mined_block_header_valid() {
        assert!(!BlockHeader::new(PREVIOUS_HASH, [5; HASH_LENGTH], 2).is_valid());
        assert!(BlockHeader::mine_new(PREVIOUS_HASH, [5; HASH_LENGTH], 2).is_valid());
    }

    #[test]
    fn mined_block_valid() {
        let txs = Transactions::dummy_txs(10);
        assert!(!Block::new(PREVIOUS_HASH, 2, &txs).is_valid());
        assert!(Block::mine_new(PREVIOUS_HASH, 2, &txs).is_valid());
    }
}
