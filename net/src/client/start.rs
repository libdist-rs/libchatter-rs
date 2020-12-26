use config::Client;

use tokio::{net::TcpStream, sync::mpsc::{channel, Sender, Receiver}};
use types::{Transaction, Block};
use util::codec::{EnCodec, block::{Codec as BlockCodec}};
use tokio_stream::{StreamExt, StreamMap};

use crate::peer::Peer;

/// The client does the following:
/// 1. Dial the known servers
/// 2. 
pub async fn start(config:&Client) -> (Sender<Transaction>, Receiver<Block>) {
    let n = config.num_nodes;
    let mut peers:Vec<Sender<Transaction>> = Vec::with_capacity(n);
    let mut map = StreamMap::with_capacity(n);
    for (i,addr) in &config.net_map {
        let tcp = TcpStream::connect(addr.clone()).await
            .expect("failed to open a tcp stream");
        let (rd, wr) = tcp.into_split();
        let enc = EnCodec::new();
        let dec = BlockCodec::new();
        let p = Peer::add_peer(rd, wr, dec, enc);
        peers.push(p.send);
        let mut p_recv = p.recv;
        let recv = Box::pin(async_stream::stream! {
            while let Some(item) = p_recv.recv().await {
                yield item;
            }
      }) as std::pin::Pin<Box<dyn futures_util::stream::Stream<Item = Block> + Send>>;
        map.insert(i.clone(), recv);
    }
    // for the outside world to talk to the network manager
    let (net_in_send, mut net_in_recv) = channel::<Transaction>(100_000);
    let (net_out_send, net_out_recv) = channel::<Block>(100_000);
    
    tokio::spawn(async move{
        loop {
            tokio::select! {
                in_opt = net_in_recv.recv() => {
                    if let Some(tx) = in_opt {
                        for i in &peers {
                            i.send(tx.clone()).await.unwrap();
                        }
                    }
                },
                out_opt = map.next() => {
                    if let Some((_id, x)) = out_opt {
                        if let Err(_e) = net_out_send.send(x).await {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    });

    return (net_in_send, net_out_recv);
}