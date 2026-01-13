use sha2::{Digest, Sha256};

pub fn compute_json_hash(json_str: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(json_str.as_bytes());
    format!("{:x}", hasher.finalize())
}
