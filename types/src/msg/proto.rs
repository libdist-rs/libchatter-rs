use serde::{Serialize, Deserialize};

use crate::{Propose, Vote};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProtocolMsg {
    NewProposal(Propose),
    Blame(Vote),
}

impl ProtocolMsg {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let c:ProtocolMsg = flexbuffers::from_slice(&bytes)
            .expect("failed to decode the protocol message");
        return c;
    }
}