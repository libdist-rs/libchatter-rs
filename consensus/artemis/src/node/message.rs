use types::artemis::{Block, ProtocolMsg, Replica};
use types::BlockTrait;
use super::*;

/// Buffer and re-order messages by queueing messages. This function adds the message to the correct queues. So that when dequeueing we dequeue them correctly.
pub fn buffer_message(sender: Replica, message: ProtocolMsg, cx: &mut Context) {
    match message {
        ProtocolMsg::Invalid | ProtocolMsg::RawNewBlock(..) | ProtocolMsg::RawResponse(..) | ProtocolMsg::RawUCRVote(..) => 
        (),
        ProtocolMsg::NewBlock(b) => 
            cx.block_processing_waiting.push_back(b),
        ProtocolMsg::Response(_, blk) => 
            cx.response_waiting.push_back((sender,blk)),
        x => 
            cx.other_buf.push_back((sender, x)),
    }
}

/// Process message dequeues buffered messages and tries reacting to them.
/// We handle messaeges in the following order:
/// - New blocks (`block_processing_waiting`)
/// - Responses (`response_waiting`)
/// - Other messages (`other_buf`)
pub async fn process_message(cx:&mut Context) 
{
    // Process view leader's blocks
    while let Some(b) = cx.block_processing_waiting.pop_front() {
        on_receive_new_block_direct(cx, b).await;
    }
    // Try resolving some responses, we got a block as a response from someone
    while let Some((sender,  block)) = cx.response_waiting.pop_front() {
        update_delivery(cx, block, sender).await;
    }
    // Try dealing with any votes that got ready or are from the future
    while let Some(v) = cx.vote_ready.remove(&cx.round()) {
        on_receive_round_vote(cx, v).await;
    }
    // Try dealing with protocol messages now
    while let Some((sender, msg)) = cx.other_buf.pop_front() {
        match msg {
            ProtocolMsg::UCRVote(v) => 
                try_receive_round_vote(cx, sender, v).await,
            ProtocolMsg::Relay(v) => 
                try_receive_round_vote(cx, sender, v).await,
            ProtocolMsg::Request(req_id,h) => 
                handle_request(sender, req_id, h, cx).await,
            ProtocolMsg::Blame(v) => 
                on_receive_blame(v, cx).await,
            _ => panic!("unreachable"),
        }
    }
}

/// Take a block and check if this block is delivered
/// If the block is delivered, we will trigger the next steps
/// Otherwise, we request the parents
pub async fn update_delivery(cx:&mut Context, b: Block, sender: Replica) {
    // We already got the block from somewhere
    if cx.storage.is_delivered_by_hash(&b.get_hash()) {
        return;
    }
    let p_hash = b.blk.header.prev;
    let is_parent_delivered = cx.storage.is_delivered_by_hash(
        &p_hash);
    if cx.block_parent_waiting.contains_key(&p_hash) {
        // We are already waiting for this block
        log::debug!("Already waiting for this block");
        return;
    }
    let b_hash = b.get_hash();
    if !is_parent_delivered {
        cx.block_parent_waiting.insert(p_hash, b_hash);
        cx.undelivered_blocks.insert(b_hash, b);
        do_request(cx, sender, b_hash).await;
        return;
    }
    // We have a new delivered block
    do_delivery(b, cx);
}