/// The core consensus module used for Sync HotStuff
/// 
/// The reactor reacts to all the messages from the network, and talks to the
/// clients accordingly.

use tokio::sync::mpsc::{Sender, Receiver};
use types::{Block, synchs::ProtocolMsg, Replica, Transaction};
use config::Node;
use super::{proposal::*, vote::on_vote};
use super::blame::*;
use super::context::Context;

pub async fn reactor(
    config:&Node,
    net_send: Sender<(Replica, ProtocolMsg)>,
    mut net_recv: Receiver<ProtocolMsg>,
    cli_send: Sender<Block>,
    mut cli_recv: Receiver<Transaction>
) {
    let mut cx = Context::new(config, net_send, cli_send);
    let block_size = config.block_size;
    let myid = config.id;
    loop {
        tokio::select! {
            pmsg_opt = net_recv.recv() => {
                // Received a protocol message
                let protmsg = match pmsg_opt {
                    None => break,
                    Some(x) => x,
                };
                // println!("Received protocol message: {:?}", protmsg);
                if let ProtocolMsg::NewProposal(mut p) = protmsg {
                    p.init();
                    println!("Received a proposal: {:?}", p);
                    let decision = on_receive_proposal(&p, &mut cx).await;
                    println!("Decision for the incoming proposal is {}", decision);
                }
                else if let ProtocolMsg::VoteMsg(v, mut p) = protmsg {
                    p.init();
                    println!("Received a vote for a proposal: {:?}", p);
                    on_vote(&v,p, &mut cx).await;
                }
                
            },
            tx_opt = cli_recv.recv() => {
                // We received a message from the client
                match tx_opt {
                    None => break,
                    Some(tx) => {
                        cx.storage.pending_tx.insert(crypto::hash::ser_and_hash(&tx),tx);
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
            do_propose(txs, &mut cx).await;
        } 
    }
}