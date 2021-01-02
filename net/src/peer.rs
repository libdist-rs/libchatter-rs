use std::collections::VecDeque;

use futures::{SinkExt, stream};
use tokio::{net::tcp::{OwnedReadHalf, OwnedWriteHalf}};
use tokio_util::codec::{Decoder, Encoder, FramedRead, FramedWrite};
use types::WireReady;
use tokio_stream::{StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc::{Sender, Receiver, channel};
// use crate::{Sender, Receiver};
// use crossfire::mpsc::bounded_future_both;

pub struct Peer<I,O> 
where I: WireReady,
O: WireReady,
{
    // Send O msg to this peer
    pub send: Sender<Arc<O>>,
    // Get I msg from this peer
    pub recv: Receiver<I>,
    
    // phantom: PhantomData<&'de O>,
}

enum InternalInMsg {
    Ready,
}

enum InternalOutMsg<O> {
    Batch(VecDeque<Arc<O>>),
}

impl<'de,I,O> Peer<I,O> 
where I: WireReady+'static+Sync+Unpin,
O: WireReady+'static + Clone+Sync,
{
    pub fn add_peer(
        rd: OwnedReadHalf,
        wr: OwnedWriteHalf,
        d: impl Decoder<Item=I, Error=std::io::Error> + Send + 'static,
        e: impl Encoder<Arc<O>> + Send + 'static
    ) -> Self 
    {
        // channels used by the peer to talk to the sockets: 
        // the send is used to get message from the outside and send it to the
        // network
        //
        // 
        let (send_in, recv_in) = channel::<I>(util::CHANNEL_SIZE);
        let (send_out, mut recv_out) = channel::<Arc<O>>(util::CHANNEL_SIZE);
        
        let mut reader = FramedRead::new(rd, d);
        let mut writer = FramedWrite::new(wr, e);
        let handle = tokio::runtime::Handle::current();
        let (internal_ch_in_send, mut internal_ch_in_recv) = channel(util::CHANNEL_SIZE);
        let (internal_ch_out_send, mut internal_ch_out_recv) = channel::<InternalOutMsg<O>>(util::CHANNEL_SIZE);
        handle.spawn(async move {
            loop {
                let opt = internal_ch_out_recv.recv().await;
                if let Some(InternalOutMsg::Batch(to_send)) = opt {
                    let mut s = stream::iter(to_send.into_iter().map(Ok));
                    if let Err(_e) = writer.send_all(&mut s).await {
                        break;
                    }
                    if let Err(_e) = internal_ch_in_send.send(InternalInMsg::Ready).await {
                        break;
                    }
                } else {
                    break;
                }
            }
        });
        handle.spawn(async move {
            let mut buffers = VecDeque::new();
            // let mut write_task= FuturesUnordered::new();
            let mut ready = true;
            loop {
                tokio::select! {
                    in_opt = reader.next() => {
                        if let None = in_opt {
                            println!("Disconnected from peer");
                            break;
                        }
                        if let Some(Ok(x)) = in_opt {
                            if let Err(_e) = send_in.send(x).await {
                                break;
                            }
                        }
                    },
                    out_opt = recv_out.recv() => {
                        if let None = out_opt {
                            break;
                        }
                        if let Some(x) = out_opt {
                            // Write if not already writing, otherwise
                            // buffer and try again later
                            if ready {
                                buffers.push_back(x);
                                if let Err(_e) = internal_ch_out_send.send(InternalOutMsg::Batch(buffers)).await {
                                    break;
                                }
                                buffers = VecDeque::new();
                            } else {
                                buffers.push_back(x);
                            }
                        }
                    },
                    internal_ch_recv_opt = internal_ch_in_recv.recv() => {
                        if let Some(InternalInMsg::Ready) = internal_ch_recv_opt {
                            ready = true;                                
                        } else {
                            break;
                        }
                    }
                }
            }
        });
        
        Self {
            send: send_out,
            recv: recv_in,
            
            // phantom: PhantomData,
        }
    }
}