use std::{marker::PhantomData, net::SocketAddr};
use tokio::{net::{TcpListener, TcpStream}, net::tcp::OwnedWriteHalf};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::{Message, Handler, Decodec, EnCodec, Writer};

struct TcpReceiver<SendMsg, RecvMsg, MsgHandler>
where
    SendMsg: Message,
    RecvMsg: Message,
    MsgHandler: Handler<SendMsg, RecvMsg>,
{
    _x: PhantomData<(SendMsg, RecvMsg)>,
    address: SocketAddr,
    handler: MsgHandler,
}

impl<SendMsg, RecvMsg, MsgHandler> TcpReceiver<SendMsg, RecvMsg, MsgHandler>
where 
    SendMsg: Message,
    RecvMsg: Message,
    MsgHandler: Handler<SendMsg, RecvMsg>,
{
    pub fn spawn(address: SocketAddr, handler: MsgHandler) {
        tokio::spawn(async move {
            Self { _x: PhantomData, address, handler }.run().await;
        });
    }
    
    fn into_sink(writer: FramedWrite<OwnedWriteHalf, EnCodec<SendMsg>>) -> Writer<SendMsg> 
    {
        Box::new(writer) as Writer<SendMsg>
    }

    fn spawn_runner(socket: TcpStream, peer_address: SocketAddr, handler: MsgHandler) 
    {
        tokio::spawn(async move {
            let (rd, wr) = socket.into_split();
            let mut reader = FramedRead::new(
                rd, 
                Decodec::<RecvMsg>::new()
            );
            let writer = FramedWrite::new(
                wr, 
                EnCodec::<SendMsg>::new()
            );
            let mut writer = Self::into_sink(writer); // Convert to dynamic type
            // for uniform abstraction for users

            while let Some(msg) = reader.next().await {
                match msg {
                    Ok(new_msg) => {
                        handler.dispatch(new_msg, &mut writer);
                    },
                    Err(e) => {
                        log::warn!("Connection error for Peer {}: {}", peer_address, e);
                        return;
                    }
                }
            }
            log::warn!("Connection closed by peer");
        });
    }

    async fn run(&self) {
        let listener = TcpListener::bind(self.address)
            .await
            .expect("Failed to bind to address");

        log::debug!("TCP Receiver is listening on {}", self.address);
        loop {
            let (sock, peer_addr) = match listener.accept().await {
                Err(e) => {
                    log::warn!("Listener error: {}", e);
                    return;
                },
                Ok(x) => x,
            };
            log::info!("Connected to {}", peer_addr);
            Self::spawn_runner(sock, peer_addr, self.handler.clone());
        }
    }
}