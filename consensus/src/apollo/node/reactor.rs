/// The core consensus module used for Apollo
/// 
/// The reactor reacts to all the messages from the network, and talks to the
/// clients accordingly.

// use tokio::sync::mpsc::{Sender, Receiver};
use crate::{Sender, Receiver};
use types::{Block, ProtocolMsg, Replica, Transaction};
use config::Node;
use super::{proposal::*};
use super::blame::*;
use super::context::Context;



pub async fn reactor(
    config:&Node,
    net_send: Sender<(Replica, ProtocolMsg)>,
    net_recv: Receiver<ProtocolMsg>,
    cli_send: Sender<Block>,
    cli_recv: Receiver<Transaction>,
    is_client_apollo_enabled: bool,
) {
    let mut cx = Context::new(config, net_send, cli_send);
    cx.is_client_apollo_enabled = is_client_apollo_enabled;
    let block_size = config.block_size;
    let myid = config.id;
    loop {
        tokio::select! {
            pmsg_opt = net_recv.recv() => {
                // Received a protocol message
                match pmsg_opt {
                    Err(_e) => break,
                    Ok(ProtocolMsg::NewProposal(p)) => {
                        on_receive_proposal(&p, &mut cx).await;
                    },
                    Ok(ProtocolMsg::Blame(v)) => {
                        on_receive_blame(v, &mut cx).await;
                    }
                    _ => {},
                };
            },
            tx_opt = cli_recv.recv() => {
                // We received a message from the client
                match tx_opt {
                    Err(_e) => break,
                    Ok(tx) => {
                        cx.storage.pending_tx.insert(crypto::hash::ser_and_hash(&tx),tx);
                    }
                }
            }
        }
        // Do we have sufficient commands, and are we the next leader?
        if cx.storage.pending_tx.len() >= block_size && 
            cx.next_leader() == myid 
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