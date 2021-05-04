use serde::{Serialize, Deserialize};
use super::*;
use crypto::hash::Hash;
use crate::WireReady;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientMsg {
    /// RawNewBlock contains Proposal, Block, and Payload
    /// Received directly from the network
    /// After init this will be transformed into a NewBlock
    RawNewBlock(Propose, Block, Payload),
    /// A processed message
    NewBlock(Propose, Payload),
    /// Request an object with Hash
    Request(Hash),
    /// Respond with an object with Hash
    RawResponse(Hash, Propose, Block),
    Response(Hash, Propose),
}

impl WireReady for ClientMsg {
    fn from_bytes(bytes: &[u8]) -> Self {
        let c:Self = bincode::deserialize(bytes)
            .expect("failed to decode the protocol message");
        c.init()
    }

    fn init(self) -> Self {
        match self {
            ClientMsg::RawNewBlock(mut prop, mut block, payload) => {
                block.hash = block.compute_hash();
                prop.block = Some(Arc::new(block));
                ClientMsg::NewBlock(prop, payload)
            },
            ClientMsg::RawResponse(h, mut prop, mut block) => {
                block.hash = block.compute_hash();
                prop.block = Some(Arc::new(block));
                ClientMsg::Response(h, prop)
            }
            _x => _x,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let bytes = bincode::serialize(self).expect("Failed to serialize client message");
        bytes
    }
}