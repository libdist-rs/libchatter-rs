/// The core consensus module used for Artemis
/// 
/// The reactor reacts to all the messages from the network, and talks to the
/// clients accordingly.

use futures::channel::mpsc::unbounded as unbounded_channel;
use futures::{StreamExt, SinkExt};
use types::artemis::{ClientMsg, Payload};
use config::Node;
use super::{
    context::Context, 
    buffer_message, 
    process_message, 
    do_new_block,
};
use std::sync::Arc;
use crate::{NetSend,NetRecv,ClientSend,ClientRecv, node::round_vote::try_round_vote};

pub async fn reactor(
    config:&Node,
    is_client_apollo_enabled: bool,
    net_send: NetSend,
    mut net_recv: NetRecv,
    cli_send: ClientSend,
    mut cli_recv: ClientRecv,
) {
    // Optimization to improve latency when the payloads are high
    let (send, mut recv) = unbounded_channel();

    let mut cx = Context::new(config, net_send, send, is_client_apollo_enabled);
    let block_size = config.block_size;
    let myid = config.id;
    let pl_size = config.payload*config.block_size;
    let cli_send_p = cli_send;

    let payload_adder = async move {
        let mut cli_send = cli_send_p;
        loop {
            let msg_arc = recv.next().await.unwrap().as_ref().clone();
            let msg = match msg_arc {
                ClientMsg::RawNewBlock(v, block_vec) => {
                    let block_vec = block_vec.into_iter().map(|(b, _pl)| {
                        let payload = Payload::with_payload(pl_size);
                        (b, payload)
                    }).collect();
                    ClientMsg::RawNewBlock(v, block_vec)
                },
                _ => continue,
            };
            cli_send.send(Arc::new(msg)).await.unwrap();
        };
    };
    #[cfg(feature="parallel")]
    let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
    #[cfg(not(feature="parallel"))]
    let rt = tokio::runtime::Handle::current();
    rt.spawn(payload_adder);
    loop {
        tokio::select! {
            // Received a protocol message
            pmsg_opt = net_recv.next() => {
                if let None = pmsg_opt {
                    log::error!(
                        "Protocol message channel closed");
                    std::process::exit(0);
                }
                let (sender, pmsg) = pmsg_opt.unwrap();
                // So basically, we extract all currently available messages and then replay them in order
                buffer_message(sender, pmsg, &mut cx);
                while let Ok(Some((sender, pmsg))) = net_recv.try_next() {
                    buffer_message(sender, pmsg, &mut cx);
                }
                process_message(&mut cx).await;
            },
            // Received a client message
            tx_opt = cli_recv.next() => {
                // We received a message from the client
                match tx_opt {
                    None => break,
                    Some(tx) => cx.storage.add_transaction(tx),
                }
            }
        }
        // Do we have sufficient commands, and are we the view leader?
        if cx.storage.get_tx_pool_size() >= block_size && 
            cx.view_leader == myid 
        {
            log::debug!(
                "I {} am the view leader and, I am proposing a block", cx.myid());
            let txs = cx.storage.cleave(block_size);
            do_new_block(txs, &mut cx).await;
        }
        // Can I start the UCR process?
        try_round_vote(&mut cx).await;
    }
}