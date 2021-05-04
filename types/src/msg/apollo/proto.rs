use serde::{Serialize, Deserialize};
use super::{Block, Propose, Vote};
use crate::WireReady;
use crypto::hash::Hash;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[repr(u8)]
pub enum ProtocolMsg {
    /// Raw Proposal
    RawNewProposal(Propose, Block),
    NewProposal(Propose),

    /// Relay
    Relay(Propose),

    Request(u64, Hash),
    /// RawResponse
    RawResponse(u64, Propose, Block),
    Response(u64, Propose),

    // Blame a node
    Blame(Vote),
}

impl WireReady for ProtocolMsg {
    fn from_bytes(bytes:&[u8]) -> Self {
        let c:Self = bincode::deserialize(bytes)
            .expect("failed to decode the protocol message");
        c.init()
    }

    fn init(self) -> Self {
        match self {
            ProtocolMsg::RawNewProposal(mut prop, mut block) => {
                block.hash = block.compute_hash();
                prop.block = Some(Arc::new(block));
                ProtocolMsg::NewProposal(prop)
            },
            ProtocolMsg::RawResponse(_i, mut prop, mut block) => {
                block.hash = block.compute_hash();
                prop.block = Some(Arc::new(block));
                ProtocolMsg::Response(_i, prop)
            },
            _x => _x,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let bytes = bincode::serialize(self).expect("Failed to serialize protocol message");
        bytes
    }
}