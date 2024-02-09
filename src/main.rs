use bs58::encode;
use sha2::{Digest, Sha256};

struct Block(String);

fn hash(content: &str, nonce: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}", content, nonce).as_bytes());
    let hash = hasher.finalize();
    encode(hash).into_string()
}

fn check_leading_ones(s: &str, leading: usize) -> bool {
    s.starts_with(&"1".repeat(leading))
}

/// Returns the nonce
fn solve(content: &Block, difficulty: usize) -> usize {
    (0..usize::MAX)
        .into_iter()
        .find(|n| check_leading_ones(&hash(&content.0, *n), difficulty))
        .unwrap()
}

fn main() {
    let block = Block(String::from("hello"));
    let nonce = solve(&block, 1);
    println!("{}", nonce)
}
