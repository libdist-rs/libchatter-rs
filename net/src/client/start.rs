use std::collections::{HashMap};

use config::Client;

// use libp2p::{identity::Keypair, futures::SinkExt};
use libp2p::futures::SinkExt;
use tokio::{stream::{StreamExt}, net::TcpStream, sync::mpsc::{channel, Sender, Receiver}};
use tokio_util::codec::{FramedRead, FramedWrite};
use types::{Transaction, Block};
use util::codec::{EnCodec, block::{Codec as BlockCodec}};

/// The client does the following:
/// 1. Dial the known servers
/// 2. 
pub async fn start(config:Client) -> (Sender<Transaction>, Receiver<Block>) {
    let (send, recv) = channel(100000);
    let mut writers = HashMap::new();
    for i in config.net_map {
        let new_send = send.clone();
        let tcp = TcpStream::connect(i.1.clone()).await.expect("failed to open a tcp stream");
        let (rd, wr) = tcp.into_split();
        let writer = FramedWrite::new(wr, EnCodec::new());
        writers.insert(i,writer);
        let mut reader = FramedRead::new(rd, BlockCodec::new());
        tokio::spawn(async move {
            loop {
                match reader.next().await {
                    None => {
                        // println!("Got nothing from the reader, breaking");
                        break;
                    },
                    Some(Err(e)) => {
                        println!("Got an error: {}", e);
                        break;
                    },
                    Some(Ok(b)) => new_send.send(b).await.
                        expect("failed to send the new block outside the network"),
                }
            }
        });
        println!("connected to server");
    }
    // for the outside world to talk to the network manager
    let (net_send, mut net_recv) = channel::<Transaction>(100_000);
    let mut out_channels = HashMap::new();
    for (id, mut conn) in writers {
        let (new_send,mut new_recv) = channel::<Transaction>(100_000);
        out_channels.insert(id, new_send);
        tokio::spawn(async move {
            loop {
                match new_recv.recv().await {
                    None => {
                        return;
                    },
                    Some(msg) => {
                        if let Err(e) = conn.send(msg).await {
                            println!("Failed to send the protocol message to the workers with error: {}", e);
                            break;
                        } 
                    }
                }
            }
        });
    }
    tokio::spawn(async move {
    loop {
        if let Some(t) = net_recv.recv().await {
            for (_i,w) in &out_channels {
                if let Err(e) = w.send(t.clone()).await {
                    println!("Failed to send a message to the server: {}", e);
                }
            }
        } else {
            break;
        }
    }
    });

    return (net_send, recv);
}