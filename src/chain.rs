use crate::block::Block;
use crate::hash::{Hash, Hashable};
use std::collections::HashMap;

struct BlockEntry {
    block: Block,
    height: usize,
}

pub struct BlockChain {
    blocks: HashMap<Hash, BlockEntry>,
    highest_block_hash: Hash,
}

impl BlockChain {
    pub fn new() -> Self {
        let genesis_block = Block::genesis();
        let genesis_hash = genesis_block.hash();
        BlockChain {
            blocks: HashMap::from([(
                genesis_hash,
                BlockEntry {
                    block: genesis_block,
                    height: 0,
                },
            )]),
            highest_block_hash: genesis_hash,
        }
    }

    fn highest_block_entry(&self) -> &BlockEntry {
        self.blocks
            .get(&self.highest_block_hash)
            .expect("highest block hash must be in the chain")
    }

    /// Returns the latest block on the main chain
    pub fn highest_block(&self) -> &Block {
        &self.highest_block_entry().block
    }

    pub fn main_chain_length(&self) -> usize {
        self.highest_block_entry().height + 1
    }

    /// Verifies a block and if it is valid, adds it to this blockchain.
    /// Returns whether the block was accepted and new or not.
    // TODO: currently, this only accepts blocks for which the parent is known i.e.
    //   orphans are rejected.
    pub fn add_block(&mut self, block: &Block) -> bool {
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
                self.highest_block_hash = hash;
            }
            return self.blocks.insert(hash, entry).is_none();
        }
        false
    }
}

#[cfg(test)]
mod test {
    use crate::block::Block;
    use crate::chain::BlockChain;
    use crate::hash::Hashable;
    use crate::tx::{Transaction, Transactions};

    #[test]
    fn add_block() {
        let mut chain = BlockChain::new();
        assert_eq!(chain.main_chain_length(), 1);
        assert_eq!(chain.highest_block(), &Block::genesis());
        let genesis_hash = chain.highest_block().hash();

        let txs = Transaction::dummy_txs(10);
        let first_block = Block::mine_new(genesis_hash, 1, Transactions(txs));
        assert!(chain.add_block(&first_block));
        assert_eq!(chain.main_chain_length(), 2);
        assert_eq!(chain.highest_block(), &first_block);

        // add a forked block on genesis block
        let txs = Transaction::dummy_txs(2);
        let second_block = Block::mine_new(genesis_hash, 1, Transactions(txs));
        assert!(chain.add_block(&second_block));
        // length is still two
        assert_eq!(chain.main_chain_length(), 2);
        // highest block is still the first "highest" block
        assert_eq!(chain.highest_block(), &first_block);

        // add a second block on the fork
        let txs = Transaction::dummy_txs(3);
        let third_block = Block::mine_new(second_block.hash(), 1, Transactions(txs));
        assert!(chain.add_block(&third_block));
        assert_eq!(chain.main_chain_length(), 3);
        // now, the highest block has switched
        assert_eq!(chain.highest_block(), &third_block);
    }
}
