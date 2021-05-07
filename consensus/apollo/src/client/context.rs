use std::time::SystemTime;
use fnv::FnvHashMap as HashMap;

use types::apollo::{GENESIS_BLOCK, Propose, Round, Storage};
use crypto::hash::Hash;
use std::sync::Arc;


pub struct Context {
    pub pending: usize,
    pub num_cmds: u128,
    pub time_map: HashMap<Hash, SystemTime>,
    pub latency_map: HashMap<Hash, (SystemTime, SystemTime)>,
    pub storage: Storage,
    pub round: Round,
    pub future_msgs: HashMap<Round, Propose>,
}

impl Context {
    pub fn new() -> Self {
        let genesis_arc = Arc::new(GENESIS_BLOCK);
        let mut cx = Context {
            pending: 0,
            num_cmds: 0,
            time_map: HashMap::default(),
            latency_map: HashMap::default(),
            storage: Storage::new(100_000),
            round:1,
            future_msgs: HashMap::default(),
        };
        cx.storage.add_delivered_block(genesis_arc.clone());
        cx.storage.add_committed_block(genesis_arc);
        cx
    }
}