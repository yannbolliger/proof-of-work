use crate::hash::{Hash, Hashable, HASH_LENGTH};
use serde::{Deserialize, Serialize};

type Address = Hash;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    spender: Address,
    receiver: Address,
    amount: u32,
}

impl Transaction {
    #[cfg(test)]
    pub fn dummy_txs(len: u32) -> Vec<Self> {
        (1..len)
            .map(|i: u32| Transaction {
                spender: [i as u8; HASH_LENGTH],
                receiver: [(i + 1) as u8; HASH_LENGTH],
                amount: i,
            })
            .collect::<Vec<_>>()
    }
}

pub const MINT_ADDRESS: Address = [1; HASH_LENGTH];
pub const GENESIS_TX: Transaction = Transaction {
    spender: MINT_ADDRESS,
    receiver: [
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ],
    amount: 100,
};

impl Hashable for Transaction {
    fn hash(&self) -> Hash {
        Self::hash_bytes(&bincode::serialize(self).unwrap())
    }
}

/// Implements merkle tree hashing for transactions
#[derive(Copy, Clone)]
pub struct Transactions<'t>(pub &'t [Transaction]);

pub const GENESIS_TXS: Transactions = Transactions(&[GENESIS_TX]);
pub const GENESIS_TXS_HASH: Hash = [
    92, 199, 78, 195, 125, 214, 27, 112, 9, 218, 38, 149, 15, 61, 223, 51, 238, 99, 110, 3, 97, 19,
    152, 59, 226, 207, 144, 91, 101, 237, 133, 25,
];

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

impl<'t> Hashable for Transactions<'t> {
    fn hash(&self) -> Hash {
        hash(self.0)
    }
}

#[cfg(test)]
mod test {
    use crate::hash::{Hashable, HASH_LENGTH};
    use crate::tx::{Transaction, Transactions, GENESIS_TXS, GENESIS_TXS_HASH};

    #[test]
    #[should_panic]
    fn test_empty() {
        Transactions(&[]).hash();
    }

    #[test]
    fn test_single() {
        let tx = Transaction {
            spender: [0; HASH_LENGTH],
            receiver: [1; HASH_LENGTH],
            amount: 100,
        };
        assert_eq!(
            Transactions(&[tx.clone(), tx.clone()]).hash(),
            Transactions(&[tx]).hash()
        );
    }

    #[test]
    fn test_many() {
        assert_eq!(GENESIS_TXS.hash(), GENESIS_TXS_HASH);
    }
}
