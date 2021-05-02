use std::time::SystemTime;
use config::Client;
use types::artemis::{Block, ClientMsg, Payload, Transaction, UCRVote};
use tokio::sync::mpsc::{Receiver, channel};
use consensus::statistics;
use std::sync::Arc;
use util::codec::EnCodec;
use util::codec::Decodec;
use futures::{SinkExt, StreamExt};
use net::futures_manager::TlsClient;
use super::*;

type TxFactory = Receiver<Arc<Transaction>>;

/// Setup a concurrent thread that produces a stream of dummy transactions
/// so that the main reactor has a buffer of transactions always ready to send to the nodes
async fn setup_tx_factory(payload: usize) -> TxFactory {
    // Start with the sink implementation
    let (send, recv) = channel(util::CHANNEL_SIZE);
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
    recv
}

pub async fn start(
    c:Arc<Client>, 
    metric: u64,
    window: usize,
) {
    // Create the client network
    let mut client_network = TlsClient::<ClientMsg, Transaction>::new(c.root_cert.clone());

    // Create the client network
    let servers = c.net_map.clone();
    let (mut net_send,mut net_recv) = 
        client_network.setup(servers, EnCodec::new(), Decodec::new()).await;

    let payload = c.payload;
    let mut cx = Context::new(c.clone());
    let mut recv = setup_tx_factory(payload).await;
    let m = metric;
    cx.pending = window;
    cx.num_cmds = 0;
    let start = SystemTime::now();
    loop {
        tokio::select! {
            // Send transactions to the nodes
            tx_opt = recv.recv(), if cx.pending > 0 => {
                if let Some(x) = tx_opt {
                    let tx = x.as_ref();
                    net_send.send((c.num_nodes,x.clone())).await
                        .expect("Failed to send to the client");
                    let hash = crypto::hash::ser_and_hash(tx);
                    cx.time_map.insert(hash, SystemTime::now());
                    cx.pending -= 1;
                    log::debug!("Sending transaction to the leader");
                } else {
                    log::info!("TxFactory closed");
                    std::process::exit(0);
                }
            },
            // Got a vote message
            block_opt = net_recv.next() => {
                let now = SystemTime::now();
                log::debug!("Got something from the nodes");
                if let None = block_opt {
                    panic!("invalid content received from the server");
                }
                let (sender, msg) = block_opt.unwrap();
                log::debug!("Got a client message: {:?} from {}", msg, sender);
                match msg {
                    ClientMsg::NewBlock(v, block_vec) => try_new_round(v, block_vec, &mut cx, now).await,
                    _ => continue,
                };
                while let Ok(Some((_, ClientMsg::NewBlock(v,block_vec)))) = net_recv.try_next() {
                    try_new_round(v, block_vec, &mut cx, now).await;
                }
            } 
        }
        if cx.num_cmds > m as u128 {
            let now = SystemTime::now();
            statistics(now, start, cx.latency_map);
            return;
        }
    }
}

/// We got a new vote message. Check if we are in the correct round and then process it.
async fn try_new_round(v: UCRVote, 
    block_vec: Vec<(Block, Payload)>, 
    cx:&mut Context,
    ts: SystemTime,
) 
{
    if cx.round < v.round {
        log::debug!("We got a vote from the future");
        // TODO
        cx.future_msgs.insert(v.round, (v, block_vec));
        return;
    }
    if cx.round > v.round {
        log::warn!("We got a vote from a round that we have already processed for");
        return;
    }
    new_round(v, block_vec, cx, ts).await;
    while let Some((v, block_vec)) = cx.future_msgs.remove(&cx.round) {
        new_round(v, block_vec, cx, ts).await;
    }
}

/// Processing votes for the correct round
async fn new_round(v: UCRVote, 
    block_vec: Vec<(Block, Payload)>, 
    cx:&mut Context,
    ts: SystemTime,
) 
{
    for (b, _) in block_vec {
        cx.pending += b.blk.body.tx_hashes.len();
        cx.storage.add_delivered_block(Arc::new(b));
    }
    if v.round < cx.config.num_faults {
        // Nothing to commit
        cx.round += 1;
        return;
    }

    let mut com_hash = v.hash;
    while !cx.storage.is_committed_by_hash(&com_hash) {
        let b_rc = cx.storage.delivered_block_from_hash(&com_hash).expect("Trying to commit an undelivered block");
        cx.storage.add_committed_block(b_rc.clone());
        com_hash = b_rc.blk.header.prev;
        // For every committed block, update the statistics
        for tx_hash in &b_rc.blk.body.tx_hashes {
            if let Some(start) = cx.time_map.remove(tx_hash) {
                cx.num_cmds += 1;
                cx.latency_map.insert(tx_hash.clone(), (start, ts));
            }
        }
    }

    // At the end, move to the next round
    cx.round += 1;
}