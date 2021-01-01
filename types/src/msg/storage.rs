use std::collections::{HashMap};

use super::{Block, Transaction};
use crate::Height;
use crypto::hash::Hash;
use linked_hash_map::LinkedHashMap;
use std::sync::Arc;


// TODO: Use storage
pub struct Storage {
    pub all_delivered_blocks_by_hash: Box<HashMap<Hash,Arc<Block>>>,
    pub all_delivered_blocks_by_ht: Box<HashMap<Height,Arc<Block>>>,
    pub committed_blocks_by_ht: Box<HashMap<Height, Arc<Block>>>,
    pub committed_blocks_by_hash: Box<HashMap<Hash, Arc<Block>>>,
    pub pending_tx: Box<LinkedHashMap<Hash,Transaction>>,
}

impl Storage {
    pub fn new(space: usize) -> Self {
        Storage{
            all_delivered_blocks_by_hash: Box::new(HashMap::new()),
            all_delivered_blocks_by_ht: Box::new(HashMap::new()),
            committed_blocks_by_hash: Box::new(HashMap::new()),
            committed_blocks_by_ht: Box::new(HashMap::new()),
            pending_tx: Box::new(LinkedHashMap::with_capacity(space)),
        }
    }
}