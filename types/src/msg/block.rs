use serde::{Serialize, Deserialize};
use super::{Transaction, Certificate};
use crate::{BlockTrait, WireReady, protocol::{Replica, Height}};
use crypto::hash::{EMPTY_HASH, Hash};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: Header,
    pub body: Body,

    // Cache
    #[serde(skip)]
    pub hash: Hash,
}

impl Block {
    pub fn with_tx(txs: Vec<Arc<Transaction>>) -> Self {
        Block{
            header: Header::new(),
            body: Body::new(txs),
            hash: EMPTY_HASH,
        }
    }

    pub fn compute_hash(&self) -> Hash {
        crypto::hash::ser_and_hash(self)
    }
}

pub const GENESIS_BLOCK: Block = Block{
    header: Header{
        prev:EMPTY_HASH,
        extra: Vec::new(),
        author: 0,
        height: 0,
        blame_certificates: Vec::new(),
    },
    body: Body{
        tx_hashes: Vec::new(),
    },
    hash: EMPTY_HASH,
};

impl WireReady for Block {
    fn from_bytes(data: &[u8]) -> Self {
        let c:Self = bincode::deserialize(data)
            .expect("failed to decode the block");
        c.init()
    }
    
    fn init(mut self) -> Self {
        self.hash = self.compute_hash();
        self
    }

    fn to_bytes(&self) -> Vec<u8> {
        let bytes = bincode::serialize(self).expect("Failed to serialize Block");
        bytes
    }
}

impl BlockTrait for Block {
    fn get_hash(&self) -> Hash {
        self.hash
    }

    fn get_height(&self) -> Height {
        self.header.height
    }

    fn get_author(&self) -> Replica {
        self.header.author
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Body {
    pub tx_hashes: Vec<Hash>,
}

impl Body {
    pub fn new(txs: Vec<Arc<Transaction>>) -> Self {
        let mut hashes = Vec::new();
        for tx in txs {
            hashes.push(crypto::hash::ser_and_hash(tx.as_ref()));
        }
        Self{
            tx_hashes: hashes,
        }
    }
}

#[derive(Serialize, Deserialize,Clone)]
pub struct Header {
    pub prev: Hash,
    pub extra: Vec<u8>,
    pub author: Replica,
    pub height: Height,
    pub blame_certificates: Vec<Certificate>,
}

impl std::fmt::Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Block Header")
            .field("author", &self.author)
            .field("height", &self.height)
            .field("prev", &self.prev)
            .finish()
    }
}

impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.tx_hashes.len() > 0 {
            f.debug_struct("Block Body")
                .field("Length", &self.tx_hashes.len())
                .field("First", &self.tx_hashes[0])
                .field("Last", &self.tx_hashes[self.tx_hashes.len()-1])
                .finish()
        } else {
            f.debug_struct("Block Body")
                .field("Length", &self.tx_hashes.len())
                .finish()
        }
    }
}

impl Header {
    pub fn new() -> Self {
        Header{
            prev:EMPTY_HASH,
            extra: Vec::new(),
            author: 0,
            height: 0,
            blame_certificates: Vec::new(),
        }
    }
}

