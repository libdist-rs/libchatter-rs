use tokio::{net::{TcpStream, tcp::{OwnedReadHalf, OwnedWriteHalf}}, sync::mpsc::{unbounded_channel, UnboundedSender, UnboundedReceiver}};
use tokio_util::codec::{FramedRead, FramedWrite};
use crate::{Message, Decodec, EnCodec, plaintcp::PeerConnectionMsg};
use tokio_stream::{StreamExt};
use futures::sink::SinkExt;

pub struct TcpConnection<SendMsg, RecvMsg>
where
    SendMsg: Message,
    RecvMsg: Message,
{
    reader: FramedRead<OwnedReadHalf, Decodec<RecvMsg>>,
    writer: FramedWrite<OwnedWriteHalf, EnCodec<SendMsg>>,
}

impl<SendMsg, RecvMsg> TcpConnection<SendMsg, RecvMsg>
where
    SendMsg: Message,
    RecvMsg: Message,
{
    pub fn new(stream: TcpStream) -> Self {
        let (read_sock, write_sock) = stream.into_split();
        let decoder = Decodec::new();
        let reader = FramedRead::new(read_sock, decoder);

        let encoder = EnCodec::new();
        let writer = FramedWrite::new(write_sock, encoder);
        Self { reader, writer }
    }

    pub fn start(&mut self) -> (UnboundedSender<PeerConnectionMsg<SendMsg,RecvMsg>>,UnboundedReceiver<PeerConnectionMsg<SendMsg, RecvMsg>>) 
    {
        let (conn_in, conn_in_recv) = unbounded_channel::<PeerConnectionMsg<SendMsg, RecvMsg>>();
        let (conn_out_in, conn_out) = unbounded_channel();
        let writer = self.writer;
        let reader = self.reader;
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    in_msg = conn_in_recv.recv() => {
                        if let None = in_msg {
                            log::warn!("Broken receiver to connection. Shutting down connection");
                            break;
                        }
                        let msg = in_msg.unwrap();
                        if let PeerConnectionMsg::SendMsg(msg_to_send) = msg {
                            writer.send(msg_to_send).await;
                        } else {
                            log::warn!("Invalid Net Message received by the peer");
                        }
                    }
                    new_msg = reader.next() => {
                        if let None = new_msg {
                            conn_out_in.send(PeerConnectionMsg::ConnectionError("Connection closed".into()));
                            break;
                        }
                        let new_msg = new_msg.unwrap();
                        if let Err(e) = new_msg {
                            conn_out_in.send(PeerConnectionMsg::ConnectionError(e.into()));
                            break;
                        }
                        let new_msg = new_msg.unwrap();
                        conn_out_in.send(PeerConnectionMsg::NewMsg(new_msg));
                    }
                }
            }
        });
        (conn_in, conn_out)
    }
}