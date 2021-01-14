use types::{CertType, Certificate, synchs::Propose};
use crypto::hash::Hash;
use super::{
    context::Context, 
    proposal::{
        on_receive_proposal
    }
};
use std::sync::Arc;

pub fn add_vote(mut c: Certificate, hash: Hash, cx: &mut Context) {
    if cx.cert_map.contains_key(&hash) {
        log::debug!(target:"consensus","Extra vote received. discarding");
        return;
    }
    match cx.vote_map.remove(&hash) {
        None => {
            // First vote
            cx.vote_map.insert(hash, c);
        },
        Some(mut cert) => {
            // Add the vote to the certificate
            cert.votes.push(c.votes.pop().unwrap());
            // Promote it to a full certificate if it has f+1 signatures
            if cert.votes.len() > cx.num_faults {
                cx.cert_map.insert(hash, cert);
            } else {
                cx.vote_map.insert(hash, cert);
            }
        },
    }
}

pub async fn on_vote(c: Certificate, mut p: Propose, cx: &mut Context) -> bool {
    let decision = false;
    log::debug!(target:"consensus", 
        "Received a vote message: {:?}", c);

    if c.votes.len() != 1 {
        log::warn!(target:"consensus", 
            "Invalid number of votes in vote message");
        return false;
    }
    // Check if we have already processed the block for which we have the vote
    // and if not check if it is valid
    let vote = &c.votes[0];
    let pk = match cx.pub_key_map.get(&vote.origin) {
        None => {
            log::warn!(target:"consensus", "vote from an unknown origin");
            return decision;
        },
        Some(x) => x,
    };
    let (sign_data, blk_hash) = match &c.msg {
        CertType::Vote(d) => (util::io::to_bytes(&c.msg), *d),
        _ => unreachable!("other vote types cant be here"),
    };

    if blk_hash != p.block_hash {
        log::warn!(target:"consensus","Invalid vote message received");
        return decision;
    }

    if !pk.verify(&sign_data, &vote.auth) {
        log::warn!(target:"consensus", "vote not correctly signed");
        return decision;
    }

    if !cx.storage.is_delivered_by_hash(&blk_hash) {
        log::debug!(target:"consensus", 
            "Received vote for an undelivered block");
        return decision;
    }

    let new_block = cx.storage.delivered_block_from_hash(&blk_hash).unwrap();
    p.block = Some(new_block);
    let new_block = p.block.as_ref().unwrap().as_ref();

    // Is this an equivocation?
    if let Some(x) = cx.storage.delivered_block_from_ht(new_block.header.height) 
    {
        // We already have a block at this height
        // Check if this is an equivocation
        if x.hash != blk_hash {
            log::warn!(target:"consensus", "Got an equivocation: {:?}, {:?}", 
                x.header, new_block.header);
            return decision;
        } 
        // else {
        //     log::debug!("We have seen this block before, and have already voted for it");
        //     // add vote for this
        //     return decision;
        // }
    }

    // This is a vote for a new block
    // add the vote for the block anyways since it may not be currently valid,
    // but may become valid after we get the missing blocks
    add_vote(c, blk_hash, cx);


    // Let the reactor know that we have to start the commit timers for this
    // block, if this is a new proposal
    return on_receive_proposal(Arc::new(p), cx).await;
}