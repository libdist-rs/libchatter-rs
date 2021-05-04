use serde::{Serialize, Deserialize};
use crate::{Block, Payload, Propose, Vote, WireReady};
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

    Blame(Vote),
    Request(u64, Hash),
    
    /// RawResponse
    RawResponse(u64, Propose, Block),
    Response(u64, Propose),
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
            // ProtocolMsg::RawRelay(mut prop, mut block) => {
            //     block = block.init();
            //     prop.block = Some(Arc::new(block));
            //     ProtocolMsg::Relay(prop)
            // },
            _x => _x,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let bytes = bincode::serialize(self).expect("Failed to serialize protocol message");
        bytes
    }
}

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