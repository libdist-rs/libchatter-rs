use std::{collections::HashMap, time::SystemTime};

use config::Client;
use types::{Block, GENESIS_BLOCK, Height, Transaction};
use tokio::sync::mpsc::{Receiver, Sender, channel};
use util::{new_dummy_tx};
use crypto::hash::Hash;

pub async fn start(
    c:&Client, 
    net_send: Sender<Transaction>, 
    mut net_recv: Receiver<Block>, 
    metric: u64,
    window: usize,
) {
    let payload = c.payload;
    // Start with the sink implementation
    let (send, mut recv) = channel(100_000);
    let m = metric;
    tokio::spawn(async move{
        let mut i = 0;
        loop {
            let tx = new_dummy_tx(i,payload);
            i += 1;
            if let Err(e) = send.send(tx).await {
                println!("Closing tx producer channel: {}", e);
                break;
            }
        }
    });
    let mut pending = window;
    let mut time_map = HashMap::new();
    let mut height_map:HashMap<Height, Block> = HashMap::new();
    let mut hash_map:HashMap<Hash, Block> = HashMap::new();
    let mut last_committed_block = GENESIS_BLOCK;
    let mut last_block:&Block; 
    // =============
    // Statistics
    // =============
    // println!("Using metric: {}", m);
    let mut latency_sum:u128 = 0;
    let mut num_cmds:u128 = 0;
    let start = SystemTime::now();
    loop {
        tokio::select! {
            tx_opt = recv.recv(), if pending > 0 => {
                if let Some(tx) = tx_opt {
                    // let bytes = to_bytes(&tx);
                    let hash = crypto::hash::ser_and_hash(&tx);
                    net_send.send(tx).await
                        .expect("Failed to send to the client");
                    time_map.insert(hash, SystemTime::now());
                    pending -= 1;
                    // println!("Sending transaction to the leader");
                } else {
                    println!("Finished sending messages");
                }
            },
            block_opt = net_recv.recv() => {
                // println!("Got something from the network");
                if let Some(mut b) = block_opt {
                    // println!("Got a block {:?}",b);
                    let now = SystemTime::now();
                    b.update_hash();
                    let b = b;
                    let is_in_ht_map = height_map.contains_key(
                        &b.header.height);
                    let is_in_hash_map = hash_map.contains_key(&b.hash);
                    if is_in_ht_map && 
                        !is_in_ht_map {
                        // Got equivocating blocks
                        panic!("Got equivocating blocks");
                    }
                    if is_in_hash_map {
                        continue;
                    }
                    last_block = &b;
                    height_map.insert(b.header.height, b.clone());
                    hash_map.insert(b.hash, b.clone());
                    if c.num_faults+1 > b.header.height as usize {
                        continue;
                    }
                    if !hash_map.contains_key(&last_block.header.prev) {
                        println!("Do not have parent for this block {:?}, yet", last_block);
                    }
                    let commit_block = height_map.get(
                        &(b.header.height-c.num_faults as u64))
                        .expect("Must be in the height map");
                    if last_committed_block.hash != commit_block.header.prev {
                        panic!("Hash chain broken by new blocks");
                        // TODO: Add delivery
                    }
                    last_committed_block = commit_block.clone();
                    // println!("got a block:{:?}",b);

                    // let commit_block = b.clone();
                    // Use f+1 rule to commit the block
                    pending += c.block_size;
                    num_cmds += c.block_size as u128;
                    for t in &commit_block.body.tx_hashes {
                        if let Some(old) = time_map.get(t) {
                            let diff = now.duration_since(*old).expect("time difference error").as_millis();
                            latency_sum += diff;
                        } else {
                            println!("transaction not found in time map");
                            // println!("time map: {:?}", time_map);
                            // println!("block hashes: {:?}", b.body.tx_hashes);
                            // return;
                            num_cmds -= 1;
                        }
                    }
                    if b.header.height % 100 == 0 {
                        println!("Got 100 blocks");
                    }
                } else {
                    panic!("invalid content received from the server");
                }
            }
        }
        if num_cmds > m as u128 {
            let now = SystemTime::now();
            println!("Statistics:");
            println!("Processed {} commands with throughput {}", num_cmds, (num_cmds as f64)/now.duration_since(start).expect("Time differencing error").as_secs_f64());
            println!("Average latency: {}", 
                (latency_sum as f64)/(num_cmds as f64));
            return;
        }
    }
}