use tokio::sync::mpsc::UnboundedSender;
use tokio_util::time::DelayQueue;
// use futures::channel::mpsc::UnboundedSender;
use types::{Block, Certificate, GENESIS_BLOCK, Height, Replica, Storage, View, synchs::ClientMsg, synchs::ProtocolMsg, synchs::Propose};
use config::Node;
use crypto::{Keypair, PublicKey, ed25519, secp256k1};
use std::collections::HashMap;
use crypto::hash::Hash;
use std::sync::Arc;

use super::phase::Phase;

pub struct Context {
    /// Networking context
    pub net_send: UnboundedSender<(Replica, Arc<ProtocolMsg>)>,
    pub cli_send: UnboundedSender<Arc<ClientMsg>>,

    /// Data context
    pub num_nodes: usize,
    pub myid: Replica,
    pub num_faults: usize,
    pub payload:usize,

    /// PKI
    pub my_secret_key: Keypair,
    pub pub_key_map:HashMap<Replica, PublicKey>,

    /// State context
    pub storage: Storage,
    /// Certificate map between block hash to certificates
    pub cert_map: HashMap<Hash, Arc<Certificate>>,
    pub height: Height,
    pub last_leader: Replica,
    pub last_seen_block: Arc<Block>,
    pub last_seen_cert: Arc<Certificate>,
    pub last_committed_block_ht: Height,
    pub vote_map: HashMap<Hash, HashMap<Replica,Certificate>>,
    pub view: View,
    pub commit_queue:DelayQueue<Arc<Propose>>,
    pub event_queue: DelayQueue<Phase>,
    pub phase: Phase,
    pub delay: u64,
}

const EXTRA_SPACE:usize = 10;

impl Context {
    pub fn new(
        config: &Node,
        net_send: UnboundedSender<(Replica, Arc<ProtocolMsg>)>,
        cli_send: UnboundedSender<Arc<ClientMsg>>,
    ) -> Self {
        let genesis_arc = Arc::new(GENESIS_BLOCK);
        let mut c = Context {
            net_send,
            num_nodes: config.num_nodes,
            cli_send,
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
            pub_key_map: HashMap::new(),
            myid: config.id,
            num_faults: config.num_faults,
            storage: Storage::new(EXTRA_SPACE*config.block_size),
            height: 0,
            last_leader: 0,
            last_seen_block: genesis_arc.clone(),
            last_committed_block_ht: 0,
            cert_map: HashMap::new(),
            view: 0,
            last_seen_cert: Arc::new(Certificate::empty_cert()),
            vote_map: HashMap::new(),
            payload:config.payload*config.block_size,
            commit_queue: tokio_util::time::DelayQueue::new(),
            phase: if config.id == 1 { Phase::Propose } else { Phase::ProposeWait },
            event_queue: DelayQueue::new(),
            delay: config.delta,
        };
        for (id,mut pk_data) in config.pk_map.clone() {
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
            c.pub_key_map.insert(id, pk);
        }

        // Initialize storage
        c.storage.add_delivered_block(genesis_arc.clone());
        c.storage.add_committed_block(genesis_arc);
        c.cert_map.insert(GENESIS_BLOCK.hash, Arc::new(Certificate::empty_cert()));
        c
    }

    /// For sync hotstuff, the next leader is the current leader
    pub fn next_leader(&self) -> Replica {
        // Apparently this is faster than % operator which is expensive in terms
        // of CPU cycles
        if self.last_leader+1 == self.num_nodes as Replica {
            0 as Replica
        } else {
            (self.last_leader + 1) as Replica
        }
    }

    /// Change Leader
    pub fn change_leader(&mut self) {
        self.last_leader = self.next_leader();
    }
}

