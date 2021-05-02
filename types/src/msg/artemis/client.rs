use serde::{Serialize, Deserialize};
use super::{Block, Payload, UCRVote};
use crypto::hash::Hash; 
use crate::{BlockTrait, WireReady};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientMsg {
    /// RawNewBlock contains a `UCRVote`, and a series of blocks with payloads from the last UCRVote
    /// Received directly from the network
    /// After init this will be transformed into a NewBlock
    RawNewBlock(UCRVote, Vec<(Block, Payload)>),
    /// A processed message
    NewBlock(UCRVote, Vec<(Block, Payload)>),
    /// Request a block with Hash
    RequestBlock(Hash),
    /// Respond with an object with Hash
    RawResponseBlock(Hash, Block),
    ResponseBlock(Hash, Block),
    /// Invalid implies that during transformation from network and back, we got some error
    Invalid,
}

impl WireReady for ClientMsg {
    fn from_bytes(bytes: &[u8]) -> Self {
        let c:Self = bincode::deserialize(bytes)
            .expect("failed to decode the protocol message");
        c.init()
    }

    fn init(self) -> Self {
        match self {
            ClientMsg::RawNewBlock(vote, block_vec) => {
                if block_vec.len() == 0 {
                    log::warn!("Got a vote with 0 blocks");
                    return ClientMsg::Invalid;
                }
                let block_vec:Vec<_> = 
                block_vec.into_iter().map(|(block,pl)| {
                    let block = block.init();
                    (block, pl)
                }).collect();
                if block_vec.last().unwrap().0.get_hash() != vote.hash {
                    log::warn!("The hash of the last block is not the hash in the vote");
                    return ClientMsg::Invalid;
                }
                ClientMsg::NewBlock(vote, block_vec)
            },
            ClientMsg::RawResponseBlock(h, block) => {
                let block = block.init();
                if block.get_hash() == h {
                    ClientMsg::ResponseBlock(h, block)
                } else {
                    ClientMsg::Invalid
                }
            }
            _x => _x,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let bytes = bincode::serialize(self).expect("Failed to serialize client message");
        bytes
    }
}