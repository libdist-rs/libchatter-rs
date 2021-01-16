/// The core consensus module used for Apollo
/// 
/// The reactor reacts to all the messages from the network, and talks to the
/// clients accordingly.

use futures::channel::mpsc::{
    UnboundedReceiver,
    UnboundedSender,
    unbounded as unbounded_channel,
};
use futures::{StreamExt, SinkExt};
// use tokio::sync::mpsc::{
//     unbounded_channel, 
//     UnboundedSender, 
//     UnboundedReceiver
// };
use types::{ClientMsg, Payload, ProtocolMsg, Replica, Transaction};
use config::Node;
use super::{context::Context, proposal::*,message::*};
use std::{
    sync::Arc, 
};

pub async fn reactor(
    config:&Node,
    is_client_apollo_enabled: bool,
    net_send: UnboundedSender<(Replica, Arc<ProtocolMsg>)>,
    mut net_recv: UnboundedReceiver<(Replica, ProtocolMsg)>,
    cli_send: UnboundedSender<Arc<ClientMsg>>,
    mut cli_recv: UnboundedReceiver<Transaction>,
) {
    // Optimization to improve latency when the payloads are high
    let (send, mut recv) = unbounded_channel();

    let mut cx = Context::new(config, net_send, send);
    cx.is_client_apollo_enabled = is_client_apollo_enabled;

    let block_size = config.block_size;
    let myid = config.id;
    let pl_size = config.payload;

    let cli_send_p = cli_send;
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.spawn(async move {
        let mut cli_send = cli_send_p;
        loop {
            let prop_arc = recv.next().await.unwrap();
            let payload = Payload::with_payload(pl_size);
            let prop = prop_arc.as_ref().clone();
            let bl = prop.block.as_ref().unwrap().as_ref().clone();
            cli_send.send(Arc::new(ClientMsg::RawNewBlock(prop, bl, payload))).await.unwrap();
        }
    });
    loop {
        tokio::select! {
            pmsg_opt = net_recv.next() => {
                // Received a protocol message
                if let None = pmsg_opt {
                    log::error!(target:"node", 
                        "Protocol message channel closed");
                    std::process::exit(0);
                }
                let (sender, pmsg) = pmsg_opt.unwrap();
                handle_message(sender, pmsg, &mut cx);
                while let Ok(Some((sender, pmsg))) = net_recv.try_next() {
                    handle_message(sender, pmsg, &mut cx);
                }
                process_message(&mut cx).await;
            },
            tx_opt = cli_recv.next() => {
                // We received a message from the client
                match tx_opt {
                    None => break,
                    Some(tx) => {
                        cx.storage.add_transaction(tx);
                    }
                }
            }
        }
        // Do we have sufficient commands, and are we the next leader?
        if cx.storage.get_tx_pool_size() >= block_size && 
            cx.next_leader() == myid 
        {
            log::debug!(target:"consensus",
                "I {} am the leader and, I am proposing", cx.myid);
            let txs = cx.storage.cleave(block_size);
            do_propose(txs, &mut cx).await;
        } 
    }
}