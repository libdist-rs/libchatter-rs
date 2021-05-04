use crypto::hash::Hash;
use types::apollo::{ProtocolMsg, Replica};
use super::context::Context;
use std::sync::Arc;

pub async fn on_recv_request(sender:Replica, req_id: u64, h: Hash, cx: &mut Context)
{
    log::debug!(
        "Got a request from {} for {:?}", sender, h);
    let p_arc = match cx.prop_chain_by_hash.get(&h) {
        None => return,
        Some(x) => x.clone(),
    };
    let prop = p_arc.as_ref().clone();
    let blk = p_arc.block.clone().unwrap().as_ref().clone();
    let msg = ProtocolMsg::RawResponse(req_id, prop, blk);
    cx.send(sender, Arc::new(msg)).await;
}

pub async fn do_request(b_hash: Hash, to: Replica, cx:&mut Context) {
    // I don't have the chain for this. Ask chain from the sender
    let msg = Arc::new(ProtocolMsg::Request(cx.req_ctr, b_hash));
    cx.send(to, msg).await;
}