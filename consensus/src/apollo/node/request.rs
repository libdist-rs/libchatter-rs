use crypto::hash::Hash;
use types::{Propose, ProtocolMsg, Replica};

use super::{context::Context, proposal::on_receive_proposal};
use std::sync::Arc;

pub async fn handle_request(sender:Replica, req_id: u64, h: Hash, cx: &Context)
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
    cx.net_send.send((sender, Arc::new(msg))).unwrap();
}

pub async fn handle_response(sender:Replica, _req_id: u64, p: Propose, cx: &mut Context) {
    log::debug!(target:"consensus", 
        "Got response for {} with {:?}", _req_id, p);
    let mut bhash = p.block_hash;
    // If we have not processed it yet
    if cx.prop_waiting.remove(&bhash) {
        on_receive_proposal(sender,p, cx).await;
    } 
    while let Some(p_new) = cx.prop_waiting_parent.remove(&bhash) {
        bhash = p_new.block_hash;
        on_receive_proposal(sender, p_new, cx).await;
    }
    // Process pending proposals if any

}