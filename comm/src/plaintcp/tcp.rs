// use std::{marker::PhantomData, net::SocketAddr};
// use tokio::{select, net::TcpListener, sync::mpsc::unbounded_channel};

// use crate::{Message, NetResult, NetSender, NetReceiver, plaintcp::Peer, Identifier};
// use super::{TcpConfig, TcpContext, NetMsg};

// #[derive(Debug)]
// pub struct TcpCommunication<SendMsg, RecvMsg, Id> 
// where
//     Id: Identifier,
// {
//     config: TcpConfig<Id>,
//     ctx: TcpContext,
    
//     /// Phantom data used to hold these markers
//     _x: PhantomData<SendMsg>,
//     _y: PhantomData<RecvMsg>,
// }

// impl<SendMsg, RecvMsg, Id> TcpCommunication<SendMsg, RecvMsg, Id> 
// where 
//     SendMsg: Message,
//     RecvMsg: Message,
//     Id: Identifier,
// {
//     pub fn init(
//         config: TcpConfig<Id>, 
//         ctx: TcpContext, 
//     ) -> Self 
//     {
//         Self {
//             config,
//             ctx,
//             _x: PhantomData,
//             _y: PhantomData,
//         }
//     }

//     async fn start_server(my_id: Id, my_addr: SocketAddr)
//     {
//         let server_sock = TcpListener::bind(my_addr)
//                                             .await
//                                             .expect("Failed to bind to server socket");
//         loop {
//             select! {
//                 sock = server_sock.accept() => {
//                     log::info!("Got a new connection from {:?}", sock.unwrap());
//                     // 
//                 }
//             }
//         }

//     }

//     pub fn start(&mut self) -> 
//         NetResult<(Box<dyn NetSender<Id, SendMsg>>, Box<dyn NetReceiver<NetMsg<Id, SendMsg, RecvMsg>>>)> 
//     {
//         let (in_send, in_recv) = unbounded_channel();
//         let (out_send, out_recv) = unbounded_channel();

//         log::info!("Opening local socket at: {}", self.config.get_my_addr());
//         let config = self.config.clone();
//         let ctx = self.ctx.clone();
//         let handle = ctx.get_handle().clone();
//         let my_id = config.get_id().clone();
//         let my_addr = config.get_my_addr();
//         tokio::spawn(async move {
//             for (peer_id, peer_addr) in config.get_peers() {
//                 // The reasoning is the following:
//                 //  Nodes with higher ids will connect to me through the server loop
//                 //  Nodes with lower ids, I will connect to, myself
//                 if *peer_id == my_id {
//                     log::debug!("Skipping my ID");
//                     continue;
//                 }
//                 // Start the connection loop
//                 let conn_addr = *peer_addr;
//                 let new_ctx = ctx.clone();
//                 let mut peer = Peer::new(*peer_id, new_ctx, conn_addr);
//                 let (peer_send, peer_recv) = peer.start().await;
//             }
//         });
//         tokio::spawn(async move {
//             Self::start_server(my_id, my_addr.clone());
//         });
//         // todo!("Implement Tcp-comm start");
//         Ok(Box::new(send_in), Box::new(recv_from_conn))
//     }
// }



// //     fn send(&self, 
// //         sender: &<Self::Config as Config>::PeerId, 
// //         msg: Self::SendMsg
// //     ) {
        
// //     }

// //     fn concurrent_send(&self, sender:usize, msg: Self::SendMsg) {
        
// //     }

// //     fn concurrent_broadcast(&self, msg: Self::SendMsg) {
        
// //     }

// //     fn broadcast(&self, msg: Self::SendMsg) {
        
// //     }
// // }

// // impl<'a, SendMsg, RecvMsg> TcpCommunication<'a, SendMsg, RecvMsg> 
// // {
// //     async fn server_setup(listener: tokio::net::TcpListener, connections: RwLock<String>) {
// //         let handles = (0..100).map(|i| {
// //             tokio::spawn(async {

// //             })
// //         }).collect();
// //     }

// //     fn setup(&mut self) -> Result<(), String> {
// //         // Step : Open a server socket
// //         let server_sock = tokio::net::TcpListener::bind(
// //             format!("0.0.0.0:{}", self.config.my_port)
// //         );
// //         self.ctx.rt.block_on(async {
// //            let s = server_sock.await;
// //            if let Err(e) = s {
// //                return Err(e.to_string());
// //            }
// //            Ok(
// //                 TcpCommunication::<SendMsg, RecvMsg>::server_setup(
// //                    s.unwrap(),
// //                    self.ctx.connections.clone()
// //                 ).await
// //            )
// //         })
// //         // Step : Start listening thread
// //         // Step : Start Connection Threads for the others
// //         // Step : Wait for sufficient connections
// //     }
// // }

// // // impl TcpCommunication {
// // //     async fn incoming_connections(
// // //         num_conn: usize, server_sock: tokio::net::TcpListener
// // //     ) -> (HashMap<usize, TcpStream>, TcpListener)
// // //     {
// // //         let mut connections_incoming = HashMap::default();
// // //         while connections_incoming.len() < num_conn {
// // //             let result = server_sock.accept().await;
// // //             if let Err(e) = result {
// // //                 error!("Server socker error: {}", e);
// // //                 continue;
// // //             }
// // //             let (mut stream, sock_addr) = result.unwrap();
// // //             stream.set_nodelay(TcpConfig::NO_DELAY);
// // //             debug!("New incoming connection from {}", sock_addr);
            
// // //             let result = stream.read_u64_le().await;
// // //             if let Err(e) = result {
// // //                 error!("Error reading Id from incoming connection: {}", e);
// // //                 continue;
// // //             }
// // //             let other_id = result.unwrap() as usize;
// // //             connections_incoming.insert(other_id, stream);
// // //         }
// // //         (connections_incoming, server_sock)
// // //     }
// // //     async fn setup(config: TcpConfig) -> Self {
// // //         // First open a listening socket
// // //         let server_sock = tokio::net::TcpListener::bind(
// // //                 config.get_listen_addr()
// // //             ).await.expect(
// // //                 format!("Failed to bind to address {}", config.get_listen_addr()).as_str()
// // //             );
// // //         // Each node connects to nodes whose oId is less than Id, and waits for incoming connections from nodes whose oId is greater than Id
// // //         let incoming_connections = config.get_num_nodes() - config.get_id() - 1;
// // //         let connect_job = tokio::spawn(
// // //             TcpCommunication::incoming_connections(
// // //                 incoming_connections, server_sock
// // //             )
// // //         );
// // //         let mut outgoing_connections = HashMap::default();
// // //         for other_id in 0..config.get_id() {
// // //             let sock_addr = config.get_node_addr(&other_id);
// // //             let total_attempts = TcpConfig::NUM_RETRIES;
// // //             let mut attempt = 0;
// // //             while attempt < total_attempts {
// // //                 let sock_addr = sock_addr.as_str();
// // //                 let result = TcpStream::connect(sock_addr)
// // //                     .await;
// // //                 if let Err(e) = result {
// // //                     warn!("Failed to connect to node {} with error {}, Retrying attempt #{}", other_id, e, attempt);
// // //                     attempt += 1;
// // //                     // Sleep before trying again
// // //                     tokio::time::sleep(Duration::from_millis(TcpConfig::CONNECTION_SLEEP_TIME)).await;
// // //                     continue;
// // //                 }
// // //                 let mut conn = result.unwrap();
// // //                 let result = conn.set_nodelay(TcpConfig::NO_DELAY);
// // //                 if let Err(e) = result {
// // //                     error!("Failed to set connection no delay parameter with error {} for node {}", e, other_id);
// // //                     warn!("Falling back to slow mechanism");
// // //                 }
// // //                 // We received a u64_le when receiving connections, we also need to send one here, so the other end can identify who the source is.
// // //                 // TODO: Prove that I am the correct Id
// // //                 let result = conn.write_u64_le(config.get_id() as u64).await;
// // //                 if let Err(e) = result {
// // //                     error!("Failed to set ")
// // //                 }
// // //                 outgoing_connections.insert(other_id, conn);
// // //                 break;
// // //             }
// // //         }
// // //         let (mut incoming, listener) = connect_job
// // //             .await.expect("Failed to receive incoming connections");
// // //         incoming.extend(outgoing_connections);

// // //         // TODO: Setup channels

// // //         Self{
// // //             config,
// // //             listener,
// // //         }
// // //     }
// // // }

// // // #[async_trait]
// // // impl Communication for TcpCommunication {
// // //     type Config = TcpConfig;

// // //     async fn init(config: Self::Config) -> Self {
// // //         TcpCommunication::setup(config).await
// // //     }

// // //     fn send(sender: usize, msg: impl crate::Message) {
// // //         unimplemented!();
// // //     }

// // //     fn concurrent_send(sender:usize, msg: impl crate::Message) {
// // //         unimplemented!();
// // //     }

// // //     fn broadcast(msg: impl crate::Message) {
// // //         unimplemented!();
// // //     }

// // //     fn concurrent_broadcast(msg: impl crate::Message) {
// // //         unimplemented!();
// // //     }
// // // }