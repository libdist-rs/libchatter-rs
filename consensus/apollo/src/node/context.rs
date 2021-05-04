use std::collections::VecDeque;
use fnv::{FnvHashMap as HashMap, FnvHashSet as HashSet};
use crypto::hash::Hash;
use crypto::{Keypair, PublicKey, ed25519, secp256k1};
// use tokio::sync::mpsc::UnboundedSender;
use futures::channel::mpsc::UnboundedSender;
use types::apollo::{Block, GENESIS_BLOCK, Propose, ProtocolMsg, Replica, Storage};
use config::Node;
use std::sync::Arc;

pub struct Context {
    /// Config context
    pub num_nodes: usize,
    pub num_faults: usize,
    pub myid: Replica,
    pub pub_key_map:HashMap<Replica, PublicKey>,
    pub my_secret_key: Keypair,
    pub payload:usize,
    pub is_client_apollo_enabled: bool,

    /// Network context
    pub net_send: UnboundedSender<(Replica, Arc<ProtocolMsg>)>,
    pub cli_send: UnboundedSender<Arc<Propose>>,
    pub relay_buf: VecDeque<(Replica, Propose)>,
    pub prop_buf: VecDeque<(Replica, Propose)>,
    pub other_buf: VecDeque<(Replica, ProtocolMsg)>,

    /// Storage context
    /// Where the blockchain and transactions are stored
    pub storage: Storage,

    /// Protocol State
    pub last_leader: Replica,
    pub last_seen_block: Arc<Block>,
    pub req_ctr:u64,
    /// The blocks we are waiting for, to handle propose messages
    pub prop_waiting: HashSet<Hash>,
    pub prop_waiting_parent: HashMap<Hash, Propose>,
    /// The chain of proposals: Map of block hash to its proposal
    pub prop_chain: HashMap<Hash, Arc<Propose>>,
}

const EXTRA_SPACE:usize = 100;

impl Context {
    pub fn new(config:&Node,
        net_send: UnboundedSender<(Replica, Arc<ProtocolMsg>)>,
        cli_send: UnboundedSender<Arc<Propose>>) -> Self {
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
                    Keypair::Ed25519(kp)
                },
                crypto::Algorithm::SECP256K1 => {
                    let sk_copy = config.secret_key_bytes.clone();
                    let sk = secp256k1::SecretKey::from_bytes(sk_copy).expect("Failed to decode the secret key from the config");
                    let kp = secp256k1::Keypair::from(sk);
                    Keypair::Secp256k1(kp)
                }
                _ => panic!("Unimplemented algorithm"),
            },
            pub_key_map: HashMap::default(),
            net_send,
            cli_send,
            storage: Storage::new(EXTRA_SPACE*config.block_size),
            last_leader: 0,
            last_seen_block: Arc::new(GENESIS_BLOCK),
            is_client_apollo_enabled: false,
            payload: config.payload*config.block_size,
            req_ctr:0,
            prop_waiting:HashSet::default(),
            prop_waiting_parent: HashMap::default(),
            prop_chain: HashMap::default(),
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


    pub fn next_leader(&self) -> Replica {
        self.next_of(self.last_leader)
    }

    pub fn next_of(&self, prev: Replica) -> Replica {
        (prev+1)%self.num_nodes
    }
}
