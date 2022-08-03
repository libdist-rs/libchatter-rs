use std::{net::SocketAddr, marker::PhantomData, collections::VecDeque, time::Duration, cmp::min};
use fnv::FnvHashMap;
use futures::SinkExt;
use rand::{rngs::SmallRng, SeedableRng};
use tokio::{sync::{oneshot, mpsc::{UnboundedSender, unbounded_channel, UnboundedReceiver}}, net::TcpStream, time::{sleep, Sleep}};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::{Message, NetError, Decodec, EnCodec};

/// This handler returns the message that the protocol was trying to send after failing several times
/// If successful, the received ack will be echoed here
/// Otherwise, the handler will be dropped, which can be handled to re-initiate the sending by the upstream protocol
pub type CancelHandler<Msg> = oneshot::Receiver<Msg>;

#[derive(Debug)]
struct InnerMsg<SendMsg, RecvMsg> 
{
    payload: SendMsg,
    cancel_handler: oneshot::Sender<RecvMsg>,
}

pub struct TcpReliableSender<Id, SendMsg, RecvMsg>
{
    address_map: FnvHashMap<Id, SocketAddr>,
    connections: FnvHashMap<Id, UnboundedSender<InnerMsg<SendMsg, RecvMsg>>>,
    _x: PhantomData<RecvMsg>,
    /// Small RNG just used to shuffle nodes and randomize connections (not crypto related).
    rng: SmallRng,        
}

impl<Id, SendMsg, RecvMsg> TcpReliableSender<Id, SendMsg, RecvMsg>
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

    fn spawn_connection(address: SocketAddr) -> UnboundedSender<InnerMsg<SendMsg, RecvMsg>>
    {
        let (tx, rx) = unbounded_channel();
        tokio::spawn(async move {
            Connection::<SendMsg, RecvMsg>::spawn(address, rx);
        });
        tx
    }
}

struct Connection<SendMsg, RecvMsg> 
{
    /// The destination address.
    address: SocketAddr,
    /// Channel from which the connection receives its commands.
    receiver: UnboundedReceiver<InnerMsg<SendMsg, RecvMsg>>,
    /// The initial delay to wait before re-attempting a connection (in ms).
    retry_delay: std::time::Duration,
    /// Buffer keeping all messages that need to be re-transmitted.
    buffer: VecDeque<(SendMsg, oneshot::Sender<RecvMsg>)>,
    _x: PhantomData<RecvMsg>,
}

impl<SendMsg, RecvMsg> Connection<SendMsg, RecvMsg>
{
    const RETRY_INITIAL: std::time::Duration = std::time::Duration::from_millis(50);
}

struct Waiter {
    delay: Duration,
    current: Duration,
    retry: usize,
}

impl Waiter {
    fn new(delay: std::time::Duration) -> Self {
        Self { delay, current: delay, retry: 0 }
    }

    fn reset(&mut self) {
        self.current = self.delay;
        self.retry = 0;
    }

    fn attempt(&self) -> usize {
        self.retry
    }

    fn new_attempt(&mut self) {
        self.retry += 1;
        self.current = min(2*self.current, Duration::from_millis(60_000));
    }
}

impl<SendMsg, RecvMsg> Connection<SendMsg, RecvMsg> 
where 
    SendMsg: Message,
    RecvMsg: Message,
{
    fn spawn(address: SocketAddr, receiver: UnboundedReceiver<InnerMsg<SendMsg, RecvMsg>>)
    {
        tokio::spawn(async move {
            Self {
                address,
                receiver,
                retry_delay: Self::RETRY_INITIAL,
                buffer: VecDeque::default(),
                _x: PhantomData,
            }.run()
            .await;
        });
    }

    async fn run(&mut self) 
    {
        let mut waiter = Waiter::new(self.retry_delay);
        loop {
            match TcpStream::connect(self.address).await {
                Ok(stream) => {
                    log::info!("Connected to {}", self.address);
                    // Reset the delay back to max
                    waiter.reset();

                    let error = self.keep_alive(stream).await;
                    log::warn!("Keep alive error for {}: {}", self.address, error);
                },
                Err(e) => {
                    log::warn!("Failed to connect {} (Attempt: {}) with error {}", self.address, waiter.attempt(), e);
                    let timer = sleep(waiter.current);
                    tokio::pin!(timer);

                    'waiting: loop {
                        tokio::select! {
                            // Wait an increasing delay before attempting to reconnect.
                            () = &mut timer => {
                                waiter.new_attempt();
                                break 'waiting;
                            },
                        }
                    }
                }
            }
        }
    }

    async fn keep_alive(&mut self, mut stream: TcpStream) -> NetError {
        // This buffer keeps all messages and handlers that we have successfully transmitted but for
        // which we are still waiting to receive an ACK.
        let mut pending_replies : VecDeque<(SendMsg, oneshot::Sender<RecvMsg>)>= VecDeque::new();
        'connection: loop {
            let (rd, wr) = stream.split();
            let mut reader = FramedRead::new(
                rd, 
                Decodec::<RecvMsg>::new()
            );
            let mut writer = FramedWrite::new(
                wr, 
                EnCodec::<SendMsg>::new()
            );
            // Try to send all messages of the buffer.
            while let Some((data, handler)) = self.buffer.pop_front() {
                // Skip messages that have been cancelled.
                if handler.is_closed() {
                    continue;
                }

                // Try to send the message.
                match writer.send(data.clone()).await {
                    Ok(()) => {
                        // The message has been sent, we remove it from the buffer and add it to
                        // `pending_replies` while we wait for an ACK.
                        pending_replies.push_back((data, handler));
                    }
                    Err(e) => {
                        // We failed to send the message, we put it back into the buffer.
                        self.buffer.push_front((data, handler));
                        break 'connection NetError::Generic(format!("Failed to send message to {} with error {}", self.address, e));
                    }
                }
            }
            // Check if there are any new messages to send or if we get an ACK for messages we already sent.
            tokio::select! {
                Some(InnerMsg{payload: data, cancel_handler}) = self.receiver.recv() => {
                    // Add the message to the buffer of messages to send.
                    self.buffer.push_back((data, cancel_handler));
                },
                response = reader.next() => {
                    let (data, handler) = match pending_replies.pop_front() {
                        Some(message) => message,
                        None => break 'connection NetError::Generic(format!("Unexpected ack from {}", self.address)),
                    };
                    match response {
                        Some(Ok(msg)) => {
                            // Notify the handler that the message has been successfully sent.
                            let _ = handler.send(msg);
                        },
                        _ => {
                            // Something has gone wrong (either the channel dropped or we failed to read from it).
                            // Put the message back in the buffer, we will try to send it again.
                            pending_replies.push_front((data, handler));
                            break 'connection NetError::Generic(format!("Failed to receive ack from {}", self.address));
                        }
                    }
                },
            }
        }
    }
}

