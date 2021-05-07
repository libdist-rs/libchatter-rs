use config::Client;
use fnv::FnvHashMap as HashMap;
use crypto::hash::Hash;
use std::{sync::Arc, convert::TryInto};
use types::artemis::{Block, GENESIS_BLOCK, Payload, Round, Storage, UCRVote, Replica};
use std::time::SystemTime;
use linked_hash_map::LinkedHashMap;

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
    /// Prop chain
    pub prop_chain: HashMap<Round, Arc<UCRVote>>,
    /// The current round
    round: Round,
    /// The current round leader
    pub round_leader: Replica,
    /// Future messages
    pub future_msgs: HashMap<Round, (UCRVote, Vec<(Block, Payload)>)>,

    /// The last f leaders
    last_f_leaders: LinkedHashMap<Replica,()>,
    /// Eligible leaders
    eligible_leaders: Vec<Replica>,
}

impl Context {
    pub fn new(config: Arc<Client>) -> Self {
        let genesis_arc = Arc::new(GENESIS_BLOCK);
        let mut cx = Context {
            pending: 0,
            num_cmds: 0,
            time_map: HashMap::default(),
            latency_map: HashMap::default(),
            storage: Storage::new(100_000),
            round: 1,
            prop_chain:HashMap::default(),
            round_leader:config.num_faults-1,
            future_msgs: HashMap::default(),
            last_f_leaders: LinkedHashMap::with_capacity(config.num_nodes),
            eligible_leaders: Vec::with_capacity(config.num_nodes),
            config,
        };
        cx.storage.add_delivered_block(genesis_arc);
         // Initialize the leaders
        for i in 0..cx.config.num_faults {
            cx.last_f_leaders.insert(i, ());
        }
        for i in cx.config.num_faults..cx.config.num_nodes {
            cx.eligible_leaders.push(i);
        }
        cx
    }

    /// Goes to the next round
    pub(crate) fn update_round(&mut self) {
        // First update the round leader
        let (new_leader, idx) = self.compute_next_round_leader();
        self.round_leader = new_leader;
        // Then update the round
        // This order is important, otherwise, some other parts of the code may call next_round_leader() and changing the round before setting cx.round_leader = cx.next_round_leader() will cause problems (I LEARNT IT THE HARD WAY)
        self.round += 1;
        // Make the f^th leader elligible again
        let (eligible_again,_) = self.last_f_leaders.pop_front().unwrap();
        self.last_f_leaders.insert(self.round_leader, ());
        self.eligible_leaders[idx] = eligible_again;
    }
    
    /// This is a private function that returns both the next leader and its index in the eligible leaders vector
    /// Returns the next round leader
    /// This function will be called several times before finally updating the round leader
    /// When changing the leader do not change the following variables:
    /// - round
    /// - eligible_leaders
    fn compute_next_round_leader(&self) -> (Replica, usize) {
        let data = (self.round+1).to_be_bytes();
        let h = crypto::hash::do_hash(&data);
        let idx = usize::from_be_bytes(h[24..].try_into().unwrap()) % self.eligible_leaders.len();
        (self.eligible_leaders[idx], idx)
    }

     /// Returns the current round 
    /// We want to ensure read only access to this value
    #[inline]
    pub const fn round(&self) -> Round {
        self.round
    }
}
