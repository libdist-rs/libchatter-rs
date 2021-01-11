use crypto::hash::{EMPTY_HASH, Hash};
use serde::{Serialize, Deserialize};
use crate::Certificate;
use std::sync::Arc;
use crate::{View, Block};

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Propose {
    /// Signature by the author
    pub proof: Vec<u8>,
    /// Certificate for the parent of this block
    pub cert: Certificate,
    /// View number for this certificate
    pub view: View,
    /// Hash of the block being proposed
    pub block_hash: Hash,

    /// Cache
    #[serde(skip_serializing, skip_deserializing)]
    pub block: Option<Arc<Block>>,
}

impl Propose {
    pub fn new() -> Self {
        Self{
            proof:Vec::new(),
            cert:Certificate::empty_cert(),
            view: 0,
            block: None,
            block_hash: EMPTY_HASH,
        }
    }
}