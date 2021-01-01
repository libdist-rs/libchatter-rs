use std::collections::{HashSet};

use super::context::Context;
use crypto::hash::EMPTY_HASH;
use types::{Block, Certificate, Transaction, VoteType, Vote, synchs::{Propose, ProtocolMsg}};
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
    if p.new_block.header.author != cx.leader_of_view() {
        println!("Got a proposal from an incorrect leader for the view");
        return false;
    }
    if p.new_block.header.height == 1 &&
        p.new_block.header.prev != EMPTY_HASH 
    {
        println!("First block does not extend the genesis block");
        return false;
    }
    if p.new_block.header.height > 1 && p.cert.votes.len() <= cx.num_faults {
        println!("Insufficient votes in the proposal");
        println!("Rejecting the proposal");
        return false;
    }

    let pk = match cx.pub_key_map.get(&p.new_block.header.author) {
        None => {unreachable!("Must have rejected before getting here");},
        Some(x) => x,
    };
    if !pk.verify(&p.new_block.hash, &p.proof) {
        println!("Got an incorrectly signed block");
        return false;
    }
    if cx.view != p.view {
        panic!("This view check should be unreachable");
    }

    // Check parent certificate
    if p.new_block.header.height == 1 {
        return true;
    }
    // Otherwise check if all the parent certificates are correctly signed
    let mut uniq_votes = HashSet::with_capacity(cx.num_faults+1);
    for v in &p.cert.votes {
        if let VoteType::Vote(data) = &v.msg {
            // Check if vote message is the same as that in the proposal
            if !check_hash_eq(data, &p.new_block.header.prev) {
                println!("The message of the vote is not the hash of the proposed block's prev");
                return false;
            }
            // check signature
            let pk = match cx.pub_key_map.get(&v.origin) {
                None => {
                    println!("Invalid vote origin");
                    return false;
                }
                Some(x) => x,
            };
            if !pk.verify(data, &v.auth) {
                println!("Invalid vote signature");
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
        println!("Parent undelivered");
        return false;
        // TODO add delivery
    }
    true
}

pub async fn on_receive_proposal(p: &Propose, cx: &mut Context) -> bool {
    let decision = false;
    // use decision to start commit timers in the reactor
    // println!("Received a proposal: {}", p.new_block.header.height);

    if cx.storage.all_delivered_blocks_by_hash.contains_key(&p.new_block.hash) {
        // println!("We have already processed this block last time");
        return decision;
    }
    
    // On receiving a proposal, check if it is in the same view
    // Check for the validity
    if !check_proposal(p, cx) {
        println!("Proposal checking failed");
        return decision;
    }
    return on_new_valid_proposal(p, cx).await;
}
    
pub async fn on_new_valid_proposal(p: &Propose, cx: &mut Context) -> bool {
    let decision = false;

    // Is the parent delivered?
    if !cx.storage.all_delivered_blocks_by_hash.contains_key(
        &p.new_block.header.prev) 
    {
        println!("We do not have the parent for this block");
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
    let decision = true;
    let ship = cx.net_send.clone();
    let ship_nodes = cx.num_nodes as u16;
    let ship_p = p.clone();
    let vote_ship = tokio::spawn(async move {
        if let Err(e) = ship.send(
            (ship_nodes, Arc::new(ProtocolMsg::VoteMsg(my_vote, ship_p))))
            .await 
        {
            println!("failed to send vote: {}", e);
        }
    });
    // Start 2\Delta timer (Moved to the reactor)

    let new_block_ref = Arc::new(p.new_block.clone());
    cx.storage.all_delivered_blocks_by_hash.insert(p.new_block.hash, 
        new_block_ref.clone());
    cx.storage.all_delivered_blocks_by_ht.insert(p.new_block.header.height, 
        new_block_ref.clone());
    for tx in &p.new_block.body.tx_hashes {
        cx.storage.pending_tx.remove(tx);
    }
    cx.height = p.new_block.header.height;
    cx.last_seen_block = new_block_ref.clone();
    cx.last_seen_cert = p.cert.clone();

    // wait for voting to finish?
    if let Err(e) = vote_ship.await {
        println!("Failed to send vote to the others:{}", e);
        return decision;
    }
    // println!("Sent a vote to all the nodes");
    decision
}

pub async fn do_propose(txs: Vec<Transaction>, cx: &mut Context) -> Propose {
    // Build the proposal
    let parent = &cx.last_seen_block;
    let mut new_block = Block::with_tx(txs);
    // update block contents here
    new_block.header.author = cx.myid;
    new_block.header.prev = parent.hash;
    // new_block.header.extra =
    new_block.header.height = parent.header.height+1;
    // Update the hash at the end
    new_block.update_hash();
    // Sign the block hash 
    let proof = match cx.my_secret_key.sign(&new_block.hash) {
        Err(e) => {
            panic!("Failed to sign the new proposal: {}", e);
            // return;
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
    // let new_block = &p.new_block;
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
        ).await {
            println!("Error broadcasting the block to all the nodes: {}", e);
        }
    });
    cx.storage.all_delivered_blocks_by_hash
        .insert(new_block_ref.hash, new_block_ref.clone());
    cx.storage.all_delivered_blocks_by_ht
        .insert(new_block_ref.header.height, new_block_ref.clone());
    // The leader can commit immediately? 
    // NOOOO! I learn it painfully! If the leader commits now, then it must also
    // acknowledge the client!
    // Commit normally, and tell the client after 2\Delta
    cx.vote_map.insert(new_block_ref.hash, new_block_cert);
    cx.height = new_block_ref.header.height;
    // the leader remains the same
    cx.last_seen_block = new_block_ref.clone();
    cx.last_committed_block_ht = cx.height;
    // the view remains the same
    broadcast.await.expect("failed to broadcast the proposal");
    return p;
}