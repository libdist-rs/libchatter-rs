use crypto::hash::Hash;
use types::{ProtocolMsg, Replica};
use futures::SinkExt;
use super::context::Context;
use std::sync::Arc;

pub async fn handle_request(sender:Replica, req_id: u64, h: Hash, cx: &mut Context)
{
    log::debug!(target:"consensus", 
        "Got a request from {} for {:?}", sender, h);
    let p_arc = match cx.prop_chain.get(&h) {
        None => return,
        Some(x) => x.clone(),
    };
    let prop = p_arc.as_ref().clone();
    let blk = p_arc.block.clone().unwrap().as_ref().clone();
    let msg = ProtocolMsg::RawResponse(req_id, prop, blk);
    cx.net_send.send((sender, Arc::new(msg))).await.unwrap();
}