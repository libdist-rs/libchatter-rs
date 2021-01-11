use bytes::BytesMut;
use serde::{Serialize, Deserialize};
use tokio_util::codec::{Decoder, LengthDelimitedCodec};
use crate::{Block, Payload, Propose, Vote, WireReady};
use crypto::hash::Hash;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProtocolMsg {
    RawNewProposal(Propose, Block),
    Relay(Propose),
    Blame(Vote),
    NewProposal(Propose),
    Request(u64, Hash),
    RawResponse(u64, Propose, Block),
    Response(u64, Propose),
}

impl WireReady for ProtocolMsg {
    fn from_bytes(bytes:&[u8]) -> Self {
        let c:ProtocolMsg = flexbuffers::from_slice(bytes)
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
            }
            _x => _x,
        }
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
        let c:ClientMsg = flexbuffers::from_slice(bytes)
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
}

pub struct ClientMsgCodec (pub LengthDelimitedCodec);

impl ClientMsgCodec {
    pub fn new() -> Self {
        ClientMsgCodec(LengthDelimitedCodec::new())
    }
}

impl Decoder for ClientMsgCodec {
    type Item = ClientMsg;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.0.decode(src)? {
            Some(data) => Ok(Some(ClientMsg::from_bytes(&data))),
            None => Ok(None),
        }
    }
}

impl std::clone::Clone for ClientMsgCodec {
    fn clone(&self) -> Self {
        ClientMsgCodec::new()
    }
}