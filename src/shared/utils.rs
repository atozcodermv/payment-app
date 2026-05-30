use sha2::{Digest, Sha256};

pub fn request_hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
