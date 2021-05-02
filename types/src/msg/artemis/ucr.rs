use crypto::{Keypair, PublicKey, hash::{EMPTY_HASH, Hash, ser_and_hash}};
use serde::{Serialize, Deserialize};

use super::{Round, View, Vote};

#[derive(Debug, Serialize, Deserialize, Clone)]
/// UCRVote message is sent by the round leader
/// It contains
/// - Hash: The hash of the block
/// - View: The view number for voting
/// - Round: The UCR round for this vote
/// - Vote: A vote on this message 
pub struct UCRVote {
    pub hash: Hash,
    pub round: Round,
    pub view: View,
    // Private so you are forced to use `compute_sig`
    vote: Vote,
}

#[derive(Serialize, Deserialize)]
struct InternalUCRVote {
    hash: Hash,
    round: Round, 
    view: View,
}

impl UCRVote {

    /// Compute and update the signature on this vote
    /// Ensure that the `hash`, `view` and `round` are set
    /// If you change any of the above, don't forget to update the signature
    pub fn compute_sig(&mut self, sk:&Keypair) {
        let ser = ser_and_hash(&self.to_internal());
        self.vote.auth = sk.sign(&ser)
            .expect("Failed to sign a ucr message");
    }

    /// Check the signature on this message
    pub fn check_sig(&self, pk: &PublicKey) -> bool {
        let ser = ser_and_hash(&self.to_internal());
        pk.verify(&ser, &self.vote.auth)
    }

    fn to_internal(&self) -> InternalUCRVote {
        InternalUCRVote{
            hash: self.hash,
            round: self.round,
            view: self.view,
        }
    }

    /// Get an empty vote instance with defaults
    pub fn new() -> Self {
        Self {
            hash: EMPTY_HASH,
            round: 0,
            view: 0,
            vote: Vote{
                auth: Vec::new(),
                origin: 0,
            },
        }
    }
}