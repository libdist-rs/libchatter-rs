use crypto::hash::Hash;
use types::{Propose, ProtocolMsg, Replica};

use super::{context::Context, proposal::on_receive_proposal};
use std::sync::Arc;

pub async fn handle_request(sender:Replica, req_id: u64, h: Hash, cx: &Context)
{
    log::debug!(target:"consensus", 
        "Got a request from {} for {:?}", sender, h);
    let p_arc = cx.prop_map.get(&h).unwrap().clone();
    let prop = p_arc.as_ref().clone();
    let blk = p_arc.block.clone().unwrap().as_ref().clone();
    let msg = ProtocolMsg::RawResponse(req_id, prop, blk);
    cx.net_send.send((sender, Arc::new(msg))).unwrap();
}

pub async fn handle_response(_req_id: u64, p: Propose, cx: &mut Context) {
    log::debug!(target:"consensus", 
        "Got response for {} with {:?}", _req_id, p);
    // If we have not processed it yet
    if cx.waiting.contains(&p.block_hash) {
        on_receive_proposal(Arc::new(p), cx).await;
    }
}