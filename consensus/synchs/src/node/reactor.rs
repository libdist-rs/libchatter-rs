/// The core consensus module used for Sync HotStuff
/// 
/// The reactor reacts to all the messages from the network, and talks to the
/// clients accordingly.

use tokio::sync::mpsc::{
    UnboundedSender, 
    UnboundedReceiver
};
use types::{Replica, Transaction, synchs::ClientMsg, synchs::ProtocolMsg};
use config::Node;
use super::{
    commit::on_commit, 
    proposal::*, 
    vote::on_vote,
    context::Context,
};
use tokio_stream::StreamExt;
use std::sync::Arc;

pub async fn reactor(
    config:&Node,
    net_send: UnboundedSender<(Replica, Arc<ProtocolMsg>)>,
    mut net_recv: UnboundedReceiver<(Replica, ProtocolMsg)>,
    cli_send: UnboundedSender<Arc<ClientMsg>>,
    mut cli_recv: UnboundedReceiver<Transaction>
) {
    let d2 = std::time::Duration::from_millis(2*config.delta);
    log::debug!("Started timers");
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
                    Some((_, x)) => x,
                };
                log::debug!(
                    "Received protocol message: {:?}", protmsg);
                if let ProtocolMsg::NewProposal(p) = protmsg {
                    log::debug!(
                        "Received a proposal: {:?}", p);
                    let p = Arc::new(p);
                    let decision = on_receive_proposal(p.clone(), &mut cx).await;
                    log::debug!(
                        "Decision for the incoming proposal is {}", decision);
                    if decision {
                        cx.commit_queue.insert(p, d2);
                    }
                }
                else if let ProtocolMsg::VoteMsg(v,p) = protmsg {
                    log::debug!(
                        "Received a vote for a proposal: {:?}", v);
                    on_vote(v, p, &mut cx).await;
                }
            },
            tx_opt = cli_recv.recv() => {
                // We received a message from the client
                log::trace!(
                    "Got tx from the client: {:?}", tx_opt);
                let tx = match tx_opt {
                    None => break,
                    Some(x) => {
                        x
                    }
                };
                cx.storage.add_transaction(tx);
            },
            b_opt = cx.commit_queue.next(), if !cx.commit_queue.is_empty() => {
                // Got something from the timer
                match b_opt {
                    None => {
                        log::info!("Timer finished");
                    },
                    Some(Ok(b)) => {
                        log::debug!("Timer fired");
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
        if cx.storage.get_tx_pool_size() >= block_size && 
            cx.next_leader() == myid && 
            cx.cert_map.contains_key(&cx.last_seen_block.hash)
        {
            log::debug!("I {} am the leader and, I am proposing", cx.myid);
            let txs = cx.storage.cleave(block_size);
            let p = do_propose(txs, &mut cx).await;
            // Leader setting the timer now
            cx.commit_queue.insert(p, d2);
        }
    }
}