use std::{collections::VecDeque, task::Context, collections::HashSet, task::Poll};

use libp2p::{Multiaddr, PeerId, core::connection::ConnectionId, swarm::{IntoProtocolsHandler, NetworkBehaviour, NetworkBehaviourAction, PollParameters, ProtocolsHandler}};
use types::{Block, Transaction};
use super::{
    Handler,
    IEvent, OEvent,
};

pub struct Behaviour {
    // Configuration details

    // Incoming events from the outside world
    // In our case it is new blocks
    in_events: VecDeque<NetworkBehaviourAction<IEvent, OEvent>>,

    // events to report outside when polled
    events: VecDeque<Transaction>,

    // all the clients we know of
    clients: HashSet<PeerId>,
}

impl Behaviour {
    pub fn new() -> Self {
        Behaviour{
            in_events:VecDeque::new(),
            events: VecDeque::new(),
            clients: HashSet::new(),
        }
    }
}

// Implement network behaviour to talk to clients
impl NetworkBehaviour for Behaviour {
    type ProtocolsHandler = Handler;
    type OutEvent = OEvent;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        println!("Creating a new handler");
        return Handler::new();
    }

    fn addresses_of_peer(&mut self, _peer_id: &PeerId) -> Vec<Multiaddr> {
        return Vec::new();
    }

    fn inject_connected(&mut self, peer_id: &PeerId) {
        println!("Connected to a new client");
        self.clients.insert(peer_id.clone());
    }

    // Inject events from the handler into the behaviour, so we can tell the
    // outside world
    fn inject_event(
        &mut self,
        _peer_id: PeerId, 
        _connection: ConnectionId, 
        event: <<Self::ProtocolsHandler as IntoProtocolsHandler>::Handler as ProtocolsHandler>::OutEvent
    ) {
        match event {
            OEvent::NewTx(tx) => {
                println!("Got a new transaction");
                self.events.push_back(tx);
            }
            // _ => println!("Unimplemented out event"),
        }
    }

    fn inject_disconnected(&mut self, peer_id: &PeerId) {
        println!("We lost a client");
        self.clients.remove(peer_id);
    }

    fn poll(
        &mut self, 
        _cx: &mut Context<'_>, 
        _params: &mut impl PollParameters
    ) -> Poll<
        NetworkBehaviourAction<
            <<Self::ProtocolsHandler as IntoProtocolsHandler>::Handler as ProtocolsHandler>::InEvent, 
            Self::OutEvent>
        > 
    {
        if let Some(tx) = self.events.pop_front() {
            return Poll::Ready(
                NetworkBehaviourAction::GenerateEvent(OEvent::NewTx(tx))
            );
        }
        if let Some(e) = self.in_events.pop_front() {
            return Poll::Ready(e);
        }
        Poll::Pending
    }
}

impl Behaviour {
    pub fn broadcast_block(&mut self, b: Block) {
        for p in &self.clients {
            self.in_events.push_back(
                NetworkBehaviourAction::NotifyHandler{
                    event: IEvent::NewBlock(b.clone()),
                    handler: libp2p::swarm::NotifyHandler::All,
                    peer_id: p.clone(),
                }
            );
        }
    }
}