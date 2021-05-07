use std::time::SystemTime;
use config::Client;
use types::apollo::{ClientMsg, Propose, Transaction};
use tokio::sync::mpsc::channel;
use consensus::statistics;
use std::sync::Arc;
use util::codec::EnCodec;
use util::codec::Decodec;
use futures::{SinkExt, StreamExt};
use net::futures_manager::TlsClient;
use super::*;

pub async fn start(
    c:&Client, 
    metric: u64,
    window: usize,
) {
    let mut client_network = TlsClient::<ClientMsg, Transaction>::new(c.root_cert.clone());
    let servers = c.net_map.clone();
    let send_id = c.num_nodes;
    let (mut net_send,mut net_recv) = 
        client_network.setup(servers, EnCodec::new(), Decodec::<ClientMsg>::new()).await;

    let payload = c.payload;
    // Start with the sink implementation
    let (send, mut recv) = channel(util::CHANNEL_SIZE);
    let m = metric;
    tokio::spawn(async move{
        let mut i = 0;
        loop {
            let tx = Transaction::new_dummy_tx(i,payload);
            i += 1;
            if let Err(e) = send.send(Arc::new(tx)).await {
                log::info!("Closing tx producer channel: {}", e);
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
    log::debug!("Sending {} number of transactions initially", first_send);
    let mut net_send_p = net_send.clone();
    let first_send_tx = tokio::spawn(async move{
    for _ in 0..(first_send) {
        let next = recv.recv().await.unwrap();
        net_send_p.send((send_id,next)).await.unwrap();
    }
    recv
    });
    let first_recv = c.num_faults;
    log::debug!("Receiving first {} blocks", first_recv);
    let first_recv_b = tokio::spawn(async move{
        let mut cx = cx;
        for _ in 0..(first_recv) {
            let (_, msg) = net_recv.next().await.unwrap();
            let prop = match msg {
                ClientMsg::NewBlock(p,_pl) => p,
                _ => continue,
            };
            update_props(prop, &mut cx);
            while let Some(p) = cx.future_msgs.remove(&cx.round) {
                let b = p.block.clone().unwrap();
                if !cx.storage.is_delivered_by_hash(&b.header.prev) {
                    panic!("Got an undelivered block");
                }
                cx.storage.add_delivered_block(b);
                cx.round += 1;
            }
        }
        (net_recv, cx)
    });
    let mut recv = first_send_tx.await.unwrap();
    let val = first_recv_b.await.unwrap();
    let mut net_recv = val.0;
    let mut cx = val.1;
    log::debug!("Finished sending first few blocks");
    let start = SystemTime::now();
    loop {
        tokio::select! {
            tx_opt = recv.recv(), if cx.pending > 0 => {
                if let Some(x) = tx_opt {
                    let tx = x.as_ref();
                    let hash = crypto::hash::ser_and_hash(tx);
                    net_send.send((send_id,x)).await
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
                log::debug!(
                    "Got something from the network");
                if let None = block_opt {
                    panic!("invalid content received from the server");
                }
                let (_, msg) = block_opt.unwrap();
                log::debug!("Got a client message: {:?}", msg);
                let prop = match msg {
                    ClientMsg::NewBlock(p, _pl) => p,
                    _ => continue,
                };
                update_props(prop, &mut cx);
                while let Ok(Some((_, ClientMsg::NewBlock(p,_)))) = net_recv.try_next() {
                    update_props(p, &mut cx);
                }
                handle_new_blocks(c, &mut cx, now);
            } 
        }
        if cx.num_cmds > m as u128 {
            let now = SystemTime::now();
            statistics(now, start, cx.latency_map);
            return;
        }
    }
}

fn update_props(p: Propose, cx:&mut Context) {
    if p.round < cx.round {
        if cx.storage.is_delivered_by_hash(&p.block_hash) {
            log::warn!("Got a block {} from the past - {}", p.round, cx.round);
            return;
        } else {
            // Someone equivocated.
            panic!("equivocation detected");
        }
    }
    cx.future_msgs.insert(p.round, p);
}

// Handle future blocks
fn handle_new_blocks(c: &Client, cx: &mut Context, now: SystemTime) {
    while let Some(p) = cx.future_msgs.remove(&cx.round) {
        let b = p.block.clone().unwrap();
        cx.storage.add_delivered_block(b.clone());
        if !cx.storage.is_delivered_by_hash(&b.header.prev) {
            panic!("Do not have parent for this block {:?}, yet",b);
        }
        let commit_round = cx.round - c.num_faults;
        let commit_block = cx.storage.delivered_block_from_ht(commit_round)
            .expect("Must be in the height map");
        
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
        cx.round += 1;
    }
}