use std::{net::SocketAddr, marker::PhantomData};
use rand::{SeedableRng, seq::SliceRandom};
use async_trait::async_trait;
use fnv::FnvHashMap;
use tokio::{sync::mpsc::{UnboundedSender, unbounded_channel, UnboundedReceiver}, net::TcpStream};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};
use futures::SinkExt;
use rand::rngs::SmallRng;
use crate::{Message, NetSender, Identifier, EnCodec, Decodec};

#[derive(Debug)]
pub struct TcpSimpleSender<Id, SendMsg, RecvMsg>
{
    address_map: FnvHashMap<Id, SocketAddr>,
    connections: FnvHashMap<Id, UnboundedSender<SendMsg>>,
    _x: PhantomData<RecvMsg>,
    /// Small RNG just used to shuffle nodes and randomize connections (not crypto related).
    rng: SmallRng,        
}

impl<Id, SendMsg, RecvMsg> TcpSimpleSender<Id, SendMsg, RecvMsg>
where
    SendMsg: Message,
    RecvMsg: Message,
{
    pub fn new() -> Self {
        Self { 
            address_map: FnvHashMap::default(),
            connections: FnvHashMap::default(),
            _x: PhantomData,
            rng: SmallRng::from_entropy(),
        }
    }

    fn spawn_connection(address: SocketAddr) -> UnboundedSender<SendMsg> 
    {
        let (tx,rx) = unbounded_channel();
        Connection::<SendMsg, RecvMsg>::spawn(address, rx);
        tx
    }
}

struct Connection<SendMsg, RecvMsg>
{
    address: SocketAddr,
    receiver: UnboundedReceiver<SendMsg>,
    _x: PhantomData<RecvMsg>,
}

impl<SendMsg, RecvMsg> Connection<SendMsg, RecvMsg>
where
    SendMsg: Message,
    RecvMsg: Message,
{
    fn spawn(address: SocketAddr, rx: UnboundedReceiver<SendMsg>) 
    {
        tokio::spawn(async move {
            Self { address, receiver: rx, _x: PhantomData }.run().await;
        });
    }

    async fn run(&mut self)
    {
        // Connect to the address
        let (mut reader, mut writer) = match TcpStream::connect(self.address).await {
            Ok(sock) => {
                let (rd, wr) = sock.into_split();
                let reader = FramedRead::new(
                    rd, 
                    Decodec::<RecvMsg>::new()
                );
                let writer = FramedWrite::new(
                    wr, 
                    EnCodec::<SendMsg>::new()
                );
                (reader, writer)
            },
            Err(e) => {
                log::warn!("Unable to connect to peer {} with error {}", self.address, e);
                return;
            }
        };

        log::info!("Connected to {}", self.address);

        // Main sender loop 
        loop {
            tokio::select! {
                // The outside world asked me to send a message
                Some(msg) = self.receiver.recv() => {
                    if let Err(e) = writer.send(msg).await {
                        log::warn!("Unable to send any message outside the system");
                        return;
                    }
                },
                // The connection sent some response
                response = reader.next() => {
                    match response {
                        Some(Ok(_)) => {
                            // Kill the response
                        }
                        _ => {
                            log::warn!("Connection gone awry!");
                            return;
                        }
                    }
                }
            }
        }
    }
}

#[async_trait]
impl<Id, SendMsg, RecvMsg> NetSender<Id, SendMsg> for TcpSimpleSender<Id, SendMsg, RecvMsg> 
where
    Id: Identifier,
    SendMsg: Message,
    RecvMsg: Message,
{
    async fn send(&mut self, sender: Id, msg: SendMsg) {
        let addr_opt = self.address_map.get(&sender);
        if let None = addr_opt {
            log::warn!("Unknown peer {:?}", sender);
            return;
        }
        let address = addr_opt.unwrap();

        if let Some(conn) = self.connections.get(&sender) {
            // We got lucky since we have already connected to this node
            let msg_copy = msg.clone();
            if conn.send(msg_copy).is_ok() {
                return;
            }
            // We have a stale connection
            // Remove it
            self.connections
                .remove(&sender);
            // If not ok, continue with a fresh connection
        }

        let conn = Self::spawn_connection(address.clone());
        if conn.send(msg).is_ok() {
            self.connections
                .insert(sender, conn);
            return;
        }
    }

    fn blocking_send(&mut self, sender:Id, msg: SendMsg) {
        let addr_opt = self.address_map.get(&sender);
        if let None = addr_opt {
            log::warn!("Unknown peer {:?}", sender);
            return;
        }
        let address = addr_opt.unwrap();

        if let Some(conn) = self.connections.get(&sender) {
            // We got lucky since we have already connected to this node
            let msg_copy = msg.clone();
            if conn.send(msg_copy).is_ok() {
                return;
            }
            // If not ok, continue with a fresh connection
            // But first, remove some stale connections
            self.connections
                .remove(&sender);
        }

        let conn = Self::spawn_connection(address.clone());
        if conn.send(msg).is_ok() {
            self.connections
                .insert(sender, conn);
            return;
        }
    }

    async fn broadcast(&mut self, msg: SendMsg, peers: &[Id]) {
        for peer in peers {
            self.send(peer.clone(), msg.clone()).await;
        }
    }

    fn blocking_broadcast(&mut self, msg: SendMsg, peers: &[Id]) {
        for peer in peers {
            self.blocking_send(peer.clone(), msg.clone());
        }
        
    }

    async fn randcast(&mut self, msg: SendMsg, mut peers: Vec<Id>, subset_size: usize) {
        peers.shuffle(&mut self.rng);
        peers.truncate(subset_size);
        self.broadcast(msg, peers.as_ref()).await;
    }

    fn blocking_randcast(&mut self, msg: SendMsg, mut peers: Vec<Id>, subset_size: usize) {
        peers.shuffle(&mut self.rng);
        peers.truncate(subset_size);
        self.blocking_broadcast(msg, peers.as_ref());
    }
}