use serde::{Serialize, Deserialize};

use crate::protocol::*;
use super::block::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VoteType {
    NoProgressBlame,
    EquivcationBlame(Block, Block), 
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vote {
    pub msg: VoteType,
    pub origin: Replica,
    pub target: Replica,
    pub auth: Vec<u8>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Certificate {
    pub votes: Vec<Vote>,
}
    
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Transaction {
    pub data: Vec<u8>,
}
    


#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Propose {
    pub new_block: Block,
    pub proof: Vec<u8>,
}

impl Propose {
    pub fn new(b: Block) -> Self {
        Propose{
            new_block: b,
            proof:Vec::new(),
        }
    }
}
