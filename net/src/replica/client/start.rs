use config::Node;
use crossfire::{
    mpsc::{
        bounded_future_both, 
        RxFuture, 
        SharedFutureBoth, 
        TxFuture
    }
};
use tokio_stream::StreamMap;
use libp2p::futures::SinkExt;
use tokio::{
    net::{
        TcpListener,
        TcpStream,
        tcp::OwnedWriteHalf
    }, 
    // sync::mpsc::{
    //     channel,
    //     Sender,
    //     Receiver,
    // },
};
use tokio_util::codec::{
    FramedRead, 
    FramedWrite
};
use types::{
    Block, 
    Transaction
};
use util::codec::{
    EnCodec, 
    tx::Codec as TxCodec
};
use tokio_stream::StreamExt;

pub async fn start(
    config:&Node
// ) -> (Sender<Block>, Receiver<Transaction>) 
) -> (TxFuture<Block, SharedFutureBoth>, RxFuture<Transaction, SharedFutureBoth>) 
{
    let cli_listen = TcpListener::bind(
        format!("0.0.0.0:{}", config.client_port)
    ).await
    .expect("Failed to bind to client port");
    
    let (send, recv) = bounded_future_both(100_000);
    let (blk_send, blk_recv) = bounded_future_both::<Block>(100_000);
    let mut readers = StreamMap::new();
    let mut writers = Vec::new();
    // Start listening to new client connections
    let new_conn_ch = cli_manager(cli_listen).await;
    // Main client event loop
    tokio::spawn(async move {
        loop {
            let enable = readers.len() > 0;
            tokio::select! {
                // If the consensus has a block to send to the others
                blk_opt = blk_recv.recv() => {
                    match blk_opt {
                        Err(_e) => break,
                        Ok(b) => {
                            // println!("Sending block to the client");
                            writers = send_blk(&b, writers).await;
                        }
                    }
                },
                // If we have a new connection
                conn_opt = new_conn_ch.recv() => {
                    if let Err(_e) = conn_opt {
                        break;
                    }
                    let conn = conn_opt.unwrap();
                    let (rd,wr) = conn.into_split();
                    let reader = FramedRead::new(rd, TxCodec::new());
                    let writer = FramedWrite::new(wr, EnCodec::new());
                    readers.insert(readers.len(), reader);
                    writers.push(writer);
                }
                // If we have new transactions
                tx_opt = readers.next(), if enable => {
                    // Got a new transaction
                    match tx_opt {
                        Some((_id,Ok(tx))) => {
                            // println!("Got a transaction: {:?}",tx);
                            send.send(tx).await.unwrap();
                        },
                        _ => {},
                    }
                }
            }
            
        }
    });
    return (blk_send, recv);
}

async fn cli_manager(
    listener: TcpListener,
) -> RxFuture<TcpStream, SharedFutureBoth>
{
    let (send, recv) = bounded_future_both(100_000);
    tokio::spawn(async move {
        loop {
            let conn = listener.accept().await;
            let conn = match conn {
                Ok((a,_b)) => {
                    a.set_nodelay(true).unwrap();
                    a
                },
                Err(e) => {
                    println!("Error:{} connecting to client", e);
                    continue;
                },
            };
           send.send(conn).await.unwrap();
        }
    });
    recv
}

async fn send_blk(b: &Block, writers: Vec<FramedWrite<OwnedWriteHalf, EnCodec>>) -> Vec<FramedWrite<OwnedWriteHalf, EnCodec>>
{
    let mut writers_vec = writers;
    let len = writers_vec.len();
    let mut wait_handles = Vec::with_capacity(len); 
    for _i in 0..len {
        let mut wr = writers_vec.remove(0);
        let new_b = b.clone();
        wait_handles.push(
            tokio::spawn(async move {
                match wr.send(new_b).await {
                    Ok(()) => Some(wr),
                    Err(_e) => None,
                }
            })
        );
    }
    for h in wait_handles {
        let wr = match h.await {
            Err(e) => {
                println!("Failed to send to client: {}", e);
                continue;
            },
            Ok(None) => {
                continue;
            }
            Ok(Some(x)) => x,
        };
        writers_vec.push(wr);
    }
    writers_vec
}