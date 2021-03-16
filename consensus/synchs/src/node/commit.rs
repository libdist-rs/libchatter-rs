use types::{synchs::ClientMsg, Payload, synchs::Propose};

use super::context::Context;
use std::sync::Arc;
// use futures::SinkExt;

/// Commit this block and all its ancestors
pub async fn on_commit(p: Arc<Propose>, cx:&mut Context) {
    let b = p.block.as_ref().unwrap();
    // Check if we have already committed this block and its ancestors
    if cx.storage.is_committed_by_hash(&b.hash) {
        return;
    }

    // Ship the block to the clients
    let ship = cx.cli_send.clone();
    let payload = cx.payload;
    let ship_b = b.clone();
    let ship_block = tokio::spawn(async move {
        let payload = Payload::with_payload(payload);
        let msg = ClientMsg::RawNewBlock(
            ship_b.as_ref().clone(), payload);
        log::debug!(
            "sending msg: {:?} to the client", msg);
        if let Err(e) = ship.send(Arc::new(msg)) {
            println!("Error sending the block to the client: {}", e);
            ()
        }
        log::debug!(
            "Committed block and sending it to the client now");
    });
    cx.last_committed_block_ht = b.header.height;
    cx.storage.add_committed_block(b.clone());
    ship_block.await.unwrap();
}