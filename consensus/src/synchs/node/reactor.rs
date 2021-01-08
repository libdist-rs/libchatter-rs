/// The core consensus module used for Sync HotStuff
/// 
/// The reactor reacts to all the messages from the network, and talks to the
/// clients accordingly.

use tokio::sync::mpsc::{
    UnboundedSender, 
    UnboundedReceiver
};
use tokio_util::time::DelayQueue;
use types::{
    Block, 
    synchs::ProtocolMsg, 
    Replica, 
    Transaction
};
use config::Node;
use super::{
    commit::on_commit, 
    proposal::*, 
    vote::on_vote,
    context::Context,
};
use tokio_stream::StreamExt;
use std::sync::Arc;
use std::borrow::Borrow;

pub async fn reactor(
    config:&Node,
    net_send: UnboundedSender<(Replica, Arc<ProtocolMsg>)>,
    mut net_recv: UnboundedReceiver<(Replica, ProtocolMsg)>,
    cli_send: UnboundedSender<Arc<Block>>,
    mut cli_recv: UnboundedReceiver<Transaction>
) {
    let d2 = std::time::Duration::from_millis(2*config.delta);
    let mut queue:DelayQueue<Arc<Block>> = tokio_util::time::DelayQueue::new();
    log::debug!(target:"consensus", "Started timers");
    let mut cx = Context::new(config, net_send, cli_send);
    let block_size = config.block_size;
    let myid = config.id;
    // Start event loop
    loop {
        tokio::select! {
            pmsg_opt = net_recv.recv() => {
                // Received a protocol message
                let protmsg = match pmsg_opt {
                    None => break,
                    Some((_, x)) => {
                        x.init()
                    },
                };
                log::debug!(target:"consensus", 
                    "Received protocol message: {:?}", protmsg);
                if let ProtocolMsg::NewProposal(p) = &protmsg {
                    log::debug!(target:"consensus", 
                        "Received a proposal: {:?}", p);
                    let decision = on_receive_proposal(p, &mut cx).await;
                    log::debug!(target:"consensus", 
                        "Decision for the incoming proposal is {}", decision);
                    if decision {
                        queue.insert(cx.last_seen_block.clone(), d2);
                    }
                }
                else if let ProtocolMsg::VoteMsg(v, p) = protmsg.borrow() {
                    log::debug!(target:"consensus", 
                        "Received a vote for a proposal: {:?}", p);
                    on_vote(&v,p, &mut cx).await;
                }
            },
            tx_opt = cli_recv.recv() => {
                // We received a message from the client
                log::debug!(target:"consensus", 
                    "Got a message from the client");
                let tx = match tx_opt {
                    None => break,
                    Some(x) => {
                        x
                    }
                };
                cx.storage.pending_tx.insert(crypto::hash::ser_and_hash(&tx),tx);
            },
            b_opt = queue.next(), if !queue.is_empty() => {
                // Got something from the timer
                match b_opt {
                    None => {
                        log::info!(target:"consensus", "Timer finished");
                    },
                    Some(Ok(b)) => {
                        log::debug!(target:"consensus", "Timer fired");
                        on_commit(b.into_inner(), &mut cx).await;
                    },
                    Some(Err(e)) => {
                        log::warn!("Timer misfired: {}", e);
                        continue;
                    }
                }
            }
        }
        // Do we have sufficient commands, and are we the next leader?
        // Also, do we have sufficient votes?
        if cx.storage.pending_tx.len() >= block_size && 
            cx.next_leader() == myid && 
            cx.cert_map.contains_key(&cx.last_seen_block.hash)
        {
            log::debug!("I {} am the leader and, I am proposing", cx.myid);
            let mut txs = Vec::with_capacity(block_size);
            for _i in 0..block_size {
                let tx = match cx.storage.pending_tx.pop_front() {
                    Some((_hash, trans)) => trans,
                    None => {
                        panic!("Dequeued when tx pool was not block size");
                    },
                };
                txs.push(tx);
            }
            let _p = do_propose(txs, &mut cx).await;
            // Leader setting the timer now
            queue.insert(cx.last_seen_block.clone(), d2);
        }
    }
}