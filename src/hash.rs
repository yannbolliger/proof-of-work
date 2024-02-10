use bs58::encode;
use sha2::{Digest, Sha256};

/// The length of a [`Hash`] (in bytes).
pub const HASH_LENGTH: usize = 32;

/// Bytes representing a [`Sha256`] hash.
pub type Hash = [u8; HASH_LENGTH];

fn hash(bytes: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}

// Because Hash is only a type alias, we can't implement methods on it directly but
// need to do it via a trait.
pub trait B58Encode {
    /// Encode this as base58 string.
    fn encode(&self) -> String;
}

impl B58Encode for Hash {
    fn encode(&self) -> String {
        encode(self).into_string()
    }
}

pub fn has_leading_zeros(s: &Hash, leading: usize) -> bool {
    s[..leading].iter().all(|b| *b == 0)
}

/// Trait making [Sha256] hashing available on the implementor.
pub trait Hashable {
    fn hash_bytes(bytes: &[u8]) -> Hash {
        hash(bytes)
    }

    /// Hash this with [Sha256].
    /// Most implementations can use [Self::hash_bytes] and just provide all their bytes.
    fn hash(&self) -> Hash;
}

#[cfg(test)]
mod test {
    use crate::hash::{has_leading_zeros, Hash, HASH_LENGTH};

    #[test]
    fn test_check_leading_zeros() {
        assert!(has_leading_zeros(&[0; HASH_LENGTH], 0));
        assert!(has_leading_zeros(&[0; HASH_LENGTH], 1));
        assert!(has_leading_zeros(&[0; HASH_LENGTH], 0));
        assert!(has_leading_zeros(&[0; HASH_LENGTH], 32));

        assert!(has_leading_zeros(&[1; HASH_LENGTH], 0));
        assert!(!has_leading_zeros(&[1; HASH_LENGTH], 1));
        assert!(!has_leading_zeros(&[1; HASH_LENGTH], 32));

        let from_zero: Hash = core::array::from_fn(|i| i as u8);
        let from_one: Hash = core::array::from_fn(|i| i as u8 + 1);
        assert!(has_leading_zeros(&from_zero, 1));
        assert!(!has_leading_zeros(&from_zero, 2));
        assert!(has_leading_zeros(&from_one, 0));
        assert!(!has_leading_zeros(&from_one, 1));
    }
}
