use std::collections::HashMap;

use libp2p::{identity::Keypair, core::PublicKey};
use tokio::sync::mpsc::{Sender};
use types::{Block, GENESIS_BLOCK, Height, ProtocolMsg, Replica, Storage};
use config::Node;

pub struct Context {
    pub num_nodes: u16,
    pub num_faults: u16,
    pub myid: Replica,
    pub pub_key_map:HashMap<Replica, PublicKey>,
    pub my_secret_key: Keypair,
    pub net_send: Sender<(Replica, ProtocolMsg)>,
    pub cli_send: Sender<Block>,
    pub is_client_apollo_enabled: bool,

    pub storage: Storage,
    pub height: Height,
    pub last_leader: Replica,
    pub last_seen_block: Block,
    pub last_committed_block_ht: Height,
    pub payload:usize,
}

const EXTRA_SPACE:usize = 100;

impl Context {
    pub fn new(config:&Node,
        net_send: Sender<(Replica, ProtocolMsg)>,
        cli_send: Sender<Block>) -> Self {
        let mut c = Context{
            num_nodes: config.num_nodes as u16,
            num_faults: config.num_faults as u16,
            myid: config.id,
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
            pub_key_map: HashMap::with_capacity(config.num_nodes),
            net_send: net_send,
            cli_send: cli_send,
            storage: Storage::new(EXTRA_SPACE*config.block_size),
            /// The height and next leader are both 1 because the genesis block
            /// is of height 0 and its author is replica 0
            height: 0,
            last_leader: 0,
            last_seen_block: GENESIS_BLOCK,
            last_committed_block_ht: 0,
            is_client_apollo_enabled: false,
            payload: config.payload,
        };
        for (id,mut pk_data) in config.pk_map.clone() {
            if id == c.myid {
                continue;
            }
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
        c
    }


    pub fn next_leader(&self) -> Replica {
        self.next_of(self.last_leader)
    }

    pub fn next_of(&self, prev: Replica) -> Replica {
        (prev+1)%self.num_nodes
    }
}
