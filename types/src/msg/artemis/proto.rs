use serde::{Serialize, Deserialize};
use super::{Block, UCRVote, Vote};
use crate::WireReady;
use crypto::hash::Hash;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[repr(u8)]
pub enum ProtocolMsg {
    /// New Block message over the network
    RawNewBlock(Block),
    /// New Block message sent by the leader
    NewBlock(Block),

    /// UCRVote message over the network
    RawUCRVote(UCRVote),
    /// UCRVote contains the original UCRVote along with a signature on the UCR message
    UCRVote(UCRVote),

    /// Forward a vote from the Round leader
    Relay(UCRVote),

    Blame(Vote),
    /// Request contains
    /// - Request ID
    /// - Hash of the block
    Request(u64, Hash),
    
    /// RawResponse
    RawResponse(u64, Block),
    /// Response consists:
    /// - counter
    /// - hash of the requested block
    /// - Block
    Response(u64, Block),

    /// Invalid messages
    Invalid
}

impl WireReady for ProtocolMsg {
    fn from_bytes(bytes:&[u8]) -> Self {
        let c:Self = bincode::deserialize(bytes)
            .expect("failed to decode the protocol message");
        c.init()
    }

    fn init(self) -> Self {
        match self {
            ProtocolMsg::RawResponse(_i, block) => {
                let block = block.init();
                ProtocolMsg::Response(_i, block)
            },
            ProtocolMsg::RawNewBlock(b) => {
                ProtocolMsg::NewBlock(b.init())
            },
            ProtocolMsg::RawUCRVote(v) => {
                ProtocolMsg::UCRVote(v)
            }
            _x => _x,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let bytes = bincode::serialize(self).expect("Failed to serialize protocol message");
        bytes
    }
}