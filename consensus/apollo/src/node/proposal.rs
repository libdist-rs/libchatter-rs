use types::apollo::{Block, Propose, ProtocolMsg, Transaction, Replica};
use types::BlockTrait;
use types::WireReady;
use super::*;
use std::sync::Arc;

/// Creates a block using the last seen block as the parent
/// Then adds the block as delivered
/// do_propose is called after ensuring that we have sufficient transactions and that we are the leader for this round
pub async fn do_propose(txs: Vec<Arc<Transaction>>, cx: &mut Context) {
    // Get the parent
    let parent = cx.last_seen_block.as_ref();

    // Create a block
    let mut new_block = Block::with_tx(txs);
    new_block.header.prev = parent.hash;
    new_block.header.author = cx.myid(); 
    new_block.header.height =  parent.header.height+1;
    // Finally, compute the hash
    let new_block = new_block.init();

    // Create a proposal
    let mut p = Propose::new(new_block.hash);
    p.round = cx.round();
    p.sig.origin = cx.myid();
    p.sign_block(&new_block, cx.my_secret_key.as_ref());
    p.block = Some(Arc::new(new_block.clone()));

    let msg = Arc::new(ProtocolMsg::RawNewProposal(p.clone(), new_block.clone()));
    let p_arc = Arc::new(p);
    cx.multicast(msg).await;
    if cx.is_client_apollo_enabled() {
        cx.multicast_client(p_arc.clone()).await;
    }

    // Make this block delivered
    cx.storage.add_delivered_block(Arc::new(new_block));

    // Self handle new propose
    on_receive_proposal(p_arc, cx).await;
}

/// Try to receive a new propose message
pub async fn try_receive_proposal(p: Propose, from:Replica, cx:&mut Context) {
    if cx.round() > p.round {
        log::debug!("Got a proposal from the past");
        return;
    }
    if cx.round() < p.round {
        log::debug!("Got a proposal from the future");
        cx.future_msgs.insert(p.round, p);
        return;
    }
    let b_hash = p.block.as_ref().map(|b| b.hash).unwrap();
    if !cx.storage.is_delivered_by_hash(&b_hash) {
        // I don't have the chain for this. Ask chain from the sender
        let msg = Arc::new(ProtocolMsg::Request(cx.req_ctr, b_hash));
        let job = cx.c_send(from, msg).await;
        cx.prop_waiting.insert(b_hash, p);
        job.await.unwrap();
        return;
    }

    // I have the chain

    let block = p.block.clone().unwrap();
    // 1) Is it correctly signed?
    if cx.round_leader() != p.sig.origin {
        return;
    }
    if cx.round_leader() != block.get_author() {
        return;
    }
    if p.sig.origin != cx.myid() && !p.check_sig(block.as_ref(), &cx.pub_key_map[&p.sig.origin]) {
        return;
    }

    on_receive_proposal(Arc::new(p), cx).await;
}

/// Called when a proposal is received due to:
/// - A block getting delivered
/// - Receiving a proposal directly
///
/// Note: This will only execute once all the block for of the proposal is delivered
pub async fn on_receive_proposal(p: Arc<Propose>, cx: &mut Context) {
    log::debug!("Handling valid proposal: {:?}", p);
    let block = p.block.clone().unwrap();
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

    // Remove transactions from the pool
    cx.storage.clear(&block.body.tx_hashes);
    cx.prop_chain_by_hash.insert(p.block_hash, p.clone());
    cx.prop_chain_by_round.insert(p.round, p.clone());

    // Trigger commit rule
    if cx.round() > cx.num_faults() {
        do_commit(cx).await;
    }

    // Should we send this to the client
    if cx.is_client_apollo_enabled() {
        cx.multicast_client(p).await;
    }

    cx.last_seen_block = block.clone();
    cx.update_round();
}