use types::{Block, Propose, ProtocolMsg, Replica, Transaction};
use super::{
    context::Context, 
    commit::on_finish_propose
};
use std::{
    sync::Arc,
    borrow::Borrow,
};

pub async fn on_receive_proposal(p: Arc<Propose>, cx: &mut Context) {
    let block = p.block.clone().unwrap();
    log::debug!(target:"consensus", "Handling proposal by [{}]: {}, {:?}", cx.myid, block.header.height, block.header);

    let proposal = p.borrow() as &Propose;
    // Check if the hash in the proposal is for the correct block
    if crypto::hash::ser_and_hash(block.borrow() as &Block) != proposal.block_hash {
        log::warn!(target:"consensus", 
            "Invalid hash in the proposal");
        return;
    }

    // 1) Is it correctly signed?
    let pk = cx.pub_key_map.get(&block.header.author).unwrap(); 
    let bytes = util::io::to_bytes(block.borrow() as &Block);
    if !pk.verify(&bytes, &proposal.proof) {
        println!("Block verification failed.");
        return;
    }

    // Do we already have this block?
    if cx.storage.is_delivered_by_hash(&block.hash) && !cx.waiting.contains(&block.hash)
    {
        log::debug!(target:"consensus", 
            "{} - Already have this block", cx.myid);
        return;
    }

    // Is this is an equivocation?
    //
    // We check for equivocation by checking that if we already have a block at
    // this height. If there is one, it must be an equivocation. Because, if
    // this was a repeated block, we must have returned from the previous check
    // for repeated blocks.
    if cx.storage.is_delivered_by_ht(block.header.height) {
        log::warn!(target:"consensus",
            "Equivocation detected: {:?}, {:?}", cx.storage.delivered_block_from_ht(block.header.height), block);
    }

    // Check validity
    cx.storage.add_delivered_block(block.clone());
    cx.prop_map.insert(block.hash, p.clone());
    
    // Are all the parents delivered?
    if let None = cx.storage.delivered_block_from_hash(&block.header.prev) {
        log::debug!(target:"consensus", 
            "Parent not found for the block: {:?}", block);
        // TODO request the block first, and then try again
        let dest = block.header.author;
        let msg = Arc::new(ProtocolMsg::Request(cx.req_ctr, block.header.prev));
        cx.waiting.insert(block.header.prev);
        cx.net_send.send((dest, msg)).unwrap();
        return;
    }

    // All parents are delivered, if we are waiting for this block's hash dont
    // wait anymore
    cx.waiting.remove(&block.hash);

    // Remove transactions from the pool
    cx.storage.clear(&block.body.tx_hashes);

    // Commit any possible blocks and process proposal
    //
    // Forward only the hash as we are not the proposer
    on_finish_propose(p, false, cx).await;
}

pub async fn do_propose(txs: Vec<Arc<Transaction>>, cx: &mut Context) {
    let mut new_block = Block::with_tx(txs);
    let parent = cx.last_seen_block.as_ref();
    new_block.header.prev = parent.hash;
    assert_eq!(new_block.header.prev, parent.hash, "Hash has moved");

    // new_block.header.extra = 
    new_block.header.author = cx.myid; 
    new_block.header.height =  parent.header.height+1;
    // new_block.header.blame_certificates = 

    // Finally, compute the hash
    let new_block_hash = new_block.compute_hash();
    
    let mut p = Propose::new(new_block_hash);
    p.block_hash = new_block_hash;
    new_block.hash = new_block_hash;

    let bytes = util::io::to_bytes(&new_block);
    p.proof = cx.my_secret_key.sign(&bytes)
        .expect("failed to sign the proposal");

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

    cx.prop_map.insert(block.hash, p_arc.clone());
    cx.storage.add_delivered_block(block);

    on_finish_propose(p_arc, true, cx).await;
}

pub async fn on_relay(sender: Replica, p: Propose, cx: &mut Context) {
    log::debug!(target:"consensus", "Got a relay message {:?}", p);
    let bhash = p.block_hash;

    // Do we have the block corresponding to the relay
    if cx.storage.is_delivered_by_hash(&bhash) {
        // We have the block, so we must have already handled the proposal
        return;
    } else {
        let dest = sender;
        let msg = ProtocolMsg::Request(cx.req_ctr, bhash);
        cx.waiting.insert(p.block_hash);
        cx.net_send.send((dest, Arc::new(msg))).unwrap();
        return;
    }
}