use std::{collections::{
        HashSet, 
        VecDeque
    }, task::{
        Context,
        Poll,
    }};
use libp2p::{
    swarm::{
        NetworkBehaviour,
        NetworkBehaviourAction,
        PollParameters,
        ProtocolsHandler,
    },
    core::{
        Multiaddr,
        connection::ConnectionId,
    },
    PeerId,
};
use types::{Transaction};

use super::{
    IEvent,
    OEvent, 
    Handler, 
    protocolconfig::ProtocolConfig as Config,
};

pub struct Behaviour {
    /// Configuration details
    config: Config,
    // Pending transactions
    pending_tx: VecDeque<Transaction>,
    // Incoming events
    in_events: VecDeque<NetworkBehaviourAction<IEvent, OEvent>>,
    // connected servers
    servers: HashSet<PeerId>,
    // have we connected to any server
    // ensure that we are connected before shipping transactions
    conn_state: ConnState,
}

enum ConnState {
    Full,
    PartConn(u8),
    Unconnected,
}

impl Behaviour {
    pub fn new(c: Config) -> Self {
        Behaviour{
            config:c,
            pending_tx: VecDeque::with_capacity(50_000),
            in_events: VecDeque::with_capacity(150_000),
            servers: HashSet::new(),
            conn_state: ConnState::Unconnected,
        }
    }
}

// Implement network behaviour
impl NetworkBehaviour for Behaviour {
    type ProtocolsHandler = Handler;
    type OutEvent = OEvent;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        println!("Creating a new handler");
        return Handler::new();
    }

    // Return the peers
    fn addresses_of_peer(&mut self, _p: &PeerId) -> Vec<Multiaddr> {
        return self.config.peers.clone();
    }

    // run as soon as a new peer joins
    fn inject_connected(&mut self, p: &PeerId) {
        println!("Connected to a new server");
        self.servers.insert(p.clone());
        self.conn_state = match self.conn_state {
            ConnState::Unconnected => ConnState::PartConn(1),
            ConnState::PartConn(i) => {
                if i+1 == (self.config.peers.len() as u8) {
                    println!("Connected to all the servers");
                    ConnState::Full
                } else {
                    ConnState::PartConn(i+1)
                }
            }
            ConnState::Full => ConnState::Full,
        }
    }

    // we lost a server connection
    fn inject_disconnected(&mut self, peer_id: &PeerId) {
        println!("We lost a server");
        self.servers.remove(peer_id);
    }

    // Inject events
    fn inject_event(
        &mut self, 
        _peer_id: PeerId, 
        _connection: ConnectionId,
        event: <Self::ProtocolsHandler as ProtocolsHandler>::OutEvent
    ) {
        self.in_events.push_back(
            NetworkBehaviourAction::GenerateEvent(event)
        );
    }

    // This poll is for outsiders to get events out of the behaviour
    fn poll(
        &mut self,
        _cx: &mut Context<'_>,
        _params: &mut impl PollParameters,
    ) -> Poll<
        NetworkBehaviourAction<
            <Self::ProtocolsHandler as ProtocolsHandler>::InEvent,
            Self::OutEvent,
        >,
    > {
        if let ConnState::Unconnected = self.conn_state {
            return Poll::Pending;
        }
        if let ConnState::PartConn(_i) = self.conn_state {
            return Poll::Pending;
        }
        // We are fully connected to all the nodes
        if let Some(e) = self.in_events.pop_front() {
            // println!("Processing event {:?}", e);
            return Poll::Ready(e);
        }
        // We are ready, create send events for all pending transactions
        if let Some(tx) = self.pending_tx.pop_front() {
            for p in &self.servers {
                self.in_events.push_back(
                    NetworkBehaviourAction::NotifyHandler{
                    event: IEvent::NewTx(tx.clone()),
                    handler: libp2p::swarm::NotifyHandler::All,
                    peer_id: p.clone(),
                });
            }
            // self.in_events.push_back();
        }
        Poll::Pending
    }
}

impl Behaviour {
    pub fn broadcast_tx(&mut self, t: &Transaction) {
        self.pending_tx.push_back(t.clone());
    }
}