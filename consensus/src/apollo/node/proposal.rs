use types::{Block, Propose, ProtocolMsg, Replica, Transaction, WireReady};
use super::{
    context::Context, 
    commit::on_finish_propose
};
use std::sync::Arc;

/// Called when a proposal is received due to:
/// - A block getting delivered
/// - Receiving a proposal directly
///
/// Note: This will only execute once all the block for of the proposal is delivered
pub async fn on_receive_proposal(sender: Replica, p: Propose, cx: &mut Context) {
    // Triggered if we have already called on_finish_proposal
    if cx.prop_chain.contains_key(&p.block_hash) {
        log::debug!(target:"consensus", "Already dealt with proposal {:?}", p);
        return;
    }

    log::debug!(target:"consensus", 
        "Handling proposal: {:?}", p);

    let block = p.block.clone().unwrap();

    // Check if the parent is delivered
    if !cx.storage.is_delivered_by_hash(&block.header.prev) {
        log::debug!(target:"consensus", 
            "Parent not found for the block: {:?}", block);
        let dest = sender;
        let msg = Arc::new(ProtocolMsg::Request(cx.req_ctr, block.header.prev));
        cx.prop_waiting_parent.insert(block.header.prev, p);
        cx.net_send.send((dest, msg)).unwrap();
        return;
    }

    // Check if the hash in the proposal is for the correct block
    if crypto::hash::ser_and_hash(block.as_ref()) != p.block_hash {
        log::warn!(target:"consensus", 
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
            log::warn!(target:"consensus",
                "Equivocation detected: {:?}, {:?}", cx.storage.delivered_block_from_ht(block.header.height), block);
            return;
        }
    }

    on_new_valid_proposal(Arc::new(p), cx).await;
    let mut bhash = block.hash;
    while let Some(p_new) = cx.prop_waiting_parent.remove(&bhash) {
        bhash = p_new.block_hash;
        on_new_valid_proposal(Arc::new(p_new), cx).await;
    }
}

/// Called once for every proposal of a height
///
/// Before calling, ensure that the block for this proposal and its parents are
/// delivered
pub async fn on_new_valid_proposal(p: Arc<Propose>, cx:&mut Context) {
    let block = p.block.clone().unwrap();

    // Check validity
    cx.storage.add_delivered_block(block.clone());
    // If we were waiting for this proposal, do not wait anymore
    cx.prop_waiting.remove(&block.hash);
    cx.prop_waiting_parent.remove(&block.hash);

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
    ) {
        log::warn!(target:"consensus",
            "Server channel closed with error: {}", e);
    };

    log::debug!(target:"consensus", 
        "Proposing block with hash: {:?}", p.block.clone().unwrap().hash);

    let p_arc = Arc::new(p);

    // Make this block delivered
    cx.storage.add_delivered_block(block);

    on_finish_propose(p_arc, true, cx).await;
}

/// Check if 
pub async fn on_relay(sender: Replica, p: Propose, cx: &mut Context) {
    log::debug!(target:"consensus", "Got a relay message {:?}", p);
    let bhash = p.block_hash;

    // Do we have the block corresponding to the relay and have we handled it
    // before?
    if cx.prop_chain.contains_key(&bhash) {
        // We have already processed this proposal before
        return;
    }
    if cx.prop_waiting.contains_key(&bhash) {
        // We are already waiting for this proposal, do not request again
        // TODO: Request Again?
        return;
    }

    log::debug!(target:"consensus",
        "Got a relay {:?}, but we dont have the block yet.", p);

    let dest = sender;
    let msg = ProtocolMsg::Request(cx.req_ctr, bhash);
    cx.prop_waiting.insert(p.block_hash,p);
    cx.net_send.send((dest, Arc::new(msg))).unwrap();
    return;
}