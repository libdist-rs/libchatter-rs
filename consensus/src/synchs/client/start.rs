use std::{collections::HashMap, time::SystemTime};

use config::Client;
use types::{Transaction, Block};
use tokio::sync::mpsc::{Receiver, Sender, channel};
use util::{new_dummy_tx, io::to_bytes};
use crypto::hash::Hash;

pub async fn start(
    c:&Client, 
    net_send: Sender<Transaction>, 
    mut net_recv: Receiver<Block>, 
    metric: u64
) {
    // Start with the sink implementation
    let (send, mut recv) = channel(100_000);
    let m = metric;
    tokio::spawn(async move{
        let mut i = 0;
        loop {
            let tx = new_dummy_tx(i);
            i += 1;
            if let Err(e) = send.send(tx).await {
                println!("Closing tx producer channel: {}", e);
                break;
            }
        }
    });
    let mut pending = c.block_size;
    let mut time_map = HashMap::new();
    let mut count_map:HashMap<Hash, usize> = HashMap::new();
    // =============
    // Statistics
    // =============
    let mut latency_sum:u128 = 0;
    let mut num_cmds:u128 = 0;
    loop {
        tokio::select! {
            tx_opt = recv.recv(), if pending > 0 => {
                if let Some(tx) = tx_opt {
                    let bytes = to_bytes(&tx);
                    let hash = crypto::hash::do_hash(&bytes);
                    time_map.insert(hash, SystemTime::now());
                    net_send.send(tx).await
                        .expect("Failed to send to the client");
                    pending -= 1;
                    // println!("Sending transaction to the leader");
                } else {
                    println!("Finished sending messages");
                }
            },
            block_opt = net_recv.recv() => {
                // println!("Got something from the network");
                if let Some(mut b) = block_opt {
                    b.update_hash();
                    let b = b;
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
                    pending += c.block_size;
                    num_cmds += c.block_size as u128;
                    for t in b.body.tx_hashes {
                        if let Some(old) = time_map.get(&t) {
                            let diff = old.duration_since(now).expect("time difference error").as_millis();
                            latency_sum += diff;
                        } else {
                            num_cmds -= 1;
                        }
                    }
                } else {
                    panic!("invalid content received from the server");
                }
            }
        }
        if num_cmds > m as u128 {
            println!("Statistics:");
            println!("Processed {} commands", num_cmds);
            println!("Average latency: {}", 
                (latency_sum as f64)/(num_cmds as f64));
        }
    }
}