use std::{
    collections::HashMap, 
    time::SystemTime
};

use config::Client;
use types::{Block, ClientMsg, GENESIS_BLOCK, Height, Transaction};
use tokio::sync::mpsc::channel;
use util::new_dummy_tx;
use crypto::hash::Hash;
use crate::statistics;
use std::sync::Arc;
use util::codec::EnCodec;
use types::ClientMsgCodec;
use futures::{SinkExt, StreamExt};
use net::futures_manager::TlsClient;

struct Context {
    pub pending: usize,
    pub num_cmds: u128,
    pub time_map: HashMap<Hash, SystemTime>,
    pub latency_map: HashMap<Hash, (SystemTime, SystemTime)>,
    pub height_map:HashMap<Height, Arc<Block>>,
    pub hash_map:HashMap<Hash, Arc<Block>>,
    pub last_committed_block: Arc<Block>,
    pub last_block:Arc<Block>, 
}

impl Context {
    pub fn new() -> Self {
        let genesis_arc = Arc::new(GENESIS_BLOCK);
        Context {
            pending: 0,
            num_cmds: 0,
            time_map: HashMap::new(),
            latency_map: HashMap::new(),
            height_map: HashMap::new(),
            hash_map: HashMap::new(),
            last_committed_block: genesis_arc.clone(),
            last_block: genesis_arc,
        }
    }
}

pub async fn start(
    c:&Client, 
    metric: u64,
    window: usize,
) {

    let mut client_network = TlsClient::<ClientMsg, Transaction>::new(c.root_cert.clone());
    let servers = c.net_map.clone();
    let send_id = c.num_nodes;
    let (mut net_send,mut net_recv) = 
        client_network.setup(servers, EnCodec::new(), ClientMsgCodec::new()).await;

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
    let mut cx = Context::new();
    cx.pending = window;
    // =============
    // Statistics
    // =============
    cx.num_cmds = 0;
    // Send f blocks worth of transactions first
    let first_send = c.num_faults*c.block_size;
    log::debug!(target:"consensus", "Sending {} number of transactions initially", first_send);
    let mut net_send_p = net_send.clone();
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
        let mut cx = cx;
        for _ in 0..(first_recv) {
            let (_, b) = net_recv.next().await.unwrap();
            let blk = match b {
                ClientMsg::NewBlock(p,_pl) => p.block.unwrap(),
                _ => continue,
            };
            let b = blk;
            let is_in_ht_map = cx.height_map.contains_key(&b.header.height);
            let is_in_hash_map = cx.hash_map.contains_key(&b.hash);
            if is_in_ht_map && !is_in_ht_map {
                panic!("Got equivocating blocks");
            }
            if is_in_hash_map {
                continue;
            }
            cx.height_map.insert(b.header.height, b.clone());
            cx.hash_map.insert(b.hash, b);
        }
        (net_recv, cx)
    });
    let mut recv = first_send_tx.await.unwrap();
    let val = first_recv_b.await.unwrap();
    let mut net_recv = val.0;
    let mut cx = val.1;
    log::debug!(target:"consensus", "Finished sending first few blocks");
    let mut new_blocks = Vec::new();
    let start = SystemTime::now();
    loop {
        tokio::select! {
            tx_opt = recv.recv(), if cx.pending > 0 => {
                if let Some(x) = tx_opt {
                    let tx = x.as_ref();
                    let hash = crypto::hash::ser_and_hash(tx);
                    net_send.send((send_id as u16,x)).await
                        .expect("Failed to send to the client");
                    cx.time_map.insert(hash, SystemTime::now());
                    cx.pending -= 1;
                    // println!("Sending transaction to the leader");
                } else {
                    println!("Finished sending messages");
                    std::process::exit(0);
                }
            },
            block_opt = net_recv.next() => {
                let now = SystemTime::now();
                log::debug!(target:"consensus",
                    "Got something from the network");
                if let None = block_opt {
                    panic!("invalid content received from the server");
                }
                let (_, b) = block_opt.unwrap();
                log::debug!(target:"consensus","Got a client message: {:?}", b);
                let blk = match b {
                    ClientMsg::NewBlock(p, _pl) => p.block.unwrap(),
                    _ => continue,
                };
                let b = blk;
                new_blocks.push(b);
                while let Ok(Some((_, ClientMsg::NewBlock(p,_)))) = net_recv.try_next() {
                    let b = p.block.clone().unwrap();
                    new_blocks.push(b);
                }
                handle_new_blocks(c, &mut new_blocks, &mut cx, now);
            } 
        }
        if cx.num_cmds > m as u128 {
            let now = SystemTime::now();
            statistics(now, start, cx.latency_map);
            return;
        }
    }
}

fn handle_new_blocks(c: &Client, blocks: &mut Vec<Arc<Block>>, cx: &mut Context, now: SystemTime) {
    for b in blocks {
        let is_in_ht_map = cx.height_map.contains_key(&b.header.height);
        let is_in_hash_map = cx.hash_map.contains_key(&b.hash);
        if is_in_ht_map && 
        !is_in_ht_map {
            // Got equivocating blocks
            panic!("Got equivocating blocks");
        }
        if is_in_hash_map {
            continue;
        }
        cx.last_block = b.clone();
        cx.height_map.insert(b.header.height,b.clone());
        cx.hash_map.insert(b.hash, b.clone());
        if c.num_faults+1 > b.header.height as usize {
            continue;
        }
        if !cx.hash_map.contains_key(&cx.last_block.header.prev) {
            println!("Do not have parent for this block {:?}, yet", cx.last_block);
            }
            let commit_block = cx.height_map.get(&(b.header.height-c.num_faults as u64))
                .expect("Must be in the height map");
            if cx.last_committed_block.hash != commit_block.header.prev {
                panic!("Hash chain broken by new blocks");
                // TODO: Add delivery
            }
            cx.last_committed_block = commit_block.clone();
            // println!("got a block:{:?}",b);
                
            // Use f+1 rule to commit the block
            cx.pending += c.block_size;
            cx.num_cmds += c.block_size as u128;
            for t in &commit_block.body.tx_hashes {
                if let Some(old) = cx.time_map.get(t) {
                    cx.latency_map.insert(t.clone(), (old.clone(), now));
                } else {
                    // Transaction not found in time map
                    cx.num_cmds -= 1;
                }
            }
    }
}