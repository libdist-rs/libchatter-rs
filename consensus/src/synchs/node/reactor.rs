/// The core consensus module used for Sync HotStuff
/// 
/// The reactor reacts to all the messages from the network, and talks to the
/// clients accordingly.

use crate::{
    Sender, 
    Receiver
};
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
    vote::on_vote
};
use super::context::Context;
use tokio_stream::StreamExt;

pub async fn reactor(
    config:&Node,
    net_send: Sender<(Replica, ProtocolMsg)>,
    net_recv: Receiver<ProtocolMsg>,
    cli_send: Sender<Block>,
    cli_recv: Receiver<Transaction>
) {
    let d2 = std::time::Duration::from_millis(2*config.delta);
    let mut queue = tokio_util::time::DelayQueue::new();
    println!("Started timers");
    let mut cx = Context::new(config, net_send, cli_send);
    let block_size = config.block_size;
    let myid = config.id;
    loop {
        tokio::select! {
            pmsg_opt = net_recv.recv() => {
                // Received a protocol message
                let protmsg = match pmsg_opt {
                    Err(_e) => break,
                    Ok(x) => x,
                };
                // println!("Received protocol message: {:?}", protmsg);
                if let ProtocolMsg::NewProposal(mut p) = protmsg {
                    p.init();
                    // println!("Received a proposal: {:?}", p);
                    let decision = on_receive_proposal(&p, &mut cx).await;
                    // println!("Decision for the incoming proposal is {}", decision);
                    if decision {
                        queue.insert(p.new_block, d2);
                    }
                }
                else if let ProtocolMsg::VoteMsg(v, mut p) = protmsg {
                    p.init();
                    // println!("Received a vote for a proposal: {:?}", p);
                    on_vote(&v,p, &mut cx).await;
                }
            },
            tx_opt = cli_recv.recv() => {
                // We received a message from the client
                // println!("Got a message from the client");
                match tx_opt {
                    Err(_e) => break,
                    Ok(tx) => {
                        cx.storage.pending_tx.insert(crypto::hash::ser_and_hash(&tx),tx);
                    }
                }
            },
            b_opt = queue.next(), if !queue.is_empty() => {
                // Got something from the timer
                match b_opt {
                    None => {
                        println!("Timer finished");
                    },
                    Some(Ok(b)) => {
                        // println!("Timer fired");
                        on_commit(b.into_inner(), &mut cx).await;
                    },
                    Some(Err(_e)) => {
                        println!("Timer misfired: {}", _e);
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
            // println!("I {} am the leader and, I am proposing", cx.myid);
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
            let p = do_propose(txs, &mut cx).await;
            // println!("Leader setting the timer now");
            // if let Err(e) = timein.send(InMsg::NewTimer(p.new_block.clone())).await {
            //     println!("Failed to send block to the timer thread: {}", e);
            // }
            queue.insert(p.new_block, d2);
        } 
    }
}