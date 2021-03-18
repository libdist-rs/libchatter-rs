use types::{Block, Propose, ProtocolMsg, Transaction, WireReady};
use super::{
    context::Context, 
    commit::on_finish_propose
};
use std::sync::Arc;
use futures::SinkExt;

/// Called when a proposal is received due to:
/// - A block getting delivered
/// - Receiving a proposal directly
///
/// Note: This will only execute once all the block for of the proposal is delivered
pub async fn on_receive_proposal(p: Propose, cx: &mut Context) {
    log::debug!(
        "Handling proposal: {:?}", p);

    let block = p.block.clone().unwrap();

    // Check if the hash in the proposal is for the correct block
    if crypto::hash::ser_and_hash(block.as_ref()) != p.block_hash {
        log::warn!(
            "Invalid hash in the proposal");
        return;
    }

    // 1) Is it correctly signed?
    let pk = cx.pub_key_map.get(&block.header.author).unwrap(); 
    let bytes = util::io::to_bytes(block.as_ref());
    if !pk.verify(&bytes, &p.proof) {
        println!("Block verification failed.");
        return;
    }

    // Is this is an equivocation?
    //
    // We check for equivocation by checking that if we already have a block at
    // this height. If there is one, it must be an equivocation. Because, if
    // this was a repeated block, we must have returned from the previous check
    // for repeated blocks.
    if let Some(x) = cx.storage.delivered_block_from_ht(block.header.height) {
        if (x.hash != block.hash) && x.header.author == block.header.author {
            log::warn!(
                "Equivocation detected: {:?}, {:?}", cx.storage.delivered_block_from_ht(block.header.height), block);
            return;
        }
    }

    on_new_valid_proposal(Arc::new(p), cx).await;
}

/// Called once for every proposal of a height
///
/// Before calling, ensure that the block for this proposal and its parents are
/// delivered
pub async fn on_new_valid_proposal(p: Arc<Propose>, cx:&mut Context) {
    let block = p.block.as_ref().unwrap();

    // Remove transactions from the pool
    cx.storage.clear(&block.body.tx_hashes);

    // Commit any possible blocks and process proposal
    //
    // Forward only the hash as we are not the proposer
    on_finish_propose(p, false, cx).await;
}

/// Creates a block using the last seen block as the parent
/// Then adds the block as delivered
pub async fn do_propose(txs: Vec<Arc<Transaction>>, cx: &mut Context) {
    // Get the parent
    let parent = cx.last_seen_block.as_ref();

    // Create a block
    let mut new_block = Block::with_tx(txs);
    new_block.header.prev = parent.hash;
    new_block.header.author = cx.myid; 
    new_block.header.height =  parent.header.height+1;

    // Finally, compute the hash
    let new_block = new_block.init();

    let serialized_block = util::io::to_bytes(&new_block);

    // Create a proposal
    let mut p = Propose::new(new_block.hash);
    p.proof = cx.my_secret_key.sign(&serialized_block)
        .expect("failed to sign the proposal");
    p.block_hash = new_block.hash; // Set proposal hash
    p.block = Some(Arc::new(new_block.clone()));

    let p = p;
    let block = p.block.clone().unwrap();

    let ship_p = p.clone();
    // The leader broadcasts the transaction
    if let Err(e) = cx.net_send.send(
        (cx.num_nodes, Arc::new(
            ProtocolMsg::RawNewProposal(ship_p,new_block)
        ))
    ).await {
        log::warn!(
            "Server channel closed with error: {}", e);
    };

    log::debug!(
        "Proposing block with hash: {:?}", p.block.clone().unwrap().hash);

    let p_arc = Arc::new(p);

    // Make this block delivered
    cx.storage.add_delivered_block(block);

    on_finish_propose(p_arc, true, cx).await;
}