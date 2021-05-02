use std::collections::{HashMap, VecDeque};
use crypto::hash::Hash;
use crypto::{Keypair, PublicKey, ed25519, secp256k1};
use futures::channel::mpsc::UnboundedSender;
use types::artemis::{Block, ClientMsg, GENESIS_BLOCK, ProtocolMsg, Replica, Round, Storage, UCRVote, View};
use config::Node;
use std::sync::Arc;

/// Config context
pub struct Context {
    /// The number of nodes in the system
    num_nodes: usize,
    /// The number of faults
    num_faults: usize,
    /// myid in the protocol
    myid: Replica,
    /// Map of node IDs and public keys 
    pub pub_key_map: HashMap<Replica, PublicKey>,
    /// My Secret Key
    my_secret_key: Arc<Keypair>,
    /// Whether or not our client supports UCR or not.
    /// If yes, UCR is enabled, and we send the block on proposing.
    /// If no, UCR is disabled, and we notify the client on committing.
    is_client_apollo_enabled: bool,

    /// A channel to send all protocol messages to nodes outside the system
    pub net_send: UnboundedSender<(Replica, Arc<ProtocolMsg>)>,
    /// A channel to communicate to all the nodes
    pub cli_send: UnboundedSender<Arc<ClientMsg>>,
    
    /// Storage context. Permanent storage goes here.
    /// The blockchain and transactions are stored here.
    pub storage: Storage,
    /// The vote chain: Map of block hash to its proposal
    pub vote_chain: HashMap<Round, Arc<UCRVote>>,
    
    /// The current round leader
    pub round_leader: Replica,
    /// The current view leader
    pub view_leader: Replica,
    /// The current view
    pub view: View,
    /// The current round
    pub round: Round,
    /// The last observed block
    pub last_seen_block: Arc<Block>,
    /// The last block for which we have seen vote messages for
    pub last_voted_block: Arc<Block>,
    /// A counter to keep track of all the requests
    pub req_ctr:u64,
    
    // Stuff related to message reordering
    /// The blocks we are waiting for, to handle vote messages
    /// If I get the `b` for this hash `h`, and `b` is delivered, then I can process this vote, and move it to `vote_ready`
    pub vote_waiting: HashMap<Hash, UCRVote>,
    /// These are votes that were waiting, but got moved here when their blocks got delivered.
    pub vote_ready: HashMap<Round,UCRVote>,
    /// The actual blocks received from the view leader
    pub block_processing_waiting: VecDeque<Block>,
    /// Response waiting stores:
    /// - The sender of the response
    /// - The hash of the response
    /// - The block for this hash
    pub response_waiting: VecDeque<(Replica, Block)>,
    /// A buffer containing all the other protocol messages:
    /// - UCRVote
    /// - Relay
    /// - Blame
    /// - Request
    pub other_buf: VecDeque<(Replica, ProtocolMsg)>,

    /// Block waiting (hash1, hash2)
    /// The block with hash2 is waiting for a block with hash1
    pub block_parent_waiting: HashMap<Hash, Hash>,
    /// Undelivered blocks (h, b)
    /// The block b with hash h is waiting for something to get delivered
    pub undelivered_blocks: HashMap<Hash, Block>,
}

const EXTRA_SPACE:usize = 100;

impl Context {
    pub fn new(config:&Node,
        net_send: UnboundedSender<(Replica, Arc<ProtocolMsg>)>,
        cli_send: UnboundedSender<Arc<ClientMsg>>,
        apollo_enabled: bool,
    ) -> Self
    {
        let genesis_arc = Arc::new(GENESIS_BLOCK);
        let mut c = Context{
            num_nodes: config.num_nodes,
            num_faults: config.num_faults,
            myid: config.id,
            my_secret_key: match config.crypto_alg {
                crypto::Algorithm::ED25519 => {
                    let mut sk_copy = config.secret_key_bytes.clone();
                    let kp = ed25519::Keypair::decode(
                        &mut sk_copy
                    ).expect("Failed to decode the secret key from the config");
                    Arc::new(Keypair::Ed25519(kp))
                },
                crypto::Algorithm::SECP256K1 => {
                    let sk_copy = config.secret_key_bytes.clone();
                    let sk = secp256k1::SecretKey::from_bytes(sk_copy).expect("Failed to decode the secret key from the config");
                    let kp = secp256k1::Keypair::from(sk);
                    Arc::new(Keypair::Secp256k1(kp))
                }
                _ => panic!("Unimplemented algorithm"),
            },
            pub_key_map: HashMap::with_capacity(config.num_nodes),
            net_send,
            cli_send,
            storage: Storage::new(EXTRA_SPACE*config.block_size),
            view_leader: 0,
            round_leader:0,
            view:0,
            round: 0,
            last_seen_block: genesis_arc.clone(),
            last_voted_block: genesis_arc,
            is_client_apollo_enabled: apollo_enabled,
            req_ctr:0,
            vote_waiting:HashMap::new(),
            vote_ready:HashMap::new(),
            vote_chain: HashMap::new(),
            block_parent_waiting:HashMap::new(),
            undelivered_blocks:HashMap::new(),
            block_processing_waiting: VecDeque::new(),
            response_waiting: VecDeque::new(),
            other_buf: VecDeque::new(),
        };
        for (id,mut pk_data) in &config.pk_map {
            if *id == c.myid {
                continue;
            }
            let pk = match config.crypto_alg {
                crypto::Algorithm::ED25519 => {
                    let kp = ed25519::PublicKey::decode(
                        &mut pk_data
                    ).expect("Failed to decode the secret key from the config");
                    PublicKey::Ed25519(kp)
                },
                crypto::Algorithm::SECP256K1 => {
                    let sk = secp256k1::PublicKey::decode(&pk_data).expect("Failed to decode the secret key from the config");
                    PublicKey::Secp256k1(sk)
                }
                _ => panic!("Unimplemented algorithm"),
            };
            c.pub_key_map.insert(*id, pk);
        }
        // Initialize storage with the genesis block
        c.storage.add_delivered_block(
            c.last_seen_block.clone()
        );
        c
    }

    /// Returns the next round leader
    pub fn next_round_leader(&self) -> Replica {
        self.next_of(self.round_leader)
    }

    pub(crate) fn next_of(&self, prev: Replica) -> Replica {
        (prev+1)%self.num_nodes
    }

    // Returns the number of nodes
    #[inline]
    pub const fn num_nodes(&self) -> usize {
        self.num_nodes
    }

    // Returns the number of faults (f)
    #[inline]
    pub const fn num_faults(&self) -> usize {
        self.num_faults
    }

    // Returns the ID of this node
    #[inline]
    pub const fn myid(&self) -> usize {
        self.myid
    }

    // Returns the secret key of this node
    #[inline]
    pub fn my_secret_key(&self) -> Arc<Keypair> {
        self.my_secret_key.clone()
    }

    // Returns whether the clients are special or not
    #[inline]
    pub const fn is_client_apollo_enabled(&self) -> bool {
        self.is_client_apollo_enabled
    }
}
