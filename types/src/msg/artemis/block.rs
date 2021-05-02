use crate::{BlockTrait, WireReady};
use crypto::{Keypair, PublicKey, hash::Hash};
use super::super::Block as OldBlock;
use super::{Vote, Replica, Height, Transaction};
use crate::GENESIS_BLOCK as OldGenesis;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

/// This block definition overrides the original block and adds a signature from the view leader
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub blk: OldBlock,
    pub sig: Vote,
}

pub const GENESIS_BLOCK: Block = Block {
    blk: OldGenesis,
    sig: Vote{
        auth: vec![],
        origin: 0,
    }
};

impl Block {
    pub fn with_tx(txs: Vec<Arc<Transaction>>) -> Self {
        Block {
            blk: OldBlock::with_tx(txs),
            sig: Vote{
                auth: vec![],
                origin:0,
            }
        }
    }

    /// Checks if the block is signed correctly by the holder of pk
    pub fn check_sig(&self, pk: &PublicKey) -> bool {
        pk.verify(&self.blk.hash, &self.sig.auth)
    }

    /// Adds a signature to the block. Make sure that the block is initialized (i.e., the hash is set properly)
    pub fn sign(&mut self, sk: &Keypair) {
        let auth = sk.sign(&self.blk.hash)
            .expect("Failed to sign the block");
        self.sig.auth = auth;
    }
}

impl BlockTrait for Block {
    fn get_hash(&self) -> Hash {
        self.blk.get_hash()
    }

    fn get_height(&self) -> Height {
        self.blk.get_height()
    }

    fn get_author(&self) -> Replica {
        self.blk.get_author()
    }
}

impl WireReady for Block {
    fn init(self) -> Self {
        let nblk = self.blk.init();
        Block {
            blk: nblk,
            sig: self.sig,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let bytes = bincode::serialize(self).expect("Failed to serialize Block");
        bytes
    }

    fn from_bytes(data: &[u8]) -> Self {
        let c:Self = bincode::deserialize(data)
            .expect("failed to decode the block");
        c.init()
    }
}