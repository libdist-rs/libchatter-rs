/// The core consensus module used for Apollo
/// 
/// The reactor reacts to all the messages from the network, and talks to the
/// clients accordingly.

use tokio::sync::mpsc::{Receiver, Sender, channel};
// use crate::{Sender, Receiver};
// use crossfire::mpsc::bounded_tx_future_rx_blocking;
use types::{Block, ProtocolMsg, Replica, Transaction};
use config::Node;
use super::{proposal::*};
use super::blame::*;
use super::context::Context;
use std::{sync::Arc, borrow::Borrow};

pub async fn reactor(
    config:&Node,
    net_send: Sender<(Replica, Arc<ProtocolMsg>)>,
    mut net_recv: Receiver<Arc<ProtocolMsg>>,
    cli_send: Sender<Arc<Block>>,
    mut cli_recv: Receiver<Arc<Transaction>>,
    is_client_apollo_enabled: bool,
) {
    // let cli_ship_p = cli_send.clone();
    let (send, mut recv) = channel(util::CHANNEL_SIZE);
    let mut cx = Context::new(config, net_send, send);
    cx.is_client_apollo_enabled = is_client_apollo_enabled;
    let block_size = config.block_size;
    let myid = config.id;
    let pl_size = config.payload;
    let cli_send_p = cli_send.clone();
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    rt.spawn(async move {
        let cli_send = cli_send_p;
        loop {
            let mut x = recv.recv().await.unwrap();
            x.add_payload(pl_size);
            cli_send.send(Arc::new(x)).await.unwrap();
            // tokio::runtime::Handle::
        }
    });
    loop {
        tokio::select! {
            pmsg_opt = net_recv.recv() => {
                // Received a protocol message
                if let None = pmsg_opt {
                    break;
                }
                let pmsg = pmsg_opt.unwrap();
                let bor:&types::ProtocolMsg = pmsg.borrow();
                match bor {
                    ProtocolMsg::NewProposal(p) => {
                        // let p_arc = Arc::new(p.clone());
                        on_receive_proposal(p, &mut cx).await;
                    },
                    ProtocolMsg::Blame(v) => {
                        on_receive_blame(v.clone(), &mut cx).await;
                    },
                    _ => {},
                };
            },
            tx_opt = cli_recv.recv() => {
                // We received a message from the client
                match tx_opt {
                    None => break,
                    Some(tx) => {
                        cx.storage.pending_tx.insert(crypto::hash::ser_and_hash(tx.borrow() as &Transaction),(tx.borrow() as &Transaction).clone());
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