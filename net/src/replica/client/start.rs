use std::net::SocketAddr;

use config::Node;
use libp2p::futures::SinkExt;
use tokio::{net::{TcpListener}, net::{TcpStream}, stream::StreamExt, sync::mpsc::channel};
use tokio_util::codec::{FramedRead, FramedWrite};
use types::{Block, Transaction};
use util::codec::{tx::{Codec as TxCodec}, block::{Codec as BlockCodec}};
use tokio::sync::mpsc::{Sender, Receiver};
use std::io::Result;

pub async fn start(config:&Node) -> (Sender<Block>, Receiver<Transaction>) {
    let cli_listen = TcpListener::bind(
        format!("0.0.0.0:{}", config.client_port)
    ).await
    .expect("Failed to bind to client port");

    let (send, recv) = channel(100_000);
    let (blk_send, mut blk_recv) = channel::<Block>(100_000);
    tokio::spawn(async move {
        let mut writers = Vec::new();
        loop {
            let new_send = send.clone();
            tokio::select! {
                new_client_opt = cli_listen.accept() => {
                    match handle_client(new_client_opt, new_send).await {
                        None => continue,
                        Some(writer) => {
                            writers.push(writer);
                        },
                    }
                },
                new_blk_opt = blk_recv.recv() => {
                    match new_blk_opt {
                        None => break,
                        Some(b) => {
                            // println!("Sending block to the client");
                            send_blk(&b, writers.clone()).await;
                        }
                    }
                    
                },
            }
        }
    });
    return (blk_send, recv);
}

async fn send_blk(b: &Block, writers: Vec<Sender<Block>>) {
    for i in writers {
        match i.send(b.clone()).await {
            Err(e) => {
                println!("Error sending block to the client: {}",e);
                continue;
            },
            Ok(()) => {},
        };
    }
}

async fn handle_client(new_client: Result<(TcpStream, SocketAddr)>, send: Sender<Transaction>) -> Option<Sender<Block>> {
    let client = match new_client {
        Err(e) => {
            println!("got an error: {}", e);
            return None;
        },
        Ok((conn,from)) => {
            println!("Connected to a new client {}", from);
            conn
        },
    };
    let (rd, wr) = client.into_split();
    let mut reader = FramedRead::new(rd, TxCodec::new());
    tokio::spawn(async move {
        loop 
        {
            let tx_opt = reader.next().await;
            let tx = match tx_opt {
                None => return,
                Some(Ok(t)) => t,
                Some(Err(e)) => {
                    println!("Got an error [{}] when reading the transaction from the client", e);
                    return;
                }
            };
            send.send(tx).await.expect("failed to send the transaction to the outside world");
        }
    });
    let (send, mut recv) = channel(100_000);
    let mut out = FramedWrite::new(wr, BlockCodec::new());
    tokio::spawn(async move {
        loop {
            match recv.recv().await {
                None => return,
                Some(x) => match out.send(x).await {
                    Err(e) => {
                        println!("Error [{}] sending the block to the client", e);
                        return;
                    },
                    Ok(()) => {},
                },
            } 
        }
    });
    return Some(send);
}