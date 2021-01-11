use std::{
    collections::HashMap, 
    sync::Arc,
    pin::Pin,
};
use log::info;
use tokio::{
    io::{
        AsyncReadExt, 
        AsyncWriteExt
    }, 
    net::{
        TcpListener, 
        TcpStream
    }, 
    sync::mpsc::{
        UnboundedReceiver, 
        UnboundedSender,
        unbounded_channel,
    },
};
use tokio_util::codec::{
    Decoder, 
    Encoder, 
};
use types::{
    Replica, 
    WireReady
};
use futures::Stream;
use tokio_stream::{StreamMap, StreamExt};
use crate::peer::Peer;

use crate::Protocol;

const ID_BYTE_SIZE:usize = std::mem::size_of::<Replica>();

type Err = std::io::Error;
type Reader = tokio::net::tcp::OwnedReadHalf;
type Writer = tokio::net::tcp::OwnedWriteHalf;

impl<I,O> Protocol<I,O>
where I:WireReady + Send + Sync + 'static + Unpin,
O:WireReady + Clone + Sync + 'static + Unpin, 
{
    pub async fn server_setup(
        &self,
        node_addr: HashMap<Replica, String>, 
        enc: impl Encoder<Arc<O>> + Clone + Send + Sync + 'static, 
        dec: impl Decoder<Item=I, Error=Err> + Clone + Send + Sync + 'static
    ) -> (UnboundedSender<(Replica, Arc<O>)>, UnboundedReceiver<(Replica, I)>)
    {
        // Task that receives connections from everyone
        let incoming_conn_task = 
        tokio::spawn(
            start_conn_all(node_addr.clone(), self.my_id)
        );
        
        // Sleep for sometime until we are sure everyone is listening
        let sleep_time = unsafe {
            config::SLEEP_TIME
        };
        tokio::time::sleep(std::time::Duration::from_secs(sleep_time)).await;
        
        // Start connecting to other nodes
        let mut writers = outgoing_conn(self.my_id, &node_addr)
            .await;
        let mut readers = incoming_conn_task
            .await
            .expect("Failed to collect all readers");
        
        info!(target:"net", "Connected to all nodes in the protocol");

        // Create a unified reader stream
        let mut unified_stream = StreamMap::new();

        // Create write end points for peers
        let mut writer_end_points = HashMap::new();
        for i in 0..self.num_nodes {
            if i == self.my_id {
                continue;
            }
            let read = readers.remove(&i).unwrap();
            let write = writers.remove(&i).unwrap();
            let enc = enc.clone();
            let dec = dec.clone();
            let peer = Peer::new(read, write, dec, enc);
            let mut recv_ch = peer.recv;
            unified_stream.insert(
                i as Replica, 
                Box::pin(async_stream::stream!{
                    while let Some(item) = recv_ch.recv().await {
                        yield(item);
                    }
                }) as Pin<Box<dyn Stream<Item=I>+Send>>
            );
            writer_end_points.insert(i as Replica, peer.send);
        }

        // Create channels so that the outside world can communicate with the
        // network
        let (in_send, in_recv) = unbounded_channel::<(Replica, I)>();
        let (out_send, out_recv) = unbounded_channel();

        // Start the event loop that processes network messages
        tokio::spawn(
            protocol_event_loop(
                self.num_nodes, 
                in_send, 
                out_recv, 
                unified_stream, 
                writer_end_points
            )
        );
    
        (out_send, in_recv)
    }

    pub async fn client_setup(
        listen: String,
        enc: impl Encoder<Arc<O>> + Clone + Send + Sync + 'static, 
        dec: impl Decoder<Item=I, Error=Err> + Clone + Send + Sync + 'static
    ) -> (UnboundedSender<Arc<O>>, UnboundedReceiver<I>) 
    {
        let (cli_in_send, cli_in_recv) = unbounded_channel();
        let (cli_out_send, cli_out_recv) = unbounded_channel();
        
        let cli_manager_stream = cli_manager(listen).await;
        tokio::spawn(
            client_event_loop(enc, dec, cli_out_recv, cli_in_send, cli_manager_stream)
        );
        (cli_out_send, cli_in_recv)
    }
}

async fn start_conn_all(
    node_addr: HashMap<Replica, String>,
    my_id: Replica,
) -> HashMap<Replica, Reader>
{
    // Start listening to incoming connections
    let listener = TcpListener::bind(
        &node_addr[&my_id]
    )   .await
    .expect("Failed to listen to protocol messages");
    let node_addr = node_addr.clone();
    let n = node_addr.len();
    let mut readers = HashMap::with_capacity(n);
    for _i in 1..n {
        // Connect to a new node
        let (mut conn, from) = listener.accept()
            .await
            .expect("Failed to listen to incoming connections");
        
        // Set nodelay
        conn.set_nodelay(true).expect("Failed to set nodelay");
        
        info!(target:"net", "New incoming connection from {}", from);
        
        // Get the ID of the connector
        let mut id_buf = [0 as u8; ID_BYTE_SIZE];
        conn
            .read_exact(&mut id_buf)
            .await
            .expect("Failed to read ID bytes");
        let id = Replica::from_be_bytes(id_buf);
        
        // Split the connection and drop the writing part
        let (read, _write) = conn.into_split();

        // Add this reader
        readers.insert(
            id, 
            read
        );
    }
    readers
}

async fn outgoing_conn(
    my_id: Replica, 
    node_addr: &HashMap<Replica, String>,
)  -> HashMap<Replica, Writer>
{
    // Create bytes of the ID
    let id_buf = my_id.to_be_bytes();
    
    let mut writers = HashMap::new();
    for (id, addr) in node_addr {
        let id = *id as Replica;
        
        // Do not connect to self
        if id == my_id {
            continue;
        }
        
        // Connect to the node
        let conn = TcpStream::connect(addr)
        .await
        .expect("Failed to connect to a protocol node");
        // Enbale high speed connection
        conn
        .set_nodelay(true)
        .expect("Failed to enable nodelay on the connection");
        
        // Split the socket into seperate RW components
        let (_read, mut write) = conn.into_split();
        
        // Send id of self on the connection
        write
            .write_all(&id_buf)
            .await
            .expect("Failed to send identification to a protocol node");
        
        writers.insert(id, write);
    }
    writers
}

async fn protocol_event_loop<I,O>(
    num_nodes: Replica, 
    in_send: UnboundedSender<(Replica, I)>,
    mut out_recv: UnboundedReceiver<(Replica, Arc<O>)>,
    mut reading_net: impl Stream<Item=(Replica, I)>+Unpin,
    writers: HashMap<Replica, UnboundedSender<Arc<O>>>
) where I: WireReady
{
    loop {
        tokio::select!{
            opt_in = reading_net.next() => {
                if let None = opt_in {
                    log::error!(target:"manager", 
                        "Failed to read a protocol message from a peer");
                    std::process::exit(0);
                }
                let (id, msg) = opt_in.unwrap();
                if let Err(e) = in_send.send((id, msg.init())) {
                    log::error!(target:"manager", 
                        "Failed to send a protocol message outside the network, with error {}", e);
                    std::process::exit(0);
                }
            },
            opt_out = out_recv.recv() => {
                if let None = opt_out {
                    log::error!(target:"manager", 
                        "Failed to read a protocol message to send outside the network");
                    std::process::exit(0);
                }
                let (to, msg) = opt_out.unwrap();
                if to < num_nodes {
                    if let Err(_e) = writers[&to].send(msg) {
                        log::error!(target:"manager",
                            "Failed to send msg to peer {}", to);
                        std::process::exit(0);
                    }
                } else {
                    for (id, writer) in &writers {
                        if let Err(e) = writer.send(msg.clone()) {
                            log::error!(target:"net", "Failed to send msg to peer {} with error {}", id, e);
                            std::process::exit(0);
                            // TODO Handle disconnection from peer
                        }
                    }
                }
            },
        }
    }
}

async fn cli_manager(addr: String) -> UnboundedReceiver<TcpStream> {
    // Wait for new connections
    let cli_sock = TcpListener::bind(addr)
        .await
        .expect("Failed to listen to client connections");

    // Create channels to let the world know that we have a new client
    // connection
    let (conn_ch_send, conn_ch_recv) = unbounded_channel();
    tokio::spawn(async move {
        loop {
            let conn_opt = cli_sock.accept().await;
            let conn = match conn_opt {
                Err(e) => {
                    log::error!(target:"manager", "Failed to accept a connection from the client with error {}", e);
                    continue;
                },
                Ok((conn, from)) => {
                    if let Err(e) = conn.set_nodelay(true) {
                        log::error!(target:"manager", "Failed to set high speed socket for client: {} with error {}", from, e);
                        continue;
                    }
                    conn
                }
            };
            if let Err(e) = conn_ch_send.send(conn) {
                log::error!(target:"manager", "Failed to send out new client connection: {}", e);
                std::process::exit(0);
            }
        }
    });
    conn_ch_recv
}

async fn client_event_loop<I,O>(
    enc: impl Encoder<Arc<O>> + Clone + Send + Sync + 'static, 
    dec: impl Decoder<Item=I, Error=Err> + Clone + Send + Sync + 'static,
    mut send_out_ch: UnboundedReceiver<Arc<O>>,
    new_in_ch: UnboundedSender<I>,
    mut new_conn_ch: UnboundedReceiver<TcpStream>
) where I:WireReady + Sync + Unpin + 'static,
O: WireReady + Clone+Unpin+Sync + 'static,
{
    let mut read_stream:StreamMap<usize, Pin<Box<dyn Stream<Item=I>+Send>>> = StreamMap::new();
    let mut client_id = 0 as usize;
    let mut writers = HashMap::new();
    let mut to_remove = Vec::new();
    loop {
        tokio::select! {
            // We received something from the client
            in_opt = read_stream.next(), if read_stream.len() > 0 => {
                if let None = in_opt {
                    log::warn!(target:"manager", "Read stream closed");
                    std::process::exit(0);
                }
                let (_id, msg) = in_opt.unwrap();
                let msg = msg.init();
                if let Err(e) = new_in_ch.send(msg) {
                    log::error!(target:"manager", "Failed to send an incoming client message outside, with error {}", e);
                    std::process::exit(0);
                }
            },
            // We have a new client
            conn_opt = new_conn_ch.recv() => {
                if let None = conn_opt {
                    log::warn!(target:"manager", "New connection channel closed");
                    std::process::exit(0);
                }
                let conn = conn_opt.unwrap();
                let (read, write) = conn.into_split();
                let client_peer = Peer::new(read, write, dec.clone(), enc.clone());
                let mut client_recv = client_peer.recv;
                read_stream.insert(client_id, Box::pin(async_stream::stream! {
                    while let Some(item) = client_recv.recv().await {
                        yield(item);
                    }
                }) as std::pin::Pin<Box<dyn futures_util::stream::Stream<Item=I> +Send>>);
                writers.insert(client_id, client_peer.send);

                client_id = client_id + 1;
            },
            // We have a new message to send to the clients
            out_opt = send_out_ch.recv() => {
                if let None = out_opt {
                    log::warn!(target:"manager", "Send out channel closed");
                    std::process::exit(0);
                }
                let msg = out_opt.unwrap();
                for (id, writer) in &writers {
                    if let Err(e) = writer.send(msg.clone()) {
                        log::info!(target:"net","Disconnected from client with error: {}", e);
                        to_remove.push(*id);
                    }
                }
            }
        }
        // Remove disconnected clients
        for id in &to_remove {
            writers.remove(id);
        }
        to_remove.clear();
    }
}