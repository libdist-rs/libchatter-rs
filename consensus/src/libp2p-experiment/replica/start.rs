use config::{Node};
use libp2p::{
    identity::Keypair,
    Multiaddr, 
    swarm::SwarmBuilder,
    tcp::TokioTcpConfig,
    Transport,
};
use tokio::sync::mpsc::channel;
// use mpsc::{Sender};
// use tokio::net::{TcpListener, TcpStream};
// use tokio::io::{self,AsyncReadExt, AsyncWriteExt};
use std::{error::Error};
use libp2p::{
    noise,
    yamux,
    core::upgrade,
    swarm::Swarm,
};
use types::LIBP2P_MULTIADDR_FMT;

use super::client::Behaviour;
use super::client::OEvent;

pub async fn start(mut config: Node) -> Result<(), Box<dyn Error>> {
    let tcp = TokioTcpConfig::new();
    let id_keys:Keypair = match config.crypto_alg{
        crypto::Algorithm::ED25519 => {
            let kp = libp2p::identity::ed25519::Keypair::decode(
                &mut config.secret_key_bytes
            ).expect("Failed to decode the secret key from the config");
            libp2p::identity::Keypair::Ed25519(kp)
        },
        crypto::Algorithm::SECP256K1 => {
            let sk = libp2p::identity::secp256k1::SecretKey::from_bytes(config.secret_key_bytes).expect("Failed to decode the secret key from the config");
            let kp = libp2p::identity::secp256k1::Keypair::from(sk);
            Keypair::Secp256k1(kp)
        }
        _ => panic!("Unimplemented algorithm"),
    }; 
    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(&id_keys)
        .unwrap();
    let noise = noise::NoiseConfig::xx(noise_keys)
        .into_authenticated();
    let yamux = yamux::YamuxConfig::default();
    let transport = tcp
        .upgrade(upgrade::Version::V1)
        .authenticate(noise)
        .multiplex(yamux)
        .boxed();
    let cliaddr:Multiaddr = format!("{}/{}", LIBP2P_MULTIADDR_FMT, config.client_port)
                .parse()
                .expect("Failed to convert client address into multiaddr");
    let my_behaviour = Behaviour::new();
    let mut swarm = SwarmBuilder::new(
        transport, 
        my_behaviour, 
        id_keys.public().into_peer_id()
    ).executor(
        Box::new(
            |fut| { 
                tokio::spawn(fut); 
            }
        )
    ).build();
    
//     // create a channel to process client events
    let (cli_send, mut cli_recv) = 
        channel(config.block_size);
    let blocksize = config.block_size;
    tokio::spawn(async move {
        let mut txs = Vec::with_capacity(blocksize as usize);
        loop {
            if let Some(tx) = cli_recv.recv().await {
                println!("Got a new transaction. Have {} transactions now.", txs.len());
                txs.push(tx);
            }
            if txs.len() == blocksize {
                println!("We have a block");
            }
        }
    });

    let x = Swarm::listen_on(&mut swarm, cliaddr.clone())
        .expect("Failed to open port to listen to clients");
    println!("Listening on:{:?}", x);
    let e = loop {
        let ev = swarm.next().await;
        match ev {
            OEvent::NewTx(tx) => {
                if let Err(e) = cli_send.send(tx).await {
                    break e;
                }
            },
        }
    };
    Err(Box::new(e))
}