use serde::{Serialize, Deserialize};
use super::Block;
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