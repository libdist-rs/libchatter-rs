use std::collections::HashSet;
use super::context::Context;
use crypto::hash::EMPTY_HASH;
use types::{
    Block, 
    Certificate, 
    Transaction, 
    VoteType, 
    Vote, 
    synchs::{
        Propose, 
        ProtocolMsg
    }
};
use std::sync::Arc;

pub fn check_hash_eq(left:&[u8], right:&[u8]) -> bool {
    // return left == right;
    if left.len() != right.len() {
        println!("The two hashes have unequal lengths");
        return false;
    }
    // Obviously this not a hash
    if left.len() != 32 {
        println!("The length of the hash is not 32");
        return false;
    }
    for i in 0..32 {
        if left[i] != right[i] {
            println!("One of the element is incorrect");
            return false
        }
    }
    true
}

pub fn check_proposal(p: &Propose, cx:&Context) -> bool {
    // Check if the author is correct
    if p.new_block.header.author != cx.leader_of_view() {
        log::warn!(target:"consensus", 
            "Got a proposal from an incorrect leader for the view");
        return false;
    }

    // Check if the first block extends the genesis block
    if p.new_block.header.height == 1 &&
        p.new_block.header.prev != EMPTY_HASH 
    {
        log::warn!(target:"consensus", 
            "First block does not extend the genesis block");
        return false;
    }

    // Check if the block has sufficient votes
    if p.new_block.header.height > 1 && p.cert.votes.len() <= cx.num_faults {
        log::warn!(target:"consensus", 
            "Insufficient votes in the proposal, rejecting the proposal");
        return false;
    }

    // Check signature for the proposal
    let pk = cx.pub_key_map.get(&p.new_block.header.author).unwrap();
    if !pk.verify(&p.new_block.hash, &p.proof) {
        log::warn!(target:"consensus", 
            "Got an incorrectly signed block");
        return false;
    }

    // Check if the view is correct
    if cx.view != p.view {
        panic!("This view check should be unreachable");
    }

    // If the parent is genesis block, then the parent is correctly certified
    if p.new_block.header.height == 1 {
        return true;
    }
    // Otherwise check if all the parent certificates are correctly signed
    let mut uniq_votes = HashSet::with_capacity(cx.num_faults+1);
    for v in &p.cert.votes {
        if let VoteType::Vote(data) = &v.msg {
            // Check if vote message is the same as that in the proposal
            if !check_hash_eq(data, &p.new_block.header.prev) {
                log::warn!(target:"consensus", 
                    "The message of the vote is not the hash of the proposed block's prev");
                return false;
            }
            // check signature
            let pk = match cx.pub_key_map.get(&v.origin) {
                None => {
                    log::warn!(target:"consensus", "Invalid vote origin");
                    return false;
                }
                Some(x) => x,
            };
            if !pk.verify(data, &v.auth) {
                log::warn!(target:"consensus", "Invalid vote signature");
                return false;
            }
            // Add this unique vote
            uniq_votes.insert(v.origin);
        } else {
            return false
        }
    }
    if uniq_votes.len() < cx.num_faults {
        return false;
    }
    // Is it extending the last known parent?
    if p.new_block.header.prev != cx.last_seen_block.hash {
        log::warn!(target:"consensus", "Parent undelivered");
        return false;
        // TODO add delivery
    }
    true
}

pub async fn on_receive_proposal(p: &Propose, cx: &mut Context) -> bool {
    // Use decision to start commit timers in the reactor
    let decision = false;
    
    log::debug!(target: "consensus", 
        "Received a proposal: {}", p.new_block.header.height);

    if cx.storage.all_delivered_blocks_by_hash.contains_key(&p.new_block.hash) {
        log::debug!("We have already processed this block last time");
        return decision;
    }
    
    // On receiving a proposal, check if it is in the same view
    // Check for the validity
    if !check_proposal(p, cx) {
        log::warn!(target:"consensus", "Proposal checking failed");
        return decision;
    }
    return on_new_valid_proposal(p, cx).await;
}
    
pub async fn on_new_valid_proposal(p: &Propose, cx: &mut Context) -> bool {
    let mut decision = false;

    // Is the parent delivered?
    if !cx.storage.all_delivered_blocks_by_hash.contains_key(
        &p.new_block.header.prev) 
    {
        log::warn!(target:"consensus", 
            "We do not have the parent for this block");
        // TODO: Request and Deliver blocks
        return decision;
    }
    // Everything looks fine, initiate voting and continue to process this
    // proposal
    let my_vote = match cx.my_secret_key.sign(&p.new_block.hash) {
        Err(e) => {
            panic!("Failed to sign a vote: {}", e);
        },
        Ok(vo) => {
            Vote{
                msg: VoteType::Vote(p.new_block.hash.clone().to_vec()),
                origin: cx.myid,
                auth: vo,
            }
        },
    };

    decision = true;

    // Create a scope and send the proposal to other nodes
    let ship = cx.net_send.clone();
    let ship_nodes = cx.num_nodes as u16;
    let ship_p = p.clone();
    let vote_ship = tokio::spawn(async move {
        if let Err(e) = ship.send(
            (ship_nodes, Arc::new(ProtocolMsg::VoteMsg(my_vote, ship_p)))) 
        {
            log::warn!(target:"consensus", 
                "failed to send vote: {}", e);
        }
    });

    let new_block_ref = Arc::new(p.new_block.clone());

    // Update the consensus context
    cx.storage.all_delivered_blocks_by_hash.insert(
        p.new_block.hash, 
        new_block_ref.clone()
    );
    cx.storage.all_delivered_blocks_by_ht.insert(
        p.new_block.header.height, 
        new_block_ref.clone()
    );
    for tx in &p.new_block.body.tx_hashes {
        cx.storage.pending_tx.remove(tx);
    }
    cx.height = p.new_block.header.height;
    cx.last_seen_block = new_block_ref.clone();
    cx.last_seen_cert = p.cert.clone();

    // wait for voting to finish?
    if let Err(e) = vote_ship.await {
        log::warn!(target:"consensus", 
            "Failed to send vote to the others:{}", e);
        return decision;
    }

    log::debug!(target:"consensus", "Sent a vote to all the nodes");
    decision
}

pub async fn do_propose(txs: Vec<Transaction>, cx: &mut Context) -> Propose {
    // Build the proposal
    let parent = &cx.last_seen_block;
    let mut new_block = Block::with_tx(txs);

    // Update block contents here
    new_block.header.author = cx.myid;
    new_block.header.prev = parent.hash;
    new_block.header.height = parent.header.height+1;
    
    // Update the hash at the end
    new_block.update_hash();
    
    // Sign the block hash 
    let proof = match cx.my_secret_key.sign(&new_block.hash) {
        Err(e) => {
            panic!("Failed to sign the new proposal: {}", e);
        },
        Ok(sig) => sig,
    };

    // Add self vote to the certificate map
    let self_vote = Vote{
        msg: VoteType::Vote(new_block.hash.to_vec()),
        origin: cx.myid,
        auth: proof.clone(),
    };

    let mut new_block_cert = Certificate::empty_cert();
    new_block_cert.votes.push(self_vote);
    let new_block_cert = new_block_cert;

    // The block is ready, build proposal
    let new_block_ref = Arc::new(new_block.clone());
    let mut p = Propose::new(new_block);
    p.proof = proof;
    p.cert = match cx.cert_map.get(&parent.hash) {
        None => {
            panic!("Must call propose only if the parent is certified");
        },
        Some(x) => x.clone(),
    };
    p.view = cx.view;

    // Ship the proposal
    let ship = cx.net_send.clone();
    let ship_num = cx.num_nodes as u16;
    let ship_p = p.clone();
    let broadcast = tokio::spawn(async move {
        if let Err(e) = ship.send(
            (ship_num, Arc::new(ProtocolMsg::NewProposal(ship_p)))
        ) {
            println!("Error broadcasting the block to all the nodes: {}", e);
        }
    });

    // Update consensus context
    cx.storage.all_delivered_blocks_by_hash
        .insert(new_block_ref.hash, new_block_ref.clone());
    cx.storage.all_delivered_blocks_by_ht
        .insert(new_block_ref.header.height, new_block_ref.clone());

    // Q) Can the leader commit immediately? 
    // A) NOOOO! I learnt it painfully! If the leader commits now, then it must
    // also acknowledge the client now, which becomes a problem!
    // Commit normally, and tell the client after 2\Delta
    cx.vote_map.insert(new_block_ref.hash, new_block_cert);
    cx.height = new_block_ref.header.height;
    // The leader remains the same
    cx.last_seen_block = new_block_ref.clone();
    cx.last_committed_block_ht = cx.height;
    // The view remains the same
    broadcast.await.expect("failed to broadcast the proposal");

    p
}