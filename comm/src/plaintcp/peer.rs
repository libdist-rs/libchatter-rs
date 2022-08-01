use std::{net::SocketAddr, marker::PhantomData};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender, UnboundedReceiver};
use crate::{NetResult, Identifier, plaintcp::{NetMsg, TcpConnection, PeerConnectionMsg}, Message};
use super::TcpContext;

pub struct Peer<Id,SendMsg, RecvMsg> 
where
    Id: Identifier,
{
    id: Id,

    // socket address for this connection
    sock_addr: SocketAddr,

    /// The amount of time I need to wait before attempting another connection attempt
    retry_duration: std::time::Duration,

    /// The number of times to attempt re-connections before assuming that the node has died
    retries: usize,

    ctx: TcpContext,

    _x: PhantomData<(Id, SendMsg, RecvMsg)>,
}

impl<Id, SendMsg, RecvMsg> Peer<Id, SendMsg, RecvMsg> 
where
    Id: Identifier,
    SendMsg: Message,
    RecvMsg: Message,
{
    pub fn new(id: Id, ctx: TcpContext, peer_socket: SocketAddr) -> Self {
        Self { 
            id,
            sock_addr: peer_socket, 
            retry_duration: ctx.get_retry_duration(), 
            retries: ctx.get_retries(), 
            ctx,
            _x: PhantomData,
        }
    }

    pub async fn start(&mut self) -> (UnboundedSender<SendMsg>, UnboundedReceiver<NetMsg<Id,SendMsg,RecvMsg>>)
    where
        Id: Identifier,
    {
        // let (peer_in, peer_in_recv) = unbounded_channel();
        // let (peer_out_send, peer_out) = unbounded_channel::<NetMsg<Id, SendMsg, RecvMsg>>();
        let (peer_in, mut peer_in_recv) = unbounded_channel::<SendMsg>();
        let (peer_out_send, peer_out) = unbounded_channel();
        let mut retries = self.retries;
        let sock_addr = self.sock_addr;
        let retry_duration = self.ctx.get_retry_duration();
        let id = self.id.clone();

        log::debug!("Attempting connection to {}", self.sock_addr);
        tokio::spawn(async move {
            let stream = loop  {
                if retries == 0 {
                    break NetResult::Err("Retries finished, unable to connect".into());
                }
                let sock = if sock_addr.is_ipv4() {
                    tokio::net::TcpSocket::new_v4()
                } else {
                    tokio::net::TcpSocket::new_v6()
                };
                if sock.is_err() {
                    let err = sock.err().expect("unreachable");
                    break Err(err.into());
                }
                let sock = sock.expect("unreachable");
                let stream_res = sock.connect(sock_addr).await;
                if stream_res.is_err() {
                    log::warn!("Connection with {} failed. Retrying...", sock_addr);
                    retries -= 1;
                    tokio::time::sleep(retry_duration).await;
                    continue;
                }
                break stream_res.map_err(|e| e.into());
            };
            if stream.is_err() {
                peer_out_send.send(
                    NetMsg::ConnectionError(
                        sock_addr, 
                        stream.err().expect("unreachable")
                    )
                );
                return;
            }
            // We are connected, reset retries!
            let stream = stream.expect("unreachable");

            log::info!("Connected to {}", sock_addr);
            
            // Setup the connection
            if let Err(e) = stream.set_nodelay(true) {
                peer_out_send.send(
                    NetMsg::ConnectionError(
                        sock_addr, 
                        e.into()
                    )
                );
            }

            let mut conn = TcpConnection::<SendMsg, RecvMsg>::new(stream);
            let (conn_out, mut conn_in) = conn.start();
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        in_msg = conn_in.recv() => {
                            if let None = &in_msg {
                                log::warn!("Broken link to connection object. Shutting down connection");
                                break;
                            }
                            let msg = in_msg.unwrap();
                            if let PeerConnectionMsg::NewMsg(new_msg) = msg {
                                // Send the message to the outside world!
                                peer_out_send.send(NetMsg::NewMsg(id.clone(),new_msg));
                                continue;
                            } 
                            if let PeerConnectionMsg::ConnectionError(e) = msg {
                                peer_out_send.send(NetMsg::ConnectionError(sock_addr, e.into()));
                                continue;
                            }
                        }
                        new_msg = peer_in_recv.recv() => {
                            if let None = new_msg {
                                log::warn!("Communication must be shutting down; Terminating.");
                                break;
                            }
                            let new_msg = new_msg.unwrap();
                            conn_out.send(PeerConnectionMsg::SendMsg(new_msg));
                        }
                    }
                }
            });
        });
        (peer_in, peer_out)
    }
}