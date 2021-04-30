/// The core consensus module used for Opt Sync
/// 
/// The reactor reacts to all the messages from the network, and talks to the
/// clients accordingly.

use tokio::sync::mpsc::{
    UnboundedSender, 
    UnboundedReceiver
};
use types::optsync::{Replica, Transaction, ClientMsg, ProtocolMsg};
use config::Node;
use crate::node::{
    commit::on_commit, 
    proposal::do_propose,
    process::process_msg,
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
                process_msg(&mut cx, protmsg).await;
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
                        log::debug!("2Delta timer finished");
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
            do_propose(txs, &mut cx).await;
        }
    }
}