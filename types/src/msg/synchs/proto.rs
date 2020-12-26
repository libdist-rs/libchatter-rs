use serde::{
    Serialize, 
    Deserialize
};

use crate::{
    Certificate, 
    Replica, 
    View, 
    Vote, 
    msg::block::Block, 
    synchs::Propose,
    WireReady
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProtocolMsg {
    /// Identification message to tell the other node that I am node ID
    Identify(Replica),
    /// New Proposal
    NewProposal(Propose),
    /// Can be a blame or a vote
    Vote(Vote),
    /// VoteMsg because a vote needs to have a block
    VoteMsg(Vote, Propose), 
    /// Certificate saying that all the nodes are waiting to quit the view
    QuitView(View, Certificate), 
    /// Status: Contains the block and its certificate
    Status(Block, Certificate),
}

impl ProtocolMsg {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let c:ProtocolMsg = flexbuffers::from_slice(&bytes)
            .expect("failed to decode the protocol message");
        return c;
    }
}

impl WireReady for ProtocolMsg {}