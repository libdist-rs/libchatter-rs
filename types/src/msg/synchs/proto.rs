use bytes::BytesMut;
use crypto::hash::Hash;
use serde::{
    Serialize, 
    Deserialize
};
use tokio_util::codec::{Decoder, LengthDelimitedCodec};
use std::sync::Arc;
use crate::{CertType, Certificate, Payload, WireReady, msg::block::Block, synchs::Propose};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProtocolMsg {
    /// New Proposal
    RawNewProposal(Propose, Block),
    NewProposal(Propose),
    /// A Vote for the proposed block
    VoteMsg(Certificate, Propose),
    /// An equivocation blame
    /// Equivocation Blame is sent when two equivocating proposals are heard
    /// It contains
    /// 1) The leader who equivocated
    /// 2) The two equivocating blocks
    EquivcationBlameMsg(Block, Block, Certificate),
    NoProgressBlameMsg(Certificate),
    /// Certificate saying that all the nodes are waiting to quit the view
    QuitViewMsg(Certificate), 
    /// Status: Contains the block and its certificate
    StatusMsg(Block, Certificate),
    /// Invalid message
    INVALID,
}

impl ProtocolMsg {
}

impl WireReady for ProtocolMsg {
    fn from_bytes(bytes: &[u8]) -> Self {
        let c:ProtocolMsg = bincode::deserialize(bytes)
            .expect("failed to decode the protocol message");
        c.init()
    }

    fn init(self) -> Self {
        match self {
            ProtocolMsg::RawNewProposal(mut p, b) => {
                let b = b.init();
                p.block = Some(Arc::new(b));
                ProtocolMsg::NewProposal(p)
            },
            ProtocolMsg::VoteMsg(ref c, _) => {
                if let CertType::Vote(_) = &c.msg {
                    return self;
                } else {
                    ProtocolMsg::INVALID
                }
            },
            ProtocolMsg::EquivcationBlameMsg(_,_,ref c) => {
                if let CertType::Blame(_,_) = &c.msg {
                    return self;
                } else {
                    ProtocolMsg::INVALID
                }
            },
            ProtocolMsg::NoProgressBlameMsg(ref c) => {
                if let CertType::Blame(_,_) = &c.msg {
                    return self;
                } else {
                    ProtocolMsg::INVALID
                }
            },
            _x => _x,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientMsg {
    /// RawNewBlock contains Proposal, Block, and Payload
    /// Received directly from the network
    /// After init this will be transformed into a NewBlock
    RawNewBlock(Block, Payload),
    /// A processed message
    NewBlock(Block, Payload),
    /// Request an object with Hash
    Request(Hash),
    /// Respond with an object with Hash
    RawResponse(Hash, Block),
    Response(Hash, Block),
}

impl WireReady for ClientMsg {
    fn from_bytes(bytes: &[u8]) -> Self {
        let c:ClientMsg = bincode::deserialize(bytes)
            .expect("failed to decode the protocol message");
        c.init()
    }

    fn init(self) -> Self {
        match self {
            ClientMsg::RawNewBlock(mut block, payload) => {
                block.hash = block.compute_hash();
                ClientMsg::NewBlock(block, payload)
            },
            ClientMsg::RawResponse(h, mut block) => {
                block.hash = block.compute_hash();
                ClientMsg::Response(h, block)
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