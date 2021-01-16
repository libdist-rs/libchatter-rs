use crypto::hash::Hash;
use serde::{Serialize, Deserialize};

use crate::WireReady;

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Transaction {
    pub data: Vec<u8>,
    pub request: Vec<u8>,
}

impl Transaction {
    pub fn compute_hash(&self) -> Hash {
        crypto::hash::ser_and_hash(self)
    }
}

impl WireReady for Transaction {
    fn from_bytes(data: &[u8]) -> Self {
        let c:Transaction = bincode::deserialize(data)
            .expect("failed to decode the block");
        c.init()
    }

    fn init(self) -> Self {
        self
    }
}
