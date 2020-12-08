use config::Client;
use libp2p::{
    identity::Keypair, 
    swarm::SwarmBuilder, 
    tcp::TokioTcpConfig, 
    core::upgrade,
    Transport,
    noise,
    yamux,
    swarm::Swarm,
};
use tokio::sync::mpsc::channel;
use types::Transaction;

use super::{Behaviour, ProtocolConfig};

fn new_dummy_tx(i:u64) -> Transaction {
    Transaction{
        data: i.to_be_bytes().to_vec(),
    }
}

pub async fn start(config:Client) {
    println!("Starting the client");
    let tcp = TokioTcpConfig::new();
    let id_keys = match config.crypto_alg{
        crypto::Algorithm::ED25519 => Keypair::generate_ed25519(),
        crypto::Algorithm::SECP256K1 => Keypair::generate_secp256k1(),
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
    let mut servers = Vec::with_capacity(config.num_nodes);
    let mut server_peers = Vec::with_capacity(config.num_nodes);
    for i in config.net_map {
        servers.push(i.1.parse().expect("failed to convert string into multiaddrs"));
    }
    println!("My servers are: {:?}", servers);
    for i in config.server_pk {
        let pk = match config.crypto_alg {
            crypto::Algorithm::ED25519 => {
                let pubk = libp2p::identity::ed25519::PublicKey::decode(&i.1).expect("failed to decode public key for a server");
                libp2p::identity::PublicKey::Ed25519(pubk)
            }
            crypto::Algorithm::SECP256K1 => {
                let pubk = libp2p::identity::secp256k1::PublicKey::decode(&i.1).expect("failed to decode public key for a server");
                libp2p::identity::PublicKey::Secp256k1(pubk)
            }
            _ => panic!("unimplemented"),
        };
        server_peers.push(
            pk.into_peer_id()
        );
    }
    println!("My server peers are: {:?}", server_peers);
    let my_behaviour = Behaviour::new(ProtocolConfig{
        blocksize: config.block_size,
        peers: servers,
    });
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

    let (send, mut recv) = channel(config.block_size*(config.num_faults+1));
    tokio::spawn(async move {
        let mut tx_ctr = 1;
        loop {
            send.send(new_dummy_tx(tx_ctr)).await.expect("failed to send new transactions on the channel");
            tx_ctr += 1;
        }
    });

    let mut pending:i32 = 50000;
    for i in server_peers {
        println!("dialing: {:?}", i);
        Swarm::dial(&mut swarm, &i ).expect("failed to dial to the server");
    }
    loop {
        tokio::select! {
            // We got an event from the server
            ev = swarm.next() => {
                match ev {
                    super::OEvent::NewBlock(b) =>
                    println!("Got block: {:?}", b),
                }
                println!("Got something from the swarm");
            },
            Some(tx) = recv.recv(), if pending > 0 => {
                pending -= 1;
                println!("Sending {} more requests", pending);
                swarm.broadcast_tx(&tx);
            }
        }
    }
}