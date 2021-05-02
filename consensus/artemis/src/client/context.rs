use config::Client;
use fnv::FnvHashMap as HashMap;
use crypto::hash::Hash;
use std::sync::Arc;
use types::artemis::{Block, GENESIS_BLOCK, Payload, Round, Storage, UCRVote};
use std::time::SystemTime;

pub(crate) struct Context {
    /// The config for this instance of the protocol
    pub config: Arc<Client>,
    /// The number of unproposed transactions
    pub pending: usize,
    /// The number of committed commands
    pub num_cmds: u128,
    /// `time_map` contains (h, t) holds the time `t` at which we sent the transaction with `h` 
    pub time_map: HashMap<Hash, SystemTime>,
    /// The latency map contains the (start time, end time) for every transaction with hash `h`
    pub latency_map: HashMap<Hash, (SystemTime, SystemTime)>,
    /// To hold all of our blocks
    pub storage: Storage,
    /// The current round
    pub round: Round,
    /// Future messages
    pub future_msgs: HashMap<Round, (UCRVote, Vec<(Block, Payload)>)>,
}

impl Context {
    pub fn new(config: Arc<Client>) -> Self {
        let genesis_arc = Arc::new(GENESIS_BLOCK);
        let mut cx = Context {
            config,
            pending: 0,
            num_cmds: 0,
            time_map: HashMap::default(),
            latency_map: HashMap::default(),
            storage: Storage::new(100_000),
            round: 0,
            future_msgs: HashMap::default(),
        };
        cx.storage.add_delivered_block(genesis_arc);
        cx
    }
}
