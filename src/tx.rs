use crate::hash::{Hash, Hashable, HASH_LENGTH};
use serde::{Deserialize, Serialize};

type Address = Hash;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    spender: Address,
    receiver: Address,
    amount: u32,
}

impl Hashable for Transaction {
    fn hash(&self) -> Hash {
        Self::hash_bytes(&bincode::serialize(self).unwrap())
    }
}

/// Implements merkle tree hashing for transactions
pub struct Transactions(pub Vec<Transaction>);

impl Transactions {
    #[cfg(test)]
    pub fn dummy_txs(len: u32) -> Self {
        Transactions(
            (1..1000)
                .map(|i: u32| Transaction {
                    spender: [i as u8; HASH_LENGTH],
                    receiver: [(i + 1) as u8; HASH_LENGTH],
                    amount: i,
                })
                .collect(),
        )
    }
}

#[inline]
fn hash(txs: &[Transaction]) -> Hash {
    match txs {
        // leaf of tree with two txs => concat their hashes and hash
        [tx1, tx2] => Transaction::hash_bytes(&[tx1.hash(), tx2.hash()].concat()),
        // leaf with single tx, i.e. last tx => concat hash with itself
        [tx] => {
            let hash = tx.hash();
            Transaction::hash_bytes(&[hash, hash].concat())
        }
        [] => panic!("cannot hash an empty merkle tree"),
        more_txs => {
            let (a, b) = more_txs.split_at(more_txs.len() / 2);
            Transaction::hash_bytes(&[hash(a), hash(b)].concat())
        }
    }
}

impl Hashable for Transactions {
    fn hash(&self) -> Hash {
        hash(&self.0)
    }
}

#[cfg(test)]
mod test {
    use crate::hash::{Hashable, HASH_LENGTH};
    use crate::tx::{Transaction, Transactions};

    #[test]
    #[should_panic]
    fn test_empty() {
        Transactions(vec![]).hash();
    }

    #[test]
    fn test_single() {
        let tx = Transaction {
            spender: [0; HASH_LENGTH],
            receiver: [1; HASH_LENGTH],
            amount: 100,
        };
        assert_eq!(
            Transactions(vec![tx.clone(), tx.clone()]).hash(),
            Transactions(vec![tx]).hash()
        );
    }

    #[test]
    fn test_many() {
        Transactions::dummy_txs(1000).hash();
    }
}
