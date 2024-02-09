use crate::block::{Block, GENESIS_BLOCK};
use crate::hash::{Hash, Hashable};
use std::collections::HashMap;

struct BlockEntry<'t> {
    block: Block<'t>,
    height: usize,
}

struct BlockChain<'t> {
    blocks: HashMap<Hash, BlockEntry<'t>>,
    highest_block_hash: Option<Hash>,
}

impl<'t> BlockChain<'t> {
    fn empty() -> Self {
        BlockChain {
            blocks: HashMap::new(),
            highest_block_hash: None,
        }
    }

    pub fn new() -> Self {
        let mut chain = Self::empty();
        chain.blocks.insert(
            GENESIS_BLOCK.hash(),
            BlockEntry {
                block: GENESIS_BLOCK,
                height: 0,
            },
        );
        chain.highest_block_hash = Some(GENESIS_BLOCK.hash());
        chain
    }

    fn highest_block_entry(&self) -> Option<&BlockEntry<'t>> {
        self.highest_block_hash.and_then(|e| self.blocks.get(&e))
    }

    /// Returns the latest block on the main chain
    pub fn highest_block(&self) -> Option<&Block<'t>> {
        self.highest_block_entry().map(|e| &e.block)
    }

    pub fn main_chain_length(&self) -> usize {
        self.highest_block_entry()
            .map(|e| e.height + 1)
            .unwrap_or_else(|| 0)
    }

    /// Verifies a block and if it is valid, adds it to this blockchain.
    /// Returns whether the block was accepted or not.
    // TODO: currently, this only accepts blocks for which the parent is known i.e.
    //   orphans are rejected.
    pub fn add_block(&mut self, block: &Block<'t>) -> bool {
        if let Some(parent) = block
            .is_valid()
            .then(|| self.blocks.get(&block.header.prev_block_hash))
            .flatten()
        {
            let entry = BlockEntry {
                block: block.clone(),
                height: parent.height + 1,
            };
            let hash = block.hash();
            if entry.height >= self.main_chain_length() {
                self.highest_block_hash = Some(hash);
            }
            return self.blocks.insert(hash, entry).is_none();
        }
        false
    }
}

#[cfg(test)]
mod test {
    use crate::block::{Block, GENESIS_BLOCK};
    use crate::chain::BlockChain;
    use crate::hash::Hashable;
    use crate::tx::{Transaction, Transactions};

    #[test]
    fn add_empty() {
        let mut chain = BlockChain::empty();
        assert!(!chain.add_block(&GENESIS_BLOCK));
        assert_eq!(chain.main_chain_length(), 0);
    }

    #[test]
    fn add_block() {
        let mut chain = BlockChain::new();
        assert_eq!(chain.main_chain_length(), 1);
        assert_eq!(chain.highest_block(), Some(&GENESIS_BLOCK));

        let txs = Transaction::dummy_txs(10);
        let first_block = Block::mine_new(GENESIS_BLOCK.hash(), 1, Transactions(&txs));
        assert!(chain.add_block(&first_block));
        assert_eq!(chain.main_chain_length(), 2);
        assert_eq!(chain.highest_block(), Some(&first_block));

        // add a forked block on genesis block
        let txs = Transaction::dummy_txs(2);
        let second_block = Block::mine_new(GENESIS_BLOCK.hash(), 1, Transactions(&txs));
        assert!(chain.add_block(&second_block));
        // length is still two
        assert_eq!(chain.main_chain_length(), 2);
        // highest block is still the first "highest" block
        assert_eq!(chain.highest_block(), Some(&first_block));

        // add a second block on the fork
        let txs = Transaction::dummy_txs(3);
        let third_block = Block::mine_new(second_block.hash(), 1, Transactions(&txs));
        assert!(chain.add_block(&third_block));
        assert_eq!(chain.main_chain_length(), 3);
        // now, the highest block has switched
        assert_eq!(chain.highest_block(), Some(&third_block));
    }
}
