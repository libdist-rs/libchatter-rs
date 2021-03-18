use tokio::{
    net::TcpStream, 
    sync::mpsc::{
        UnboundedReceiver, 
        UnboundedSender, 
        unbounded_channel
    }
};
// use futures::channel::mpsc::{
//     UnboundedReceiver,
//     UnboundedSender,
//     unbounded as unbounded_channel,
// };
// use futures::SinkExt;
use tokio_rustls::webpki::DNSNameRef;
use tokio_stream::StreamMap;
use tokio_util::codec::{
    Decoder, 
    Encoder
};
use std::{
    io::Error, 
    sync::Arc, 
};
use fnv::FnvHashMap as HashMap;

use types::{
    Replica, 
    WireReady
};
use super::{
    TlsClient, 
};
use super::peer::Peer;
use tokio_stream::StreamExt;

impl<I,O> TlsClient<I,O>
where I:WireReady + Send + Sync + 'static + Unpin,
O:WireReady + Clone + Sync + 'static + Unpin, 
{
    pub async fn setup(
        &mut self,
        node_addr: HashMap<Replica, String>, 
        enc: impl Encoder<Arc<O>> + Send + Clone + 'static, 
        dec: impl Decoder<Item=I, Error=Error> + Clone + Send + 'static
    ) -> (UnboundedSender<(Replica, Arc<O>)>, UnboundedReceiver<(Replica, I)>)
    {
        let n = node_addr.len();
        let mut read_stream = StreamMap::with_capacity(n);

        for (i, addr) in node_addr {
            let peer = self.add_new_peer(addr, enc.clone(), dec.clone()).await;
            
            // Add the receive part of the peer to the read stream
            let mut recv = peer.recv;
            
            // Create a read stream from a receiver
            let recv = Box::pin(async_stream::stream!{
                while let Some(item) = recv.recv().await {
                    yield item;
                }
            }) as std::pin::Pin<Box<dyn futures_util::stream::Stream<Item=I> +Send>>;
            
            // Add it to our maps
            read_stream.insert(i, recv);
            self.peers.insert(i, peer.send);
        }

        self.start_event_loop(read_stream)
    }

    pub(crate) async fn add_new_peer(
        &self,
        addr: String, 
        enc: impl Encoder<Arc<O>> + Send + 'static, 
        dec: impl Decoder<Item=I, Error=Error> + Clone + Send + 'static
    ) -> Peer<I,O> {
        // Connect to the server
        let conn = TcpStream::connect(addr)
            .await
            .expect("Failed to connect to a server");

        // Speed up the connection
        conn.set_nodelay(true)
            .expect("Failed to speed up the socket");

        let domain = DNSNameRef::try_from_ascii_str("nodes.com").unwrap();
        let conn = self.connector.connect(domain, conn).await.unwrap();

        // Split the socket into read and write components
        let (read, write) = tokio::io::split(conn);

        // Return the peer
        Peer::new(read, write, dec, enc)
    }

    pub(crate) fn start_event_loop(
        &mut self,
        mut stream: impl tokio_stream::Stream<Item=(Replica, I)> + Unpin + Send + 'static
    ) -> (UnboundedSender<(Replica, Arc<O>)>, UnboundedReceiver<(Replica, I)>) {
        let (in_send, mut in_recv) 
            = unbounded_channel::<(Replica, Arc<O>)>();
        let (out_send, out_recv) 
            = unbounded_channel();
        // I hope no new peers will be added later
        let n = self.peers.len();
        let peers = self.peers.clone();
        // log::trace!("Using peers: {:?}", peers);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    to_send_opt = in_recv.recv() => {
                        if let None = to_send_opt {
                            log::warn!("Network receiver closed");
                            std::process::exit(0);
                            // Must have occurred because someone dropped the
                            // receiver, indicating a shutdown
                        }
                        let (to, msg) = to_send_opt.unwrap();
                        if (to as usize) < n {
                            let opt = peers[&to].send(msg);
                            if let Err(e) = opt {
                                panic!("failed to send a message out to peer {} with error {}", to, e);
                            }
                        } else {
                            for (_i, sender) in &peers {
                                let opt = sender.send(msg.clone());
                                if let Err(e) = opt {
                                    panic!("failed to send a message out to peer {} with error {}", to, e);
                                }
                            }
                        }
                    },
                    recvd_msg_opt = stream.next() => {
                        if let None = recvd_msg_opt {
                            log::warn!("Unified stream closed");
                            std::process::exit(0);
                            // TODO: Handle client disconnection from the server
                        }
                        let recvd_msg = recvd_msg_opt.unwrap();
                        let out_opt = out_send.send(recvd_msg);
                        if let Err(e) = out_opt {
                            panic!("Failed a received message outside: {}", e);
                        }
                    },
                }
            }
        });
        (in_send, out_recv)
    }
}

