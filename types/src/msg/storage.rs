use std::collections::{HashMap, HashSet};

use super::{Block, Transaction};
use crate::Height;
use crypto::hash::Hash;
use linked_hash_map::LinkedHashMap;
use std::sync::Arc;


// TODO: Use storage
pub struct Storage {
    all_delivered_blocks_by_hash: HashMap<Hash,Arc<Block>>,
    all_delivered_blocks_by_ht: HashMap<Height,Arc<Block>>,
    committed_blocks_by_hash: HashSet<Hash>,
    committed_blocks_by_ht: HashSet<Height>,
    pending_tx: LinkedHashMap<Hash,Arc<Transaction>>,
}

impl Storage {
    pub fn new(space: usize) -> Self {
        Storage{
            all_delivered_blocks_by_hash: HashMap::new(),
            all_delivered_blocks_by_ht: HashMap::new(),
            committed_blocks_by_hash: HashSet::new(),
            committed_blocks_by_ht: HashSet::new(),
            pending_tx: LinkedHashMap::with_capacity(space),
        }
    }

    /// Fetches a delivered block by referencing the height
    ///
    /// Returns a cloned ARC of Block
    pub fn delivered_block_from_ht(&self, height: Height) -> Option<Arc<Block>> {
        let opt = self.all_delivered_blocks_by_ht.get(&height);
        if let None = opt {
            return None;
        }
        Some(opt.unwrap().clone())
    }



    /// Fetches a delivered block using the hash
    ///
    /// Returns a cloned ARC of Block
    pub fn delivered_block_from_hash(&self, hash: &Hash) -> Option<Arc<Block>> {
        let opt = self.all_delivered_blocks_by_hash.get(hash);
        if let None = opt {
            return None;
        }
        Some(opt.unwrap().clone())
    }

    /// Fetches a committed block using the height
    ///
    /// Assumes that the block is delivered
    ///
    /// Returns a cloned ARC of Block
    pub fn committed_block_from_ht(&self, height: Height) -> Option<Arc<Block>> {
        if self.committed_blocks_by_ht.contains(&height) {
            self.delivered_block_from_ht(height)
        } else {
            None
        }
    }

    /// Fetches a committed block using the hash
    /// 
    /// Assumes that the block is delivered
    ///
    /// Returns a cloned ARC of Block
    pub fn committed_block_by_hash(&self, hash: &Hash) -> Option<Arc<Block>> {
        if self.committed_blocks_by_hash.contains(hash) {
            self.delivered_block_from_hash(hash)
        } else {
            None
        }
    }

    /// Adds a block to delivered block. Optionally, provide the hash.
    ///
    /// Warning: This assumes that the provided hash is correct, the caller must
    /// ensure that this agreement holds
    pub fn add_delivered_block(&mut self, b_rc: Arc<Block>) {
        let ht = b_rc.header.height;
        self.all_delivered_blocks_by_hash.insert(b_rc.hash, b_rc.clone());
        self.all_delivered_blocks_by_ht.insert(ht, b_rc);
    }

    /// Adds a block to delivered block. Optionally, provide the hash.
    ///
    /// Warning: This assumes that the provided hash is correct, the caller must
    /// ensure that this agreement holds
    pub fn add_committed_block(&mut self, b_rc: Arc<Block>) {
        self.committed_blocks_by_hash.insert(b_rc.hash);
        self.committed_blocks_by_ht.insert(b_rc.header.height);
    }

    pub fn is_committed_by_ht(&self, height: Height) -> bool {
        self.committed_blocks_by_ht.contains(&height)
    }

    pub fn is_delivered_by_ht(&self, height: Height) -> bool {
        self.all_delivered_blocks_by_ht.contains_key(&height)
    }

    pub fn is_delivered_by_hash(&self, hash: &Hash) -> bool {
        self.all_delivered_blocks_by_hash.contains_key(hash)
    }

    pub fn is_committed_by_hash(&self, hash: &Hash) -> bool {
        self.committed_blocks_by_hash.contains(hash)
    }

    /// Cleave removes block size number of transactions from the tx pool
    ///
    /// Used for block creation
    pub fn cleave(&mut self, block_size: usize) -> Vec<Arc<Transaction>> {
        let mut txs = Vec::with_capacity(block_size);
        for _i in 0..block_size {
            let tx = match self.pending_tx.pop_front() {
                Some((_hash, trans)) => trans,
                None => {
                    panic!("Dequeued when tx pool was not block size");
                },
            };
            txs.push(tx);
        }
        txs
    }

    /// Clear removes the transaction hashes from the pool
    pub fn clear(&mut self, tx_hashes: &Vec<Hash>) {
        for h in tx_hashes {
            self.pending_tx.remove(h);
        }
    }

    /// Adds a transaction to the pool
    pub fn add_transaction(&mut self, t: Transaction) {
        let tx_hash = t.compute_hash();
        let t_rc = Arc::new(t);
        self.pending_tx.insert(tx_hash, t_rc);
    }

    /// Returns the number of transactions currently in the tx pool
    ///
    /// Used to determine if we are ready to propose
    pub fn get_tx_pool_size(&self) -> usize {
        self.pending_tx.len()
    }
}