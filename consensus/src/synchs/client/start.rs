use std::{collections::{HashMap, HashSet}, time::{SystemTime}};

use config::Client;
use types::{Transaction, Block};
use tokio::sync::mpsc::{channel};
use tokio::sync::mpsc::{Sender, Receiver};
// use crate::{Sender, Receiver};
use util::{new_dummy_tx};
use crypto::hash::Hash;
use crate::statistics;
use std::sync::Arc;
use std::borrow::Borrow;

pub async fn start(
    c:&Client, 
    net_send: Sender<Arc<Transaction>>, 
    mut net_recv: Receiver<Arc<Block>>, 
    metric: u64,
    window: usize,
) {
    // Start with the sink implementation
    let (send, mut recv) = channel(util::CHANNEL_SIZE);
    let m = metric;
    let payload = c.payload;
    tokio::spawn(async move{
        let mut i = 0;
        loop {
            let tx = new_dummy_tx(i,payload);
            i += 1;
            if let Err(e) = send.send(Arc::new(tx)).await {
                println!("Closing tx producer channel: {}", e);
                std::process::exit(0);
            }
        }
    });
    let mut pending = window;
    let mut time_map = HashMap::new();
    let mut count_map:HashMap<Hash, usize> = HashMap::new();
    let mut finished_map:HashSet<Hash> = HashSet::new();
    let mut latency_map = HashMap::new();
    // =============
    // Statistics
    // =============
    // println!("Using metric: {}", m);
    // let mut latency_sum:u128 = 0;
    let mut num_cmds:u128 = 0;
    let start = SystemTime::now();
    loop {
        tokio::select! {
            tx_opt = recv.recv(), if pending > 0 => {
                if let Some(x) = tx_opt {
                    // let bytes = to_bytes(&tx);
                    let hash = crypto::hash::ser_and_hash(x.borrow() as &Transaction);
                    net_send.send(x).await
                        .expect("Failed to send to the client");
                    time_map.insert(hash, SystemTime::now());
                    pending -= 1;
                    // println!("Sending transaction to the leader");
                } else {
                    println!("Finished sending messages");
                    break;
                }
            },
            block_opt = net_recv.recv() => {
                // Got something from the network
                if let Some(b) = block_opt {
                    // println!("got a block:{:?}",b);
                    // Check if the block is valid?
                    if !count_map.contains_key(&b.hash) {
                        count_map.insert(b.hash, 1);
                        continue;
                    }
                    let ct = count_map.get(&b.hash).unwrap().clone();
                    if ct < c.num_faults {
                        count_map.insert(b.hash, ct+1);
                        continue;
                    }
                    let now = SystemTime::now();
                    if finished_map.contains(&b.hash) {
                        continue;
                    }
                    pending += c.block_size;
                    num_cmds += c.block_size as u128;
                    for t in &b.body.tx_hashes {
                        if let Some(old) = time_map.get(t) {
                            // let diff = now.duration_since(*old).expect("time difference error").as_millis();
                            // latency_sum += diff;
                            latency_map.insert(t.clone(), (old.clone(),now));
                        } else {
                            println!("transaction not found in time map");
                            // println!("time map: {:?}", time_map);
                            // println!("block hashes: {:?}", b.body.tx_hashes);
                            // return;
                            num_cmds -= 1;
                        }
                    }
                    finished_map.insert(b.hash);
                    // if b.header.height % 100 == 0 {
                        // println!("Got 100 blocks");
                    // }
                } else {
                    panic!("invalid content received from the server");
                }
            }
        }
        if num_cmds > m as u128 {
            let now = SystemTime::now();
            statistics(now, start, latency_map);
            return;
        }
    }
}