use tokio::sync::mpsc::UnboundedSender;
use types::{Block, Certificate, synchs::ClientMsg, GENESIS_BLOCK, Height, Replica, Storage, View, synchs::ProtocolMsg};
use config::Node;
use libp2p::{
    identity::Keypair, 
    core::PublicKey
};
use std::collections::HashMap;
use crypto::hash::Hash;
use std::sync::Arc;

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
    pub cert_map: HashMap<Hash, Certificate>, // Contains all certified blocks
    pub height: Height,
    pub last_leader: Replica,
    pub last_seen_block: Arc<Block>,
    pub last_seen_cert: Certificate,
    pub last_committed_block_ht: Height,
    pub vote_map: HashMap<Hash, Certificate>,
    pub view: View,
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
            net_send: net_send,
            num_nodes: config.num_nodes,
            cli_send: cli_send,
            my_secret_key: match config.crypto_alg {
                crypto::Algorithm::ED25519 => {
                    let mut sk_copy = config.secret_key_bytes.clone();
                    let kp = libp2p::identity::ed25519::Keypair::decode(
                        &mut sk_copy
                    ).expect("Failed to decode the secret key from the config");
                    libp2p::identity::Keypair::Ed25519(kp)
                },
                crypto::Algorithm::SECP256K1 => {
                    let sk_copy = config.secret_key_bytes.clone();
                    let sk = libp2p::identity::secp256k1::SecretKey::from_bytes(sk_copy).expect("Failed to decode the secret key from the config");
                    let kp = libp2p::identity::secp256k1::Keypair::from(sk);
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
            last_seen_cert: Certificate::empty_cert(),
            vote_map: HashMap::new(),
            payload:config.payload*config.block_size,
        };
        for (id,mut pk_data) in config.pk_map.clone() {
            let pk = match config.crypto_alg {
                crypto::Algorithm::ED25519 => {
                    let kp = libp2p::identity::ed25519::PublicKey::decode(
                        &mut pk_data
                    ).expect("Failed to decode the secret key from the config");
                    libp2p::identity::PublicKey::Ed25519(kp)
                },
                crypto::Algorithm::SECP256K1 => {
                    let sk = libp2p::identity::secp256k1::PublicKey::decode(&pk_data).expect("Failed to decode the secret key from the config");
                    libp2p::identity::PublicKey::Secp256k1(sk)
                }
                _ => panic!("Unimplemented algorithm"),
            };
            c.pub_key_map.insert(id, pk);
        }

        // Initialize storage
        c.storage.add_delivered_block(genesis_arc.clone());
        c.storage.add_committed_block(genesis_arc);
        c.cert_map.insert(GENESIS_BLOCK.hash, Certificate::empty_cert());
        c
    }

    /// For sync hotstuff, the next leader is the current leader
    pub fn next_leader(&self) -> Replica {
        self.last_leader
    }

    /// Leader of a view
    pub fn leader_of_view(&self) -> Replica {
        (self.view % self.num_nodes as u64) as u16
    }
}

