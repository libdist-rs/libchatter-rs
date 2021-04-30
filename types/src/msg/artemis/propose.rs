use serde::{Serialize, Deserialize};
use crypto::hash::Hash;
use std::sync::Arc;
use super::Block;

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Propose {
    pub proof: Vec<u8>,
    pub block_hash: Hash,

    #[serde(skip_serializing, skip_deserializing)]
    pub block: Option<Arc<Block>>,
}

impl Propose {
    pub fn new(block_hash: Hash) -> Self {
        Propose{
            proof:Vec::new(),
            block_hash,
            block:None,
        }
    }
}