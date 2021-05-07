use types::apollo::{Propose, ProtocolMsg, Replica};
use super::*;
use std::sync::Arc;
use futures::SinkExt;

pub async fn process_message(cx:&mut Context) 
{
    log::debug!("Handling proposals {:?}", cx.prop_buf);
    while let Some((sender, p)) = cx.prop_buf.pop_front() {
        delivery_check(sender, p, cx).await;
    }
    log::debug!("Handling relays {:?}", cx.relay_buf);
    while let Some((sender, p)) = cx.relay_buf.pop_front() {
        delivery_check(sender, p, cx).await;
    }
    log::debug!("Handling others: {:?}", cx.other_buf);
    while let Some((sender, pmsg)) = cx.other_buf.pop_front() {
        match pmsg {
            ProtocolMsg::Request(rid, h) => {
                on_recv_request(sender, rid, h, cx).await;
            }
            ProtocolMsg::Blame(v) => {
                on_receive_blame(v, cx).await;
            }
            _x => {
                debug_assert!(
                    if let ProtocolMsg::NewProposal(_) = _x {false} else{ true });
                debug_assert!(
                    if let ProtocolMsg::Response(_,_) = _x {false} else {true}
                );
                debug_assert!(if let ProtocolMsg::Relay(_) = _x {false} else {true});
            },
        };
    }

    while let Some((sender, p)) = cx.future_msgs.remove(&cx.round()) {
        delivery_check(sender, p, cx).await;
    }
}

pub fn handle_message(sender: Replica, message: ProtocolMsg, cx: &mut Context) {
    match message {
        ProtocolMsg::Response(_, p) => cx.prop_buf.push_back((sender, p)),
        ProtocolMsg::NewProposal(p) => cx.prop_buf.push_back((sender, p)),
        ProtocolMsg::Relay(p) => cx.relay_buf.push_back((sender, p)),
        x => cx.other_buf.push_back((sender, x)),
    }
}

pub async fn delivery_check(sender:Replica, p: Propose, cx: &mut Context) {
    // Check if the proposals are already processed
    if cx.prop_chain_by_round.contains_key(&p.round) {
        log::debug!("Already handled {:?} before", p);
        return;
    }

    // Check if the parents are delivered
    let parent_hash = p.block.as_ref().map(|b| b.header.prev);
    if parent_hash.is_none() {
        log::debug!(
            "Block unknown: {:?}", p.block_hash);
        let msg = Arc::new(ProtocolMsg::Request(cx.req_ctr, p.block_hash));
        cx.prop_waiting.insert(p.block_hash, p);
        cx.net_send.send((sender, msg)).await.unwrap();
        return;
    }
    debug_assert!(parent_hash.is_some());

    let parent_hash = parent_hash.unwrap();

    if !cx.storage.is_delivered_by_hash(&parent_hash) {
        let msg = Arc::new(ProtocolMsg::Request(cx.req_ctr, parent_hash));
        cx.storage.add_delivered_block(p.block.clone().unwrap());
        cx.prop_waiting_parent.insert(parent_hash, p);
        cx.net_send.send((sender,msg)).await.unwrap();
        return;
    }

    // By now all parents and the current block is delivered
    debug_assert!(cx.storage.is_delivered_by_hash(&parent_hash));

    // Mark this block as delivered, since all its parents are delivered
    let block = p.block.clone().unwrap();
    log::debug!(
        "Block {} is delivered", block.header.height);
    cx.storage.add_delivered_block(block);

    let mut block_hash = p.block_hash;
    if cx.round() < p.round {
        cx.future_msgs.insert(p.round, (sender, p));
    } else {
        try_receive_proposal(p, sender, cx).await;
    }
    cx.prop_waiting.remove(&block_hash);

    while let Some(mut p_new) = cx.prop_waiting_parent.remove(&block_hash) {
        block_hash = p_new.block_hash;
        p_new.block = Some(cx.storage.delivered_block_from_hash(&block_hash).unwrap());
        if cx.round() < p_new.round {
            cx.future_msgs.insert(p_new.round, (sender, p_new));
        } else {
            try_receive_proposal(p_new, sender, cx).await;
        }
    }
}