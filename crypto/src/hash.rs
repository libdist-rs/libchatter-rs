use sha2::{Digest, Sha256};
use serde::Serialize;

pub const HASH_SIZE:usize = 32;
pub type Hash = [u8; HASH_SIZE];

pub const EMPTY_HASH:Hash = [0 as u8; 32];

pub fn do_hash(bytes: &[u8]) -> Hash {
    let hash = Sha256::digest(bytes);
    return hash.into();
} 

pub fn ser_and_hash(obj: &impl Serialize) -> Hash {
    let mut s = flexbuffers::FlexbufferSerializer::new();
    obj.serialize(&mut s)
        .unwrap();
    let bytes = s.view();
    return do_hash(bytes);
}