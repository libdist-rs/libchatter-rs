use types::Block;

use super::context::Context;
use std::sync::Arc;
use std::borrow::Borrow;

/// Commit this block and all its ancestors
pub async fn on_commit(b: Arc<Block>, cx:&mut Context) {
    // Check if we have already committed this block and its ancestors
    if cx.storage.committed_blocks_by_hash.contains_key(&b.hash) {
        return;
    }

    // Ship the block to the clients
    let ship = cx.cli_send.clone();
    let ship_b = b.clone();
    let payload = cx.payload;
    let ship_block = tokio::spawn(async move {
        let mut ship_b = (ship_b.borrow() as &Block).clone();
        ship_b.add_payload(payload);
        log::debug!(target:"consensus", "sending block: {:?}", ship_b);
        if let Err(e) = ship.send(Arc::new(ship_b)) {
            println!("Error sending the block to the client: {}", e);
            ()
        }
        log::debug!(target:"consensus", 
            "Committed block and sending it to the client now");
    });
    cx.last_committed_block_ht = b.header.height;
    cx.storage.committed_blocks_by_hash.insert(b.hash, b.clone());
    cx.storage.committed_blocks_by_ht.insert(b.header.height, b);
    ship_block.await.unwrap();
}