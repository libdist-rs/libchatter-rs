use config::Client;

// use libp2p::{identity::Keypair, futures::SinkExt};
use libp2p::futures::SinkExt;
use tokio::{stream::{StreamExt}, net::TcpStream, sync::mpsc::{channel, Sender, Receiver}};
use tokio_util::codec::{FramedRead, FramedWrite};
use types::{Transaction, Block};
use util::codec::{EnCodec, block::{Codec as BlockCodec}};

fn new_dummy_tx(idx: u64) -> Transaction {
    Transaction{
        data: idx.to_be_bytes().to_vec(),
    }
}

/// The client does the following:
/// 1. Dial the known servers
/// 2. 
pub async fn start(config:Client) -> (Sender<Block>, Receiver<Block>) {
    let (send, mut recv) = channel(100000);
    let mut writers = Vec::new();
    for i in config.net_map {
        let new_send = send.clone();
        let tcp = TcpStream::connect(i.1).await.expect("failed to open a tcp stream");
        let (rd, wr) = tcp.into_split();
        let writer = FramedWrite::new(wr, EnCodec::new());
        writers.push(writer);
        let mut reader = FramedRead::new(rd, BlockCodec::new());
        tokio::spawn(async move {
            loop {
                match reader.next().await {
                    None => break,
                    Some(Err(e)) => {
                        println!("Got an error: {}", e);
                        break;
                    },
                    Some(Ok(b)) => new_send.send(b).await.
                        expect("failed to send the new block outside the network"),
                }
            }
        });
        println!("connected to server {}", i.0);
    }
    // for the outside world to talk to the network manager
    // let (net_send, mut net_recv) = channel(100_000);
    let (tx_send, mut tx_recv) = channel(100_000);
    // net_rt.spawn(async move {
    //     loop {
    //         tokio::select! {
    //             block = recv.recv() => {
    //                 println!("Got a new block {:?} from the network", block);
    //                 // net_get_send.send(block).await.expect("failed to send a new block outside the network");
    //             },
    //             tx_opt = net_recv.recv() => {
    //                 let tx = match tx_opt {
    //                     None => {break;},
    //                     Some(t) => t,
    //                 };
    //                 println!("Sending transaction: {:?}", tx);
    //                 for w in writers {
    //                     w.send(tx.clone());
    //                 }
    //             }
    //         }
    //     }
    // });

    tokio::spawn(async move {
        let mut tx_ctr = 1;
        loop {
            tx_send.send(new_dummy_tx(tx_ctr)).await
                .expect("failed to send a new transaction to the network");
            tx_ctr += 1;
            // println
        }
    });
    let mut pending:i64 = 500_000;
    let mut num_blks = 0;
    loop {
        tokio::select! {
            block_opt = recv.recv() => {
                let _block = match block_opt {
                    None => {break;},
                    Some(b) => b,
                };
                // println!("Got a new block: {:?}", block);
                pending += config.block_size as i64;
                num_blks += 1;
                if num_blks % 100 == 0 {
                    println!("Got 100 blocks");
                }
            },
            tx_opt = tx_recv.recv(), if pending > 0 => {
                let tx = match tx_opt {
                    None => {break;},
                    Some(t) => t,
                };
                pending -= 1;
                for w in &mut writers {
                    w.send(tx.clone()).await.expect("failed to send the new transaction to the network");
                }
            }
        }
    }
}