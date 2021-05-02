use crypto::hash::Hash;
use types::artemis::{ProtocolMsg, Replica};
use super::context::Context;
use std::sync::Arc;

/// This function is called when some node requests blocks because it doesn't know the chain
pub async fn handle_request(sender:Replica, req_id: u64, h: Hash, cx: &mut Context)
{
    log::debug!("Got a request from {} for {:x?}", sender, h);
    let is_delivered = cx.storage.is_delivered_by_hash(&h);
    if !is_delivered && !cx.undelivered_blocks.contains_key(&h) {
        // I don't have the chain to respond to this request
        return;
    };
    let blk = if is_delivered {
        cx.storage.delivered_block_from_hash(&h).unwrap().as_ref().clone()
    } else {
        cx.undelivered_blocks.get(&h).unwrap().clone()
    };
    let msg = Arc::new(ProtocolMsg::RawResponse(req_id, blk));
    cx.send(sender, msg).await;
}

/// Request this block
pub async fn do_request(cx:&mut Context, sender:Replica, h: Hash) {
    log::debug!("Requesting hash: {:x?}", h);
    let msg = Arc::new(ProtocolMsg::Request(cx.req_ctr, h));
    cx.send(sender, msg).await;
}