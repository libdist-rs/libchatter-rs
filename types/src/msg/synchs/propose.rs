use serde::{Serialize, Deserialize};
use crate::Certificate;

use super::super::{Block};
use crate::View;

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Propose {
    /// New block being proposed
    pub new_block: Block,
    /// Signature by the author
    pub proof: Vec<u8>,
    /// Certificate for the parent of this block
    pub cert: Certificate,
    /// View number for this certificate
    pub view: View,
}

impl Propose {
    pub fn new(b: Block) -> Self {
        Propose{
            new_block: b,
            proof:Vec::new(),
            cert:Certificate::empty_cert(),
            view: 0,
        }
    }

    pub fn init(&mut self) {
        self.new_block.update_hash();
    }
}