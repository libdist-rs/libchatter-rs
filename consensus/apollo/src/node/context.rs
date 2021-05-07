use std::collections::VecDeque;
use fnv::FnvHashMap as HashMap;
use crypto::hash::Hash;
use crypto::{Keypair, PublicKey, ed25519, secp256k1};
use futures::channel::mpsc::UnboundedSender;
use types::apollo::{Block, GENESIS_BLOCK, Propose, ProtocolMsg, Replica, Storage, Round};
use config::Node;
use std::sync::Arc;

pub struct Context {
    /// Config context
    /// The number of nodes in the system
    num_nodes: usize,
    /// The number of faults in the system
    num_faults: usize,
    /// My ID
    myid: Replica,
    /// Everyone's public keys
    pub pub_key_map:HashMap<Replica, PublicKey>,
    /// My key 
    pub my_secret_key: Arc<Keypair>,
    /// Whether the client supports Apollo or not
    is_client_apollo_enabled: bool,

    /// Network context
    pub net_send: UnboundedSender<(Replica, Arc<ProtocolMsg>)>,
    pub cli_send: UnboundedSender<Arc<Propose>>,

    // Reordering context
    pub prop_buf: VecDeque<(Replica, Propose)>,
    pub relay_buf: VecDeque<(Replica, Propose)>,
    pub other_buf: VecDeque<(Replica, ProtocolMsg)>,
    pub future_msgs: HashMap<Round, (Replica, Propose)>,

    /// Storage context
    /// Where the blockchain and transactions are stored
    pub storage: Storage,
    /// The chain of proposals: Map of block hash to its proposal
    pub prop_chain_by_round: HashMap<Round, Arc<Propose>>,
    pub prop_chain_by_hash: HashMap<Hash, Arc<Propose>>,

    /// Round state
    round: Round,
    round_leader: Replica,

    // Protocol state
    pub last_seen_block: Arc<Block>,
    /// The blocks we are waiting for, to handle propose messages
    pub prop_waiting: HashMap<Hash, Propose>,
    /// The blocks we are waiting for to handle the propose message
    pub prop_waiting_parent: HashMap<Hash, Propose>,
    pub req_ctr:u64,
}

const EXTRA_SPACE:usize = 100;

impl Context {
    pub fn new(config:&Node,
        net_send: UnboundedSender<(Replica, Arc<ProtocolMsg>)>,
        cli_send: UnboundedSender<Arc<Propose>>,
        is_apollo_enabled: bool,
    ) -> Self {
        let mut c = Context{
            num_nodes: config.num_nodes,
            relay_buf: VecDeque::new(),
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
            pub_key_map: HashMap::default(),
            net_send,
            cli_send,
            storage: Storage::new(EXTRA_SPACE*config.block_size),
            round_leader: 0,
            round: 1,
            future_msgs: HashMap::default(),
            last_seen_block: Arc::new(GENESIS_BLOCK),
            is_client_apollo_enabled: is_apollo_enabled,
            req_ctr:0,
            prop_waiting:HashMap::default(),
            prop_waiting_parent: HashMap::default(),
            prop_chain_by_hash: HashMap::default(),
            prop_chain_by_round: HashMap::default(),
            prop_buf: VecDeque::new(),
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
        // Initialize storage
        c.storage.add_delivered_block(
            c.last_seen_block.clone()
        );
        c
    }

    #[inline]
    pub(crate) fn num_nodes(&self) -> usize {
        self.num_nodes
    }

    #[inline]
    pub(crate) fn num_faults(&self) -> usize {
        self.num_faults
    }

    #[inline]
    pub(crate) fn myid(&self) -> Replica {
        self.myid
    }

    #[inline]
    pub(crate) fn round(&self) -> Replica {
        self.round
    }

    #[inline]
    pub(crate) fn is_client_apollo_enabled(&self) -> bool {
        self.is_client_apollo_enabled
    }

    #[inline]
    pub(crate) fn round_leader(&self) -> Replica {
        self.round_leader
    }

    pub(crate) fn update_round(&mut self) {
        self.round_leader = self.next_leader();
        self.round += 1;
    }

    pub(crate) fn next_leader(&self) -> Replica {
        self.next_of(self.round_leader)
    }

    pub(crate) fn next_of(&self, prev: Replica) -> Replica {
        if prev+1 == self.num_nodes {
            0 as Replica
        } else {
            prev+1
        }
    }
}
