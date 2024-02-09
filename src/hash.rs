use bs58::encode;
use sha2::{Digest, Sha256};

/// The length of a `Hash` (in bytes).
pub const HASH_LENGTH: usize = 32;
pub type Hash = [u8; HASH_LENGTH];

fn hash(bytes: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}

pub trait B58Encode {
    fn encode(&self) -> String;
}

impl B58Encode for Hash {
    fn encode(&self) -> String {
        encode(self).into_string()
    }
}

pub fn check_leading_zeros(s: &Hash, leading: usize) -> bool {
    s[..leading].iter().all(|b| *b == 0)
}

pub trait Hashable {
    fn bytes(&self) -> Vec<u8>;

    fn hash(&self) -> Hash {
        hash(&self.bytes())
    }
}
