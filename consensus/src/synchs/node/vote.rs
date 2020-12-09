use types::{Certificate, Vote, VoteType, synchs::Propose};
use context::Context;
use crypto::hash::Hash;

use super::{context, proposal::{check_hash_eq, on_receive_proposal}};

pub fn add_vote(v: &Vote, hash: Hash,cx: &mut Context) {
    if cx.cert_map.contains_key(&hash) {
        // println!("Extra vote received. discarding");
        return;
    }
    match cx.vote_map.remove(&hash) {
        None => {
            // First vote
            let mut cert = Certificate::empty_cert();
            cert.votes.push(v.clone());
            cx.vote_map.insert(hash, cert);
        },
        Some(mut c) => {
            // add the vote to the certificate
            c.votes.push(v.clone());
            // promote it to a full certificate if it has f+1 signatures
            if c.votes.len() > cx.num_faults {
                cx.cert_map.insert(hash, c.clone());
            } else {
                cx.vote_map.insert(hash, c.clone());
            }
        },
    }
}

pub async fn on_vote(v: &Vote, p:Propose, cx: &mut Context) -> bool {
    let decision = false;
    // print!("Received a vote message: {:?}", v);

    // Check if we have already processed the block for which we have the vote
    // and if not check if it is valid
    let pk = match cx.pub_key_map.get(&v.origin) {
        None => {
            println!("vote from an unknown origin");
            return decision;
        },
        Some(x) => x,
    };
    let data = match &v.msg {
        VoteType::Vote(d) => d,
        _ => unreachable!("other vote types cant be here"),
    };
    if !pk.verify(&data, &v.auth) {
        println!("vote not correctly signed");
        return decision;
    }

    if !check_hash_eq(data, &p.new_block.hash) {
        println!("vote not for the corresponding hash");
        return decision;
    }
    let hash = p.new_block.hash;

    // Is this an equivocation?
    if let Some(x) = cx.storage.all_delivered_blocks_by_ht
        .get(&p.new_block.header.height) 
    {
        // We already have a block at this height
        // Check if this is an equivocation
        if x.hash != hash {
            println!("Got an equivocation");
            return decision;
        } else {
            // println!("We have seen this block before, and have already voted for it");
            // add vote for this
            add_vote(v, p.new_block.hash, cx);
            return decision;
        }
    }

    // This is a vote for a new block
    // add the vote for the block anyways since it may not be currently valid,
    // but may become valid after we get the missing blocks
    add_vote(v, hash, cx);


    // Let the reactor know that we have to start the commit timers for this
    // block, if this is a new proposal
    return on_receive_proposal(&p, cx).await;
}