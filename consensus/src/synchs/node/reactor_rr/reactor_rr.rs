/// The core round-robin consensus reactor for Sync HotStuff
/// 
/// The reactor reacts to all the messages from the network, and talks to the
/// clients accordingly.

use tokio::sync::mpsc::{
    UnboundedSender, 
    UnboundedReceiver
};
use types::{Replica, Transaction, synchs::ClientMsg, synchs::ProtocolMsg};
use config::Node;
use super::{commit::on_commit, context::Context, phase::Phase, proposal::*, status::{do_status, on_recv_status}, vote::*};
use tokio_stream::StreamExt;
use std::{sync::Arc, time::Duration};

pub async fn reactor_rr(
    config:&Node,
    net_send: UnboundedSender<(Replica, Arc<ProtocolMsg>)>,
    mut net_recv: UnboundedReceiver<(Replica, ProtocolMsg)>,
    cli_send: UnboundedSender<Arc<ClientMsg>>,
    mut cli_recv: UnboundedReceiver<Transaction>
) {
    let d2 = std::time::Duration::from_millis(2*config.delta);
    log::debug!(target:"consensus", "Started timers");
    let mut cx = Context::new(config, net_send, cli_send);
    // Start event loop
    loop {
        tokio::select! {
            pmsg_opt = net_recv.recv() => {
                // Received a protocol message
                let protmsg = match pmsg_opt {
                    None => break,
                    Some((_, x)) => x,
                };
                log::debug!("Received protocol message: {:?}", protmsg);
                match protmsg {
                    ProtocolMsg::NewProposal(p) => {
                        log::debug!("Received a proposal message: {:?}", p);
                        let p = Arc::new(p);
                        let decision = on_receive_proposal(p.clone(), &mut cx).await;
                        log::debug!("Decision for the incoming proposal is {}", decision);
                        if decision {
                            cx.commit_queue.insert(p, d2);
                        }
                    }
                    ProtocolMsg::VoteMsg(v,p) => {
                        log::debug!(target:"consensus", 
                        "Received a vote for a proposal: {:?}", v);
                        on_vote(v, p, &mut cx).await;
                    }
                    ProtocolMsg::ChangeView(v,c) => {
                        log::debug!("Received a quit view message {:?}", c);
                        on_recv_quit_view(v, c, &mut cx).await;
                    }
                    ProtocolMsg::StatusMsg(cert) => {
                        on_recv_status(cert, &mut cx).await;
                    }
                    other => {
                        log::debug!("Not handling {:?}", other);
                    }
                }
            },
            tx_opt = cli_recv.recv() => {
                // We received a message from the client
                log::trace!(target:"consensus", 
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
            },
            event = cx.event_queue.next(), if !cx.event_queue.is_empty() => {
                log::debug!("Triggered event {:?}", event);
                let b = match event {
                    None => continue,
                    Some(Ok(b)) => {
                        b.into_inner()
                    },
                    Some(Err(e)) => {
                        log::warn!("Event queue misfired: {}", e);
                        continue;
                    }
                };
                if b == Phase::Status {
                    // We finished the \Delta wait after sending the quit view
                    // message
                    // Send the Status message and wait for the next proposal
                    do_status(&mut cx).await;
                } else if b == Phase::StatusWait {
                    // I as a leader finished waiting for the status
                    // Start proposing the next block if ready
                    log::debug!("I {} am now the leader", cx.myid);
                    cx.phase = Phase::Propose;
                } 
            }
        }
        try_propose(config, &mut cx, &d2).await;
    }
}

async fn try_propose(c: &Node, cx: &mut Context, d2: &Duration) {
    // Do we have sufficient commands, and are we the next leader?
    // Also, do we have sufficient votes?
    let is_pool_suff = cx.storage.get_tx_pool_size() >= c.block_size; 
    let am_i_leader = cx.next_leader() == c.id;
    let is_no_cert = cx.cert_map.contains_key(&cx.last_seen_block.hash);
    let is_wrong_phase = cx.phase == Phase::Propose;

    // log::debug!("Not proposing because: {} {} {} {}", is_pool_suff, am_i_leader, is_no_cert, is_wrong_phase);

    if is_pool_suff && am_i_leader && 
        is_no_cert && is_wrong_phase
    {
        log::debug!("I {} am the leader and, I am proposing", cx.myid);
        let txs = cx.storage.cleave(c.block_size);
        let p = do_propose(txs, cx).await;
        // Leader setting the timer now
        cx.commit_queue.insert(p, *d2);
    }
}