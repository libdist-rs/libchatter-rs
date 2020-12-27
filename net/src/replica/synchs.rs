use std::{collections::HashMap, time::Duration};

use config::Node;
use tokio::{
    net::{
        TcpListener, 
        TcpStream,
    },
    sync::mpsc::Sender,
};
use tokio_util::codec::{
    FramedRead, 
    FramedWrite
};
use types::{
    synchs::ProtocolMsg, 
    Replica
};
use util::codec::EnCodec;
use tokio_stream::{StreamExt, StreamMap};
use crossfire::{
    mpsc::{
        SharedFutureBoth, 
        RxFuture, 
        TxFuture, 
        bounded_future_both
    }
};
use futures::SinkExt;

use crate::peer::Peer;

pub async fn start(
    config:&Node
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
        let mut readers = HashMap::with_capacity(n);
        for _i in 1..n {
            let (conn, from) = listener
                .accept()
                .await
                .expect("Failed to accept a connection");
            conn.set_nodelay(true).unwrap();
            println!("Connected to {}", from);
            let (rd, wr) = conn.into_split();
            let mut reader = FramedRead::new(rd, util::codec::synchs::Codec::new());
            if let Some(Ok(ProtocolMsg::Identify(id))) = reader.next().await {
                readers.insert(id, reader);
            } else {
                panic!("Invalid message received during identification");
            }
            drop(wr);
        }
        readers
    });
    tokio::time::sleep(Duration::from_secs_f64(2.0)).await;
    let mut writers = HashMap::with_capacity(n);
    for i in 0..n {
        if i as Replica == config.id {
            // writers.insert(i,None));
            continue;
        }
        let id = i as Replica;
        let peer = &my_net_map[&id];
        let conn = TcpStream::connect(peer)
            .await
            .expect("Failed to connect to a peer");
        conn.set_nodelay(true).unwrap();
        let (rd, wr) = conn.into_split();
        let mut writer = FramedWrite::new(wr, EnCodec::new());
        writer.send(ProtocolMsg::Identify(_myid)).await
            .expect("Failed to identify to another node");
        writers.insert(id,writer);
        drop(rd);
        println!("Connected to peer: {}", id);
    }
    // println!("Writers: {:?}", writers);
    
    // Wait till we are connected to everyone
    let mut readers = conn_everyone
        .await
        .expect("Failed to connected to everyone");
    // Convert readers into a stream
    // let mut stream = stream::setup(readers);
    let mut map = StreamMap::new();
    let mut peers:HashMap<Replica, Sender<ProtocolMsg>> = HashMap::with_capacity(n);

    for i in 0..n {
        if i as Replica == config.id {
            continue;
        }
        let repl_id = i as Replica;
        let rd = readers.remove(&repl_id).unwrap().into_inner();
        let d = util::codec::synchs::Codec::new();
        let wr = writers.remove(&repl_id).unwrap().into_inner();
        let e = EnCodec::new();
        let p = Peer::add_peer(rd,wr,d,e);
        let mut p_recv = p.recv;
        let recv = Box::pin(async_stream::stream! {
            while let Some(item) = p_recv.recv().await {
                yield item;
            }
      }) as std::pin::Pin<Box<dyn futures_util::stream::Stream<Item = ProtocolMsg> + Send>>;
        // let recv = p.recv;
        map.insert(repl_id, recv);
        peers.insert(repl_id, p.send);
    }

    // let x = map.next();

    let (msg_rd_send, msg_rd_recv) = bounded_future_both(100_000);
    let (msg_wr_send, msg_wr_recv) = bounded_future_both::<(Replica, ProtocolMsg)>(100_000);

    tokio::spawn(async move {
        loop {
            tokio::select! {
                opt_in = map.next() => {
                    if let Some((_i,x)) = opt_in {
                        if let Err(_e) = msg_rd_send.send(x).await {
                            break;
                        }
                    }
                    else {
                        break;
                    }
                },
                opt_out = msg_wr_recv.recv() => {
                    if let Ok((id,msg)) = opt_out {
                        if id == n as Replica {
                            for (_i,p) in &peers {
                                p.send(msg.clone()).await.unwrap();
                            }
                        } else {
                            peers[&id].send(msg).await.unwrap();
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    });

    Some((msg_wr_send,msg_rd_recv))
}