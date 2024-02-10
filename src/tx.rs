use crate::hash::{B58Encode, Hash, Hashable, HASH_LENGTH};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};

/// An address is just a hash.
///
/// For now, we use any hash but the idea is that this could be the hash of a public key
/// from the account owner.
type Address = Hash;

/// A transaction for an amount of "coin" from a sender to a receiver address.
// TODO: there is integrity on address/account balances currently. Transactions
//   are plain/unsigned and everyone can propose/make up amounts/txs.
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Transaction {
    spender: Address,
    receiver: Address,
    amount: u32,
    timestamp: u64,
}

impl Transaction {
    /// Quickly, easily create the requested number of dummy transactions.
    /// Mostly for testing purposes.
    pub fn dummy_txs(len: u32) -> Vec<Self> {
        (1..=len)
            .map(|i: u32| Transaction {
                spender: [i as u8; HASH_LENGTH],
                receiver: [(i + 1) as u8; HASH_LENGTH],
                amount: i,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Current time is after unix epoch")
                    .as_secs(),
            })
            .collect::<Vec<_>>()
    }
}

impl Debug for Transaction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Transaction {{ spender: {}, receiver: {}, amount: {}, timestamp: {} }}",
            self.spender.encode(),
            self.receiver.encode(),
            self.amount,
            self.timestamp
        )
    }
}

/// Address used in the genesis block to mint 100 "coin".
///
/// The idea would be to forbid this address for public use and take it as "origin" for
/// newly minted "coin" in future implementations of "coinbase" transactions.
pub const MINT_ADDRESS: Address = [1; HASH_LENGTH];

/// UNIX timestamp of UTC 2024/02/10 00:00:00
pub const GENESIS_TIME: u64 = 1707519600;

/// The only transaction in the genesis block of this chain.
/// Grants 100 "coin" to the address `0x100000...`
pub const GENESIS_TX: Transaction = Transaction {
    spender: MINT_ADDRESS,
    receiver: [
        1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ],
    amount: 100,
    timestamp: GENESIS_TIME,
};

impl Hashable for Transaction {
    fn hash(&self) -> Hash {
        Self::hash_bytes(&bincode::serialize(self).unwrap())
    }
}

/// An ordered sequence of transactions for inclusion in a block.
/// Implements merkle tree hashing for transactions.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Transactions(pub Vec<Transaction>);

impl Transactions {
    pub fn genesis() -> Self {
        Transactions(vec![GENESIS_TX])
    }
}

/// The merkle tree hash of [`Transactions::genesis()`] for inclusion in the genesis block.
pub const GENESIS_TXS_HASH: Hash = [
    82, 124, 234, 124, 91, 88, 141, 159, 176, 214, 86, 126, 142, 46, 16, 73, 125, 96, 127, 71, 253,
    45, 55, 100, 237, 182, 18, 51, 33, 131, 181, 243,
];

/// Merkle tree hashing implementation as per Bitcoin:
///  - full binary tree
///  - hashes of leaves are concatenated and rehashed
///  - the last transaction is hashed twice and concatenated if the number is odd
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
    use crate::tx::{Transaction, Transactions, GENESIS_TXS_HASH};

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
            timestamp: 1,
        };
        assert_eq!(
            Transactions(vec![tx.clone(), tx.clone()]).hash(),
            Transactions(vec![tx]).hash()
        );
    }

    #[test]
    fn test_genesis() {
        assert_eq!(Transactions::genesis().hash(), GENESIS_TXS_HASH);
    }

    #[test]
    fn test_many() {
        Transactions(Transaction::dummy_txs(10000)).hash();
    }
}
