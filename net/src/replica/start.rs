use std::collections::HashMap;

use config::Node;
use tokio::sync::{mpsc::{Receiver, Sender}, oneshot::channel};
use tokio::{net::{TcpListener, TcpStream}, sync::mpsc::channel as schannel};
use tokio_util::codec::{FramedRead, FramedWrite};
use types::{ProtocolMsg, Replica};
use util::codec::{proto::Codec, EnCodec};
use libp2p::futures::{SinkExt, StreamExt};

pub async fn start(config:&Node) -> 
    Option<(Sender<(Replica, ProtocolMsg)>,Receiver<ProtocolMsg>)>
{
    let my_net_map = config.net_map.clone();
    let proto_listen = TcpListener::bind(
        config.net_map[&config.id].clone()
    ).await
    .expect("Failed to bind to client port");
    // Channels to talk to the network via protocol network
    let (send, recv) = schannel(100_000);
    let mut recv_socks = Vec::new();

    let mut connected = 0;
    let to_connect = config.num_nodes - 1;
    let my_id = config.id;

    let (oneshot_send, one_shot_recv) = channel();
    let listen = tokio::spawn(async move {
        // Then wait for others to connect to you
        while connected < to_connect {
            match proto_listen.accept().await {
                Err(e) => {
                    println!("Error listening to the server: {}", e);
                    continue;
                },
                Ok((conn, from)) => {
                    println!("Incoming connection accepted for {}", from);
                    let (rd, _wr) = conn.into_split();
                    let reader = FramedRead::new(rd, Codec::new());
                    recv_socks.push(reader);
                    connected += 1;
                },
            }
        }
        if let Err(_e) = oneshot_send.send(recv_socks) {
            panic!("Cannot send recv socks out");
        }
        println!("Finished listening. Connected to read.");
    });

    // Spawn connection to the others first
    tokio::time::sleep(std::time::Duration::from_secs_f64(2.0)).await;

    println!("Trying to connect to the other nodes");
    let mut send_socks = HashMap::new();
    let mut idx = 0;
    let mut retry_count:i32 = 300;
    // for (id,addr) in my_net_map {
    loop {
        if (idx as usize) > to_connect {
            break;
        }
        if retry_count == 0 {
            println!("Failed to connect to the server: {}", idx);
            break;
        }
        if idx == my_id {
            idx += 1;
            continue;
        }
        let addr= match my_net_map.get(&idx) {
            None => panic!("No address for server: {}", idx),
            Some(x) => x,
        };
        println!("Trying to connect to: {}", addr.clone());
        match TcpStream::connect(addr).await {
            Err(e) => {
                println!("Failed to connected to server: {}, with error: {}", idx, e);
                    retry_count -= 1;
            },
            Ok(conn) => {
                println!("Successfully connected to write to server: {}", addr.clone() );
                let (_rd, wr) = conn.into_split();
                let writer = FramedWrite::new(wr, EnCodec::new());
                send_socks.insert(idx,writer);
                retry_count = 300;
            }
        }
        idx += 1;
    }
    listen.await.expect("failed to connect to all the nodes");
    let recv_socks = one_shot_recv.await.expect("Failed to finish the listening thread");
    println!("All servers connected to me :)");
    // Setup read threads
    for mut conn in recv_socks {
        let send = send.clone();
        tokio::spawn(async move {
            loop {
                match conn.next().await {
                    None => {},
                    Some(Err(e)) => {
                        println!("Error receiving a message from the server: {}", e);
                        break;
                    }
                    Some(Ok(x)) => {
                        if let Err(e) = send.send(x).await {
                            println!("Failed to send a protocol message to the server: {}", e);
                        }
                    }
                }
            }
        });
    }
    // recv is what we will return to the outside to deal with incoming protocol
    // messages

    let mut writers = Vec::new();
    for (_id, mut conn) in send_socks {
        let (new_send,mut new_recv) = schannel::<ProtocolMsg>(100_000);
        writers.push(new_send);
        tokio::spawn(async move {
            loop {
                match new_recv.recv().await {
                    None => {
                        return;
                    },
                    Some(msg) => {
                        if let Err(e) = conn.send(msg).await {
                            println!("Failed to send the protocol message to the workers with error: {}", e);
                        } 
                    }
                }
            }
        });
    }
    let send_all:u16 = config.num_nodes as u16;
    let (control_send, mut control_recv) = schannel::<(Replica, ProtocolMsg)>(100_000);
    tokio::spawn(async move{
        loop {
            let ev = control_recv.recv().await;
            if let Some((id, msg)) = ev {
                if id == send_all {
                    for w in &writers {
                        if let Err(e) = w.send(msg.clone()).await {
                            println!("failed to tell the workers to send a message, with error: {}", e);
                        }
                    }
                    continue;
                }
                if let Err(e) = writers[id as usize].send(msg).await {
                    println!("failed to tell the workers to send a message, with error: {}", e);
                }
                continue;
            }
            if let None = ev {
                break;
            }
        }
    });
    
    Some((control_send, recv))
}