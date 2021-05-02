use std::collections::{HashMap, HashSet};

use crate::{BlockTrait, Height, TxTrait};
use crypto::hash::Hash;
use linked_hash_map::LinkedHashMap;
use std::sync::Arc;


/// Storage holds on to all the blocks and transactions
/// Disable feature `mempool` if the end program does not need any client.
/// Eg., RandPiper, OptRand, or protocols that only require a bunch of servers to do something, with no inputs from clients.
pub struct Storage<B,T> {
    all_delivered_blocks_by_hash: HashMap<Hash,Arc<B>>,
    all_delivered_blocks_by_ht: HashMap<Height,Arc<B>>,
    committed_blocks_by_hash: HashSet<Hash>,
    committed_blocks_by_ht: HashSet<Height>,
    #[cfg(feature="mempool")]
    pending_tx: LinkedHashMap<Hash,Arc<T>>,
}

impl<B,T> Storage<B,T> 
where 
B: BlockTrait,
T: TxTrait,
{
    pub fn new(space: usize) -> Self {
        Storage{
            all_delivered_blocks_by_hash: HashMap::new(),
            all_delivered_blocks_by_ht: HashMap::new(),
            committed_blocks_by_hash: HashSet::new(),
            committed_blocks_by_ht: HashSet::new(),
            #[cfg(feature="mempool")]
            pending_tx: LinkedHashMap::with_capacity(space),
        }
    }

    /// Fetches a delivered block by referencing the height
    ///
    /// Returns an ARC of the Block
    pub fn delivered_block_from_ht(&self, height: Height) -> Option<Arc<B>> {
        let opt = self.all_delivered_blocks_by_ht.get(&height);
        if let None = opt {
            return None;
        }
        Some(opt.unwrap().clone())
    }

    /// Fetches a delivered block using the hash
    ///
    /// Returns an ARC of the Block
    pub fn delivered_block_from_hash(&self, hash: &Hash) -> Option<Arc<B>> {
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
    pub fn committed_block_from_ht(&self, height: Height) -> Option<Arc<B>> {
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
    pub fn committed_block_by_hash(&self, hash: &Hash) -> Option<Arc<B>> {
        if self.committed_blocks_by_hash.contains(hash) {
            self.delivered_block_from_hash(hash)
        } else {
            None
        }
    }

    /// Adds a block to delivered block.
    ///
    /// Warning: This assumes that the provided hash is correct, the caller must
    /// ensure that this agreement holds
    pub fn add_delivered_block(&mut self, b_rc: Arc<B>) {
        let ht = b_rc.get_height();
        self.all_delivered_blocks_by_hash.insert(b_rc.get_hash(), b_rc.clone());
        self.all_delivered_blocks_by_ht.insert(ht, b_rc);
    }

    /// Adds a block to delivered block.
    ///
    /// Warning: This assumes that the provided hash is correct, the caller must
    /// ensure that this agreement holds
    pub fn add_committed_block(&mut self, b_rc: Arc<B>) {
        self.committed_blocks_by_hash.insert(b_rc.get_hash());
        self.committed_blocks_by_ht.insert(b_rc.get_height());
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
    /// Used to create blocks from the pending transactions
    #[cfg(feature="mempool")]
    pub fn cleave(&mut self, block_size: usize) -> Vec<Arc<T>> {
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
    #[cfg(feature="mempool")]
    pub fn clear(&mut self, tx_hashes: &Vec<Hash>) {
        for h in tx_hashes {
            self.pending_tx.remove(h);
        }
    }

    /// Adds a transaction to the pool
    #[cfg(feature="mempool")]
    pub fn add_transaction(&mut self, t: T) {
        let tx_hash = t.get_hash();
        let t_rc = Arc::new(t);
        self.pending_tx.insert(tx_hash, t_rc);
    }

    /// Returns the number of transactions currently in the tx pool
    ///
    /// Used to determine if we are ready to propose
    #[cfg(feature="mempool")]
    pub fn get_tx_pool_size(&self) -> usize {
        self.pending_tx.len()
    }
}