/// The core consensus module used for Apollo
/// 
/// The reactor reacts to all the messages from the network, and talks to the
/// clients accordingly.

use tokio::sync::mpsc::{
    unbounded_channel, 
    UnboundedSender, 
    UnboundedReceiver
};
use types::{Block, ClientMsg, Payload, Propose, ProtocolMsg, Replica, Transaction};
use config::Node;
use super::{blame::*, context::Context, proposal::*, request::{handle_request, handle_response}};
use std::{
    sync::Arc, 
    borrow::Borrow,
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
        let cli_send = cli_send_p;
        loop {
            let prop_arc = recv.recv().await.unwrap();
            let payload = Payload::with_payload(pl_size);
            let bl = (prop_arc
                .block
                .clone()
                .unwrap()
                .borrow() as &Block)
                .clone();
            let prop = (prop_arc
                .borrow() as &Propose)
                .clone();
            cli_send.send(Arc::new(ClientMsg::RawNewBlock(prop, bl, payload))).unwrap();
        }
    });
    loop {
        tokio::select! {
            pmsg_opt = net_recv.recv() => {
                // Received a protocol message
                if let None = pmsg_opt {
                    log::error!(target:"node", 
                        "Protocol message channel closed");
                    std::process::exit(0);
                }
                let (sender, pmsg) = pmsg_opt.unwrap();
                match pmsg {
                    ProtocolMsg::NewProposal(p) => {
                        on_receive_proposal(Arc::new(p), &mut cx).await;
                    },
                    ProtocolMsg::Blame(v) => {
                        on_receive_blame(v, &mut cx).await;
                    },
                    ProtocolMsg::Relay(p) => {
                        on_relay(sender, p, &mut cx).await;
                    }
                    ProtocolMsg::Request(rid, h) => {
                        handle_request(sender, rid, h, &cx).await;
                    },
                    ProtocolMsg::Response(rid, p) => {
                        handle_response(rid, p, &mut cx).await;
                    }
                    _ => {},
                };
            },
            tx_opt = cli_recv.recv() => {
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