use serde::{Serialize, Deserialize};
use crypto::{Keypair, PublicKey, hash::Hash};
use std::sync::Arc;
use super::*;

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Propose {
    pub sig: Vote,
    pub round: Round,
    pub block_hash: Hash,

    #[serde(skip)]
    pub block: Option<Arc<Block>>,
}

impl Propose {
    pub fn new(block_hash: Hash) -> Self {
        Propose{
            round: 0,
            sig:Vote {
                auth: Vec::new(),
                origin: 0,
            },
            block_hash,
            block:None,
        }
    }

    /// How to generate a signature for the proposal
    pub fn sign_block(&mut self, b: &Block, sk: &Keypair) {
        let auth = sk.sign(&crypto::hash::ser_and_hash(b))
            .expect("Failed to sign a block");
        self.sig.auth = auth;
    }

    /// Check the signature of this proposal on the block
    pub fn check_sig(&self, b:&Block, pk: &PublicKey) -> bool {
        pk.verify(&crypto::hash::ser_and_hash(b), &self.sig.auth)
    }
}