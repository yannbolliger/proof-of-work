use crate::hash::{has_leading_zeros, B58Encode, Hash, Hashable, HASH_LENGTH};
use crate::tx::{Transactions, GENESIS_TXS_HASH};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::u32;

/// Fully identifies a block on the chain.
/// A block is valid iff hashing all its bytes results in a hash with at least `difficulty` leading
/// zero bytes.
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct BlockHeader {
    pub prev_block_hash: Hash,
    merkle_hash: Hash,
    difficulty: u32,
    pub nonce: u32,
}

impl Debug for BlockHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BlockHeader {{ prev_block_hash: {}, merkle_hash: {}, difficulty: {}, nonce: {} }}",
            self.prev_block_hash.encode(),
            self.merkle_hash.encode(),
            self.difficulty,
            self.nonce
        )
    }
}

/// The nonce making [GENESIS_HEADER] valid.
pub const GENESIS_NONCE: u32 = 437;

/// The hard-coded first block (header) on this chain.
pub const GENESIS_HEADER: BlockHeader = BlockHeader {
    prev_block_hash: [0; HASH_LENGTH],
    difficulty: 1,
    merkle_hash: GENESIS_TXS_HASH,
    nonce: GENESIS_NONCE,
};

impl BlockHeader {
    /// Creates a new block header with 0 nonce.
    /// This block header is only valid after [Self::solve]'ing it and changing the nonce.
    pub fn new(prev_block_hash: Hash, merkle_hash: Hash, difficulty: u32) -> Self {
        BlockHeader {
            prev_block_hash,
            merkle_hash,
            difficulty,
            nonce: 0,
        }
    }

    /// Creates a new, _valid_ block. I.e. mines/solves it such that the hash
    /// satisfies the given difficulty.
    pub fn mine_new(prev_block_hash: Hash, merkle_hash: Hash, difficulty: u32) -> Self {
        let initial = Self::new(prev_block_hash, merkle_hash, difficulty);
        Self {
            nonce: initial.solve(),
            ..initial
        }
    }

    /// Mines the nonce needed to solve this block/make it valid.
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

/// A full block on this chain.
/// A block is valid iff
/// - its [BlockHeader] is valid
/// - the hash of its [Transactions] is equal to the merkle_tree_hash of its [BlockHeader]
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Transactions,
}

pub const MAX_TXS: usize = 100;

impl Block {
    /// Create a new block. This block is only valid after mining/solving its header
    /// and changing the nonce.
    pub fn new(prev_block_hash: Hash, difficulty: u32, transactions: Transactions) -> Self {
        Block {
            header: BlockHeader::new(prev_block_hash, transactions.hash(), difficulty),
            transactions,
        }
    }

    /// Returns the first block on this chain.
    pub fn genesis() -> Self {
        Block {
            header: GENESIS_HEADER,
            transactions: Transactions::genesis(),
        }
    }

    /// Creates a new, _valid_ block. I.e. mines/solves its nonce.
    pub fn mine_new(prev_block_hash: Hash, difficulty: u32, transactions: Transactions) -> Self {
        Block {
            header: BlockHeader::mine_new(prev_block_hash, transactions.hash(), difficulty),
            transactions,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.transactions.hash() == self.header.merkle_hash && self.header.is_valid()
    }
}

impl Hashable for Block {
    /// A block's hash is only its header's hash.
    fn hash(&self) -> Hash {
        self.header.hash()
    }
}

#[cfg(test)]
mod test {
    use crate::block::{Block, BlockHeader, GENESIS_NONCE};
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
        assert!(!Block::new(PREVIOUS_HASH, 2, Transactions(txs.clone())).is_valid());
        assert!(Block::mine_new(PREVIOUS_HASH, 2, Transactions(txs)).is_valid());
    }

    #[test]
    fn genesis_block_is_valid() {
        let genesis_block = Block::genesis();
        assert_eq!(genesis_block.header.solve(), GENESIS_NONCE);
        assert!(genesis_block.is_valid());
        assert_eq!(
            genesis_block.transactions.hash(),
            genesis_block.header.merkle_hash
        );
    }
}
