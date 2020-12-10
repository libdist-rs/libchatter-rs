use serde::{Serialize, Deserialize};
use super::{Transaction, Certificate};
use crate::protocol::{Replica, Height};
use crypto::hash::{EMPTY_HASH, Hash};

#[derive(Serialize, Deserialize, Clone)]
pub struct BlockBody {
    pub tx_hashes: Vec<Hash>,
    pub responses :Vec<u8>,
}

impl BlockBody {
    pub fn new(txs: Vec<Transaction>) -> Self {
        let mut hashes = Vec::new();
        for tx in txs {
            hashes.push(crypto::hash::ser_and_hash(&tx));
        }
        BlockBody{
            tx_hashes: hashes,
            responses: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize,Clone)]
pub struct BlockHeader {
    pub prev: Hash,
    pub extra: Vec<u8>,
    pub author: Replica,
    pub height: Height,
    pub blame_certificates: Vec<Certificate>,
}

impl std::fmt::Debug for BlockHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Block Header")
            .field("author", &self.author)
            .field("height", &self.height)
            .field("prev", &self.prev)
            .finish()
    }
}

impl std::fmt::Debug for BlockBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.tx_hashes.len() > 0 {
            f.debug_struct("Block Body")
                .field("Length", &self.tx_hashes.len())
                .field("First", &self.tx_hashes[0])
                .field("Last", &self.tx_hashes[self.tx_hashes.len()-1])
                .field("Payload", &self.responses)
                .finish()
        } else {
            f.debug_struct("Block Body")
                .field("Length", &self.tx_hashes.len())
                .field("Payload", &self.responses)
                .finish()
        }
    }
}

impl BlockHeader {
    pub fn new() -> Self {
        BlockHeader{
            prev:EMPTY_HASH,
            extra: Vec::new(),
            author: 0,
            height: 0,
            blame_certificates: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub body: BlockBody,

    #[serde(skip_serializing, skip_deserializing)]
    pub hash: Hash,
    // #[serde(skip_serializing, skip_deserializing)]
    pub payload: Vec<u8>,
}

impl Block {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let c:Block = flexbuffers::from_slice(&bytes)
            .expect("failed to decode the block");
        return c;
    }

    pub fn with_tx(txs: Vec<Transaction>) -> Self {
        Block{
            header: BlockHeader::new(),
            body: BlockBody::new(txs),
            hash: EMPTY_HASH,
            payload: Vec::new(),
        }
    }

    pub fn add_payload(&mut self, payload:usize) {
        self.payload = vec![1; payload];
    }

    pub fn update_hash(&mut self) {
        let temp = self.payload.drain(..).collect();
        self.hash = crypto::hash::ser_and_hash(&self);
        self.payload = temp;
    }
}

pub const GENESIS_BLOCK: Block = Block{
    header: BlockHeader{
        prev:EMPTY_HASH,
        extra: Vec::new(),
        author: 0,
        height: 0,
        blame_certificates: Vec::new(),
    },
    body: BlockBody{
        tx_hashes: Vec::new(),
        responses: Vec::new(),
    },
    hash: EMPTY_HASH,
    payload: vec![],
    // cert: Certificate{
        // votes: vec![],
    // },
};