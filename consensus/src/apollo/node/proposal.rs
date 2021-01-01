use types::{Block, GENESIS_BLOCK, Propose, ProtocolMsg, Transaction};

use super::{context::Context, commit::on_finish_propose};
use std::sync::Arc;

pub async fn on_receive_proposal(p: &Propose, cx: &mut Context) {
    // println!("Handling proposal by [{}]: {}, {:?}", cx.myid, p.new_block.header.height, p.new_block.header);

    // let mut p = p.clone();
    // p.new_block = block;
    // let p = p;
    let block = &p.new_block; // Make the block immutable again, so we dont accidently move something
    // 1) Is it correctly signed?
    if let Some(pk) = cx.pub_key_map.get(&block.header.author) {
        let bytes = util::io::to_bytes(&block);
        if !pk.verify(&bytes, &p.proof) {
            println!("Block verification failed.");
            return;
        }
    }
    // Do we already have this block?
    if let Some(x) = cx.storage.all_delivered_blocks_by_hash.get(
        &block.hash) 
    {
        if x.hash != block.hash {
            println!("Equivocation detected");
            return;
        } else {
            // println!("{} - Already have this block", cx.myid);
            return;
        }
    }
    // Check validity
    
    // Are all the parents delivered?
    if !cx.storage.all_delivered_blocks_by_hash.contains_key(&block.header.prev) && block.header.height != GENESIS_BLOCK.header.height + 1 {
        // println!("Parent not found for the block: {:?}", block);
        return;
        // TODO request the block first, and then try again
    }

    // Remove transactions from the pool
    for h in &block.body.tx_hashes {
        cx.storage.pending_tx.remove(h);
    }
    // Commit any possible blocks and process proposal
    on_finish_propose(p, cx).await;
}

pub async fn do_propose(txs: Vec<Transaction>, cx: &mut Context) {
    let mut new_block = Block::with_tx(txs);
    let parent = &cx.last_seen_block;
    new_block.header.prev = parent.hash;
    assert_eq!(new_block.header.prev, parent.hash, "Hash has moved");
    // new_block.header.extra = 
    new_block.header.author = cx.myid; 
    new_block.header.height =  parent.header.height+1;
    // new_block.header.blame_certificates = 

    new_block.update_hash();

    let mut p = Propose::new(new_block);
    let bytes = util::io::to_bytes(&p.new_block);
    p.proof = cx.my_secret_key.sign(&bytes)
        .expect("failed to sign the proposal");
    // The leader broadcasts the transaction
    if let Err(e) = cx.net_send.send(
        (cx.num_nodes, Arc::new(ProtocolMsg::NewProposal(p.clone())))
    ).await {
        println!("Server channel closed with error: {}", e);
    };
    // println!("Proposing block with hash: {:?}", p.new_block.header);
    on_finish_propose(&p, cx).await;
}