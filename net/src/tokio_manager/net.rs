use rustls::{Certificate, NoClientAuth, PrivateKey, ServerConfig};
use tokio::sync::mpsc::{
    UnboundedSender
};
use tokio_rustls::{TlsAcceptor, TlsConnector, rustls::{self, ClientConfig}};
use types::{
    Replica, 
    WireReady
};
use std::{
    marker::PhantomData, 
    sync::Arc
};
use fnv::FnvHashMap as HashMap;

pub struct TlsClient<I,O> 
where I:WireReady,
O:WireReady,
{
    pub(crate) peers: HashMap<Replica, UnboundedSender<Arc<O>>>,
    pub(crate) connector: TlsConnector,
    phantom: PhantomData<(I,O)>,
}

impl<I,O> TlsClient<I,O> 
where I:WireReady,
O:WireReady,
{
    /// Initialize a client manager with the network messages
    pub fn new(root_cert: Vec<u8>) -> Self {
        let mut config = ClientConfig::new();
        let cert = rustls::Certificate(root_cert);
        config.root_store.add(&cert)
            .expect("Failed to add the root certificate");

        Self{
            peers: HashMap::default(),
            phantom: PhantomData,
            connector: TlsConnector::from(Arc::new(config)),
        }
    }
}

pub struct Protocol<I,O> 
where I:WireReady,
O:WireReady,
{
    pub(crate) my_id: Replica,
    pub(crate) num_nodes: Replica,
    pub(crate) cli_acceptor: TlsAcceptor,
    phantom: PhantomData<(I,O)>,
}

impl<I,O> Protocol<I,O> 
where I:WireReady,
O:WireReady,
{
    pub fn new(my_id: Replica, num_nodes: Replica, _root_cert: Vec<u8>, my_cert: Vec<u8>, my_priv_key: Vec<u8>) -> Self {
        let mut config = ServerConfig::new(NoClientAuth::new());
        let my_cert = Certificate(my_cert);
        let mut cert_chain = Vec::new();
        cert_chain.push(my_cert);
        let my_key = PrivateKey(my_priv_key);
        config.set_single_cert(cert_chain, my_key).unwrap();
        let acceptor = TlsAcceptor::from(Arc::new(config));
        Self{
            phantom: PhantomData,
            my_id,
            num_nodes,
            cli_acceptor: acceptor,
        }
    }
}