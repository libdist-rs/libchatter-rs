// Artemis Reactors
use futures::channel::mpsc::{UnboundedSender,UnboundedReceiver};
use std::sync::Arc;
use types::artemis::{Replica, ProtocolMsg, ClientMsg, Transaction};

pub mod client;
pub mod node;

pub type NetSend = UnboundedSender<(Replica, Arc<ProtocolMsg>)>;
pub type NetRecv = UnboundedReceiver<(Replica, ProtocolMsg)>;
pub type ClientSend = UnboundedSender<Arc<ClientMsg>>;
pub type ClientRecv = UnboundedReceiver<Transaction>;
