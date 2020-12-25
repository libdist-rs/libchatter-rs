use std::time::Duration;

use config::Node;
use tokio::{
    net::{
        tcp::OwnedWriteHalf, 
        TcpListener, 
        TcpStream,
    },
};
use tokio_util::codec::{
    FramedRead, 
    FramedWrite
};
use types::{
    ProtocolMsg, 
    Replica
};
use util::codec::EnCodec;
use libp2p::futures::SinkExt;
use tokio_stream::StreamExt;
use crossfire::mpsc::{
    SharedFutureBoth, 
    RxFuture, 
    TxFuture, 
    bounded_future_both,
};

use super::super::combine_streams;

pub async fn start(
    config:&Node
// ) -> Option<(Sender<(Replica, ProtocolMsg)>,Receiver<ProtocolMsg>)>
) -> Option<(TxFuture<(Replica, ProtocolMsg),SharedFutureBoth>,RxFuture<ProtocolMsg, SharedFutureBoth>)>
{
    let my_net_map = config.net_map.clone();
    let _myid = config.id;
    let listener = TcpListener::bind(
        config.my_ip()
    ).await
    .expect("Failed to bind at my address");
    let n = config.num_nodes;
    let conn_everyone = tokio::spawn(async move{
        let mut readers = Vec::with_capacity(n);
        for _i in 1..n {
            let (conn, from) = listener
                .accept()
                .await
                .expect("Failed to accept a connection");
            println!("Connected to {}", from);
            let (rd, wr) = conn.into_split();
            let reader = FramedRead::new(rd, util::codec::proto::Codec::new());
            readers.push(reader);
            drop(wr);
        }
        readers
    });
    tokio::time::sleep(Duration::from_secs_f64(2.0)).await;
    let mut writers = Vec::new();
    for i in 0..n {
        if i as Replica == config.id {
            writers.push((i,None));
            continue;
        }
        let id = i as Replica;
        let peer = &my_net_map[&id];
        let conn = TcpStream::connect(peer)
            .await
            .expect("Failed to connect to a peer");
        let (rd, wr) = conn.into_split();
        writers.push((i,Some(FramedWrite::new(wr, EnCodec::new()))));
        drop(rd);
        println!("Connected to peer: {}", id);
    }
    // println!("Writers: {:?}", writers);
    
    // Wait till we are connected to everyone
    let readers = conn_everyone
        .await
        .expect("Failed to connected to everyone");
    // Convert readers into a stream
    // let mut stream = stream::setup(readers);
    let mut stream = combine_streams(readers);
    // let (proto_msg_in_send, proto_msg_in_recv) = schannel(100_000);
    let (proto_msg_in_send, proto_msg_in_recv) = bounded_future_both(100_000);
    // let (proto_msg_out_send, mut proto_msg_out_recv) = 
    let (proto_msg_out_send, proto_msg_out_recv) = 
        // schannel::<(Replica, ProtocolMsg)>(100_000);
        bounded_future_both::<(Replica, ProtocolMsg)>(100_000);
    tokio::spawn(async move{
        loop {
            let in_msg = stream.next()
            .await
            .expect("Failed to read from the combined stream");
            proto_msg_in_send.send(
                in_msg.1.unwrap()
            ).await
            .expect("Failed to send the message outside the networking module");
        }
    });
    tokio::spawn(async move{
        let mut writers = writers;
        loop {
            let (sender_id, msg) = proto_msg_out_recv.recv()
                .await
                .expect("Failed to receive incoming message from the outside world");
            // println!("Node {}: trying to send a message to {}", myid, sender_id);
            if sender_id < n as u16 {
                send_one(
                    writers[sender_id as usize].1.as_mut().unwrap(), 
                    msg
                ).await;
                continue;
            }
            writers = send_all(&mut writers, msg).await;
        }
    });
    println!("Successfully connected to all the nodes");
    Some((proto_msg_out_send, proto_msg_in_recv))
    // None
}

async fn send_one(writer: &mut FramedWrite<OwnedWriteHalf, EnCodec>, 
    msg:ProtocolMsg) 
{
    writer.send(msg).await.unwrap();
}

async fn send_all(writers: &mut Vec<(usize,Option<FramedWrite<OwnedWriteHalf, EnCodec>>)>, msg: ProtocolMsg) -> Vec<(usize,Option<FramedWrite<OwnedWriteHalf, EnCodec>>)>
{
    let mut return_writers = Vec::with_capacity(writers.len());
    for i in 0..writers.len() {
        return_writers.push((i,None));
    }
    let mut handles = Vec::with_capacity(writers.len());
    let m = &msg;
    for _i in 0..writers.len() {
        let c = m.clone();
        match writers.pop() {
            Some((id, Some(mut wr))) => {
                handles.push(tokio::spawn(async move {
                    wr.send(c).await.unwrap();
                    (id,wr)
                }));
            },
            Some((id, None)) => {
                return_writers[id] = (id,None);
            }
            None => {},
        }
    }
    for h in handles {
        let (x,y) = h.await.unwrap();
        return_writers[x] = (x,Some(y));
    }
    return_writers
}