use crate::hash::{has_leading_zeros, Hash, Hashable, HASH_LENGTH};
use crate::tx::{Transactions, GENESIS_TXS, GENESIS_TXS_HASH};
use serde::{Deserialize, Serialize};
use std::u32;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct BlockHeader {
    pub prev_block_hash: Hash,
    merkle_hash: Hash,
    difficulty: u32,
    pub nonce: u32,
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

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Block<'t> {
    pub header: BlockHeader,
    transactions: Transactions<'t>,
}

pub const GENESIS_NONCE: u32 = 442;
pub const GENESIS_BLOCK: Block = Block {
    header: BlockHeader {
        prev_block_hash: [0; HASH_LENGTH],
        difficulty: 1,
        merkle_hash: GENESIS_TXS_HASH,
        nonce: GENESIS_NONCE,
    },
    transactions: GENESIS_TXS,
};

impl<'t> Block<'t> {
    pub fn new(prev_block_hash: Hash, difficulty: u32, transactions: Transactions<'t>) -> Self {
        Block {
            header: BlockHeader::new(prev_block_hash, transactions.hash(), difficulty),
            transactions,
        }
    }

    pub fn mine_new(
        prev_block_hash: Hash,
        difficulty: u32,
        transactions: Transactions<'t>,
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
    use crate::block::{Block, BlockHeader, GENESIS_BLOCK, GENESIS_NONCE};
    use crate::hash::{Hash, Hashable, HASH_LENGTH};
    use crate::tx::{Transaction, Transactions};

    const PREVIOUS_HASH: Hash = [7; HASH_LENGTH];

    #[test]
    fn mined_block_header_valid() {
        assert!(!BlockHeader::new(PREVIOUS_HASH, [5; HASH_LENGTH], 2).is_valid());
        assert!(BlockHeader::mine_new(PREVIOUS_HASH, [5; HASH_LENGTH], 2).is_valid());
    }

    #[test]
    fn mined_block_valid() {
        let txs = Transaction::dummy_txs(10);
        assert!(!Block::new(PREVIOUS_HASH, 2, Transactions(&txs)).is_valid());
        assert!(Block::mine_new(PREVIOUS_HASH, 2, Transactions(&txs)).is_valid());
    }

    #[test]
    fn genesis_block_is_valid() {
        assert!(GENESIS_BLOCK.is_valid());
        assert_eq!(
            GENESIS_BLOCK.transactions.hash(),
            GENESIS_BLOCK.header.merkle_hash
        );
        assert_eq!(GENESIS_BLOCK.header.solve(), GENESIS_NONCE);
    }
}
