use std::{
    collections::{
        HashMap, 
        HashSet
    }, 
    time::SystemTime
};
use config::Client;
use types::{
    Transaction, 
    Block
};
use tokio::sync::mpsc::{
    channel
};
use util::new_dummy_tx;
use crypto::hash::Hash;
use crate::statistics;
use std::sync::Arc;
use util::codec::{
    EnCodec, 
    block::Codec
};
use std::borrow::Borrow;

pub async fn start(
    c:&Client, 
    metric: u64,
    window: usize,
) {
    let mut client_network = net::Client::<Block, Transaction>::new();
    let servers = c.net_map.clone();
    let send_id = c.num_nodes as u16;
    let (net_send,mut net_recv) = 
        client_network.setup(servers, EnCodec::new(), Codec::new()).await;

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
                log::info!(target:"consensus", "Closing tx producer channel: {}", e);
                std::process::exit(0);
            }
        }
    });
    let mut pending = window;
    let mut time_map = HashMap::new();
    let mut count_map:HashMap<Hash, usize> = HashMap::new();
    let mut finished_map:HashSet<Hash> = HashSet::new();
    let mut latency_map = HashMap::new();
    let mut num_cmds:u128 = 0;

    let start = SystemTime::now();
    loop {
        tokio::select! {
            tx_opt = recv.recv(), if pending > 0 => {
                if let Some(x) = tx_opt {
                    let hash = crypto::hash::ser_and_hash(x.borrow() as &Transaction);
                    net_send.send((send_id, x))
                        .expect("Failed to send to the client");
                    time_map.insert(hash, SystemTime::now());
                    pending -= 1;
                    log::debug!(target:"consensus", 
                        "Sending transaction to the leader");
                } else {
                    log::info!(target:"client", "Finished sending messages");
                    std::process::exit(0);
                }
            },
            block_opt = net_recv.recv() => {
                // Got something from the network
                if let Some((_, mut b)) = block_opt {
                    b.update_hash();
                    let b = Arc::new(b);
                    log::debug!(target:"consensus", "got a block:{:?}",b);
                    
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
                            latency_map.insert(t.clone(), (old.clone(),now));
                        } else {
                            log::warn!(target:"consensus", 
                                "transaction not found in time map");
                            num_cmds -= 1;
                        }
                    }
                    finished_map.insert(b.hash);
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