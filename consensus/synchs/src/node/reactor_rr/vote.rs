use types::{CertType, Certificate, View, synchs::{Propose, ProtocolMsg}};
use crypto::hash::Hash;
use super::{context::Context, phase::Phase, proposal::{
        on_receive_proposal
    }};
use std::sync::Arc;
use fnv::{FnvHashMap as HashMap, FnvHashSet as HashSet};

pub async fn add_vote(c: Certificate, hash: Hash, cx: &mut Context) {
    debug_assert!(c.votes.len() == 1);
    log::debug!("Adding vote {:?}", c);

    if cx.cert_map.contains_key(&hash) {
        log::debug!("Extra vote received. discarding");
        return;
    }
    let mut cert_map = match cx.vote_map.remove(&hash) {
        None => {
            log::debug!("First vote");
            let mut c_map = HashMap::default();
            c_map.insert(c.votes[0].origin, c);
            // First vote
            cx.vote_map.insert(hash, c_map);
            return;
        },
        Some(cert_map) => cert_map,
    };
    // Add the vote to the certificate
    log::debug!("Not first vote: {:?}", c);
    cert_map.insert(c.votes[0].origin, c);
    // Promote it to a full certificate if it has f+1 signatures
    if cert_map.len() > cx.num_faults {
        let mut new_cert = Certificate::empty_cert();
        for (_o,mut v) in cert_map {
            new_cert.votes.push(v.votes.pop().unwrap());
            new_cert.msg = v.msg;
        }
        let cert_arc = Arc::new(new_cert);
        log::debug!("Promoting and updating last seen certificate {:?}", cert_arc);

        cx.cert_map.insert(hash, cert_arc.clone());
        cx.last_seen_cert = cert_arc.clone();
        on_quit_view(cert_arc, cx).await;
    } else {
        cx.vote_map.insert(hash, cert_map);
    }
}

pub async fn on_recv_quit_view(v: View, cert: Certificate, cx: &mut Context) {
    log::debug!("Quitting view {:?}", cert);

    if cx.view > v {
        log::debug!("Already quit the view");
        return;
    }

    // We may seen a few votes from different nodes, but we are not sure if all
    // of the votes in this are valid
    let (sign_data, block_hash) = match &cert.msg {
        CertType::Vote(ref _v, ref h) => (util::io::to_bytes(&cert.msg),*h),
        _ => panic!("Quit view code unreachable"),
    };

    if cert.votes.len() <= cx.num_faults {
        log::debug!("Insufficient votes in a quit view certificate");
        return;
    }

    let mut unique_votes = HashSet::default();
    for vote in &cert.votes {
        let pk = match cx.pub_key_map.get(&vote.origin) {
            None => {
                log::warn!("vote from an unknown origin");
                return;
            },
            Some(x) => x,
        };
        if !pk.verify(&sign_data, &vote.auth) {
            log::warn!("quit view vote not correctly signed");
            return;
        }
        if unique_votes.contains(&vote.origin) {
            log::debug!("Duplicate vote in a quit view certificate");
            return;
        } else {
            unique_votes.insert(vote.origin);
        }
    }
    // The certificate:
    // 1) Contain f+1 signatures
    // 2) Contain f+1 valid votes
    // 3) Contain f+1 unique votes

    // Update the last seen cert, if this is the first time we are observing this certificate
    let cert_arc = Arc::new(cert);
    if !cx.cert_map.contains_key(&block_hash) {
        cx.cert_map.insert(block_hash, cert_arc.clone());
        cx.vote_map.remove(&block_hash);
    }

    // Everything went well, I too will quit the view
    on_quit_view(cert_arc, cx).await;
}

pub async fn on_quit_view(cert: Arc<Certificate>, cx: &mut Context) {
    log::debug!("Quitting view on observing {:?}", cert);
    cx.last_seen_cert = cert.clone();

    // Broadcast the certificate to indicate a quit view, if the others have not already quit the view
    cx.net_send.send((cx.num_nodes, Arc::new(ProtocolMsg::ChangeView(cx.view,cert.as_ref().clone())))).unwrap();
    // Go to the next view
    cx.view += 1;
    cx.phase = Phase::NextView(cx.view);

    // Wait Delta
    cx.event_queue.insert(Phase::Status, 
        std::time::Duration::from_millis(cx.delay));
}

pub async fn on_vote(c: Certificate, mut p: Propose, cx: &mut Context) -> bool {
    let decision = false;
    log::debug!("Received a vote message: {:?}", c);

    if c.votes.len() != 1 {
        log::warn!(
            "Invalid number of votes in vote message");
        return false;
    }
    // Check if we have already processed the block for which we have the vote
    // and if not check if it is valid
    let vote = &c.votes[0];
    let pk = match cx.pub_key_map.get(&vote.origin) {
        None => {
            log::warn!("vote from an unknown origin");
            return decision;
        },
        Some(x) => x,
    };
    let (sign_data, blk_hash) = match &c.msg {
        CertType::Vote(_v, d) => (util::io::to_bytes(&c.msg), *d),
        _ => unreachable!("other vote types cant be here"),
    };

    if blk_hash != p.block_hash {
        log::warn!("Invalid vote message received");
        return decision;
    }

    if !pk.verify(&sign_data, &vote.auth) {
        log::warn!("vote not correctly signed");
        return decision;
    }

    if !cx.storage.is_delivered_by_hash(&blk_hash) {
        log::debug!(
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
            log::warn!("Got an equivocation: {:?}, {:?}", 
                x.header, new_block.header);
            return decision;
        } 
    }

    // This is a vote for a new block
    // add the vote for the block anyways since it may not be currently valid,
    // but may become valid after we get the missing blocks
    add_vote(c, blk_hash, cx).await;

    // Let the reactor know that we have to start the commit timers for this
    // block, if this is a new proposal
    return on_receive_proposal(Arc::new(p), cx).await;
}