use async_std::{io, task};
use futures::{future, prelude::*};
use libp2p::{
    floodsub::{self, Floodsub, FloodsubEvent},
    identity,
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    swarm::{NetworkBehaviourEventProcess, SwarmEvent}, NetworkBehaviour, PeerId, Swarm,
};
use std::{
    error::Error,
    task::{Context, Poll},
};

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    let transport = libp2p::development_transport(local_key).await?;

    let floodsub_topic = floodsub::Topic::new("one-to-all");

    #[derive(NetworkBehaviour)]
    struct MyBehaviour {
        floodsub: Floodsub,
        mdns: Mdns,
        #[behaviour(ignore)]
        #[allow(dead_code)]
        ignored_member: bool,
    }

    impl NetworkBehaviourEventProcess<FloodsubEvent> for MyBehaviour {
        fn inject_event(&mut self, message: FloodsubEvent) {
            if let FloodsubEvent::Message(message) = message {
                println!("Received: '{:?}' from {:?}", String::from_utf8_lossy(&message.data), message.source);

                let ota = floodsub::Topic::new("one-to-all");
                let ato = floodsub::Topic::new("all-to-one");
                let args: Vec<String> = std::env::args().collect();
                if args[1] != "main" && message.topics[0] == ota {
                    self.floodsub.publish_any(ato, message.data);
                    return
                }
            }
        }
    }

    impl NetworkBehaviourEventProcess<MdnsEvent> for MyBehaviour {
        fn inject_event(&mut self, event: MdnsEvent) {
            match event {
                MdnsEvent::Discovered(list) => {
                    for (peer, _) in list {
                        self.floodsub.add_node_to_partial_view(peer);
                    }
                }
                MdnsEvent::Expired(list) => {
                    for (peer, _) in list {
                        if !self.mdns.has_node(&peer) {
                            self.floodsub.remove_node_from_partial_view(&peer);
                        }
                    }
                }
            }
        }
    }

    let mut swarm = {
        let mdns = task::block_on(Mdns::new(MdnsConfig::default()))?;
        let mut behaviour = MyBehaviour {
            floodsub: Floodsub::new(local_peer_id),
            mdns,
            ignored_member: false,
        };
        behaviour.floodsub.subscribe(floodsub_topic.clone());

        let args: Vec<String> = std::env::args().collect();
        if args[1] == "main" {
            println!("Here!!");
            behaviour.floodsub.subscribe(floodsub::Topic::new("all-to-one"));
        }
        Swarm::new(transport, behaviour, local_peer_id)
    };

    // if let Some(to_dial) = std::env::args().nth(1) {
        // let addr: Multiaddr = to_dial.parse()?;
        // swarm.dial_addr(addr)?;
        // println!("Dialed {:?}", to_dial)
    // }

    let mut stdin = io::BufReader::new(io::stdin()).lines();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    task::block_on(future::poll_fn(move |cx: &mut Context<'_>| {
        loop {
            match stdin.try_poll_next_unpin(cx)? {
                Poll::Ready(Some(line)) => swarm
                    .behaviour_mut()
                    .floodsub
                    .publish(floodsub_topic.clone(), line.as_bytes()),
                Poll::Ready(None) => panic!("Stdin closed"),
                Poll::Pending => break,
            }
        }
        loop {
            match swarm.poll_next_unpin(cx) {
                Poll::Ready(Some(event)) => {
                    if let SwarmEvent::NewListenAddr { address, .. } = event {
                        println!("Listening on {:?}", address)
                    }
                }
                Poll::Ready(None) => return Poll::Ready(Ok(())),
                Poll::Pending => break,
            }
        }
        Poll::Pending
    }))
}
