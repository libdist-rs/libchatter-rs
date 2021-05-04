use types::{BlockTrait, artemis::{ClientMsg, Payload, ProtocolMsg, Replica, UCRVote}};
use super::*;
use std::{collections::VecDeque, sync::Arc};

/// Called to check if we are ready to do UCR voting
/// Check if
/// 1. Am I the next round leader?
/// 2. Do I have new blocks?
pub async fn try_round_vote(cx: &mut Context) {
    // I am not the next round leader, return
    if cx.myid() != cx.round_leader {
        log::debug!("I {} am not the leader for {}", cx.myid(), cx.round_leader);
        return;
    }
    // Do I have any new blocks that I can vote for?
    if cx.last_seen_block.get_height() <= cx.last_voted_block.get_height() {
        log::debug!("I {} do not have any new blocks", cx.myid());
        return;
    }
    log::debug!("I am the round leader and I have new blocks to vote for");
    do_round_vote(cx).await;
}

/// Triggered when it is this node's turn to UCR vote
pub async fn do_round_vote(cx: &mut Context) {
    let mut v = UCRVote::new();
    v.hash = cx.last_seen_block.get_hash();
    v.round = cx.round();
    v.view = cx.view;
    v.compute_sig(&cx.my_secret_key());
    // Multicast the vote
    let msg = Arc::new(ProtocolMsg::RawUCRVote(v.clone()));
    cx.multicast(msg).await;
    let mut block_vec = VecDeque::new();
    let mut tail = v.hash;
    while cx.last_voted_block.get_hash() != tail {
        let b = cx.storage.delivered_block_from_hash(&tail).expect("Failed to get block");
        block_vec.push_front(b.as_ref().clone());
        tail = b.blk.header.prev;
    }
    // panic!("Unimplemented block vector creation");
    let block_vec = block_vec.into_iter().map(|b|{
        (b,Payload::empty())
    }).collect();
    let msg = Arc::new(ClientMsg::RawNewBlock(v.clone(), block_vec));
    cx.multicast_client(msg).await;
    // Process self vote
    on_receive_round_vote(cx, v).await;
}

/// `try_receive_round_vote` is called to check if all the chain is delivered.
/// If it is, then we call `on_receive_round_vote`, otherwise we request it from the sender
/// Also checks if we got votes from the future/past
pub async fn try_receive_round_vote(cx:&mut Context, from: Replica, ucr_vote: UCRVote) {
    // We may get multiple votes from relay and do_round_vote
    if cx.round() > ucr_vote.round {
        log::debug!("Discarding duplicate votes for round {}, already in round {}", ucr_vote.round, cx.round());
        return;
    }
    // Is this the correct round?
    if cx.round() < ucr_vote.round {
        // We got a ucr_vote from the future
        log::debug!("Got a vote from the future");
        // What TODO? Keep it ready until we move to this round
        cx.vote_ready.insert(ucr_vote.round, ucr_vote);
        return;
    }
    // Do I have the chain?
    if !cx.storage.is_delivered_by_hash(&ucr_vote.hash) {
        // I don't have the chain for this. Ask chain from the sender
        let msg = Arc::new(ProtocolMsg::Request(cx.req_ctr, ucr_vote.hash));
        let job = cx.c_send(from, msg).await;
        cx.vote_waiting.insert(ucr_vote.hash, ucr_vote);
        job.await.unwrap();
        return;
    }
    // I have the chain
    on_receive_round_vote(cx, ucr_vote).await;
}

/// `on_receive_vote` is called after ensuring that we have the chain, and we are ready to process this message for this round
pub async fn on_receive_round_vote(cx:&mut Context, ucr_vote: UCRVote) {
    // Check signature
    if cx.view != ucr_vote.view {
        log::warn!("Invalid view in UCR vote message");
        return;
    }
    // The view is correct by now
    if cx.myid() != cx.round_leader {
        if !ucr_vote.check_sig(&cx.pub_key_map[&cx.round_leader]) {
            log::warn!("Invalid signature on the UCR Vote");
            return;
        }
    }
    // The signature is correct by now

    // Add this to our vote chain
    cx.vote_chain.insert(ucr_vote.round, Arc::new(ucr_vote.clone()));

    // Trigger commit rule
    if cx.round() > cx.num_faults() {
        do_commit(cx);
    }

    // Update the last voted block
    let last_voted_block = cx.storage.delivered_block_from_hash(&ucr_vote.hash)
        .expect("Obtained a vote for an unknown hash");
    cx.last_voted_block = last_voted_block.clone();
    
    // Relay the vote
    let msg = Arc::new(ProtocolMsg::Relay(ucr_vote));
    let job = cx.c_send(cx.next_round_leader(), msg).await;
    
    // Update leaders, and round
    cx.update_round();
    log::debug!("Going to the next round  {}", cx.round());
    job.await.unwrap();
}