use std::{collections::HashMap, time::SystemTime};

use config::Client;
use types::{Block, GENESIS_BLOCK, Height, Transaction};
use tokio::sync::mpsc::{channel};
use util::{new_dummy_tx};
use crypto::hash::Hash;
use crate::statistics;
use std::sync::Arc;
use util::codec::{EnCodec, block::Codec};
use std::borrow::Borrow;

pub async fn start(
    c:&Client, 
    metric: u64,
    window: usize,
) {

    let mut client_network = net::Client::<Block, Transaction>::new();
    let servers = c.net_map.clone();
    let send_id = c.num_nodes;
    let (net_send,mut net_recv) = 
        client_network.setup(servers, EnCodec::new(), Codec::new()).await;

    let payload = c.payload;
    // Start with the sink implementation
    let (send, mut recv) = channel(util::CHANNEL_SIZE);
    let m = metric;
    tokio::spawn(async move{
        let mut i = 0;
        loop {
            let tx = new_dummy_tx(i,payload);
            i += 1;
            if let Err(e) = send.send(Arc::new(tx)).await {
                log::info!(target:"consensus","Closing tx producer channel: {}", e);
                std::process::exit(0);
            }
        }
    });
    let mut pending = window;
    let mut time_map = HashMap::new();
    let mut latency_map = HashMap::new();
    let mut height_map:HashMap<Height, Arc<Block>> = HashMap::new();
    let mut hash_map:HashMap<Hash, Arc<Block>> = HashMap::new();
    let mut last_committed_block = Arc::new(GENESIS_BLOCK);
    let mut last_block:&Block; 
    // =============
    // Statistics
    // =============
    let mut num_cmds:u128 = 0;
    // Send f blocks worth of transactions first
    let first_send = c.num_faults*c.block_size;
    log::debug!(target:"consensus", "Sending {} number of transactions initially", first_send);
    let net_send_p = net_send.clone();
    let first_send_tx = tokio::spawn(async move{
    for _ in 0..(first_send) {
        let next = recv.recv().await.unwrap();
        net_send_p.send((send_id as u16,next)).await.unwrap();
    }
    recv
    });
    let first_recv = c.num_faults;
    log::debug!(target:"consensus", "Receiving first {} blocks", first_recv);
    let first_recv_b = tokio::spawn(async move{
    for _ in 0..(first_recv) {
        let (_, mut b) = net_recv.recv().await.unwrap();
        b.update_hash();
        let b = Arc::new(b);
        let is_in_ht_map = height_map.contains_key(&b.header.height);
        let is_in_hash_map = hash_map.contains_key(&b.hash);
        if is_in_ht_map && !is_in_ht_map {
            panic!("Got equivocating blocks");
        }
        if is_in_hash_map {
            continue;
        }
        height_map.insert(b.header.height, b.clone());
        hash_map.insert(b.hash, b);
    }
    (net_recv, height_map, hash_map)
    });
    let mut recv = first_send_tx.await.unwrap();
    let val = first_recv_b.await.unwrap();
    let mut net_recv = val.0;
    let mut height_map = val.1;
    let mut hash_map = val.2;
    log::debug!(target:"consensus", "Finished sending first few blocks");
    let start = SystemTime::now();
    loop {
        tokio::select! {
            tx_opt = recv.recv(), if pending > 0 => {
                if let Some(x) = tx_opt {
                    let tx = x.borrow() as &Transaction;
                    let hash = crypto::hash::ser_and_hash(tx);
                    net_send.send((send_id as u16,x)).await
                    .expect("Failed to send to the client");
                    time_map.insert(hash, SystemTime::now());
                    pending -= 1;
                    // println!("Sending transaction to the leader");
                } else {
                    println!("Finished sending messages");
                    std::process::exit(0);
                }
            },
            block_opt = net_recv.recv() => {
                // println!("Got something from the network");
                if let None = block_opt {
                    panic!("invalid content received from the server");
                }
                let (_, mut b) = block_opt.unwrap(); 
                b.update_hash();
                let b = Arc::new(b);
                // println!("Got a block {:?}",b);
                let now = SystemTime::now();
                let is_in_ht_map = height_map.contains_key(&b.header.height);
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
                // if last_block.header.height % 100 == 0 {
                //     println!("Got 100 blocks");
                // }
                height_map.insert(b.header.height,b.clone());
                hash_map.insert(b.hash, b.clone());
                if c.num_faults+1 > b.header.height as usize {
                    continue;
                }
                if !hash_map.contains_key(&last_block.header.prev) {
                    println!("Do not have parent for this block {:?}, yet", last_block);
                }
                let commit_block = height_map.get(&(b.header.height-c.num_faults as u64))
                .expect("Must be in the height map");
                if last_committed_block.hash != commit_block.header.prev {
                    panic!("Hash chain broken by new blocks");
                    // TODO: Add delivery
                }
                last_committed_block = commit_block.clone();
                // println!("got a block:{:?}",b);
                
                // Use f+1 rule to commit the block
                pending += c.block_size;
                num_cmds += c.block_size as u128;
                for t in &commit_block.body.tx_hashes {
                    if let Some(old) = time_map.get(t) {
                        latency_map.insert(t.clone(), (old.clone(), now));
                    } else {
                        // Transaction not found in time map
                        num_cmds -= 1;
                    }
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