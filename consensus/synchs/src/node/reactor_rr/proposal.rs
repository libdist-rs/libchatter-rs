use fnv::{FnvHashMap as HashMap, FnvHashSet as HashSet};
use super::{context::Context, phase::Phase};
use crypto::hash::EMPTY_HASH;
use types::synchs_rr::{
    Block, CertType, Certificate, Transaction, Vote, 
    Propose, ProtocolMsg
};
use std::sync::Arc;

pub fn check_proposal(p: Arc<Propose>, cx:&Context) -> bool {
    let new_block = p.block.clone().unwrap();
    // Check if the author is correct
    if new_block.header.author != cx.next_leader() {
        log::warn!(
            "Got a proposal from an incorrect leader for the view");
        return false;
    }

    // Check if the first block extends the genesis block
    if new_block.header.height == 1 &&
        new_block.header.prev != EMPTY_HASH 
    {
        log::warn!(
            "First block does not extend the genesis block");
        return false;
    }

    // Check if the block has sufficient votes
    if new_block.header.height > 1 && p.cert.votes.len() <= cx.num_faults {
        log::warn!(
            "Insufficient votes in the proposal, rejecting the proposal");
        return false;
    }

    // Check signature for the proposal
    let pk = cx.pub_key_map.get(&new_block.header.author).unwrap();
    if !pk.verify(&new_block.hash, &p.proof) {
        log::warn!(
            "Got an incorrectly signed block");
        return false;
    }

    // Check if the view is correct
    if cx.view != p.view {
        panic!("This view check should be unreachable");
    }

    // If the parent is genesis block, then the parent is correctly certified
    if new_block.header.height == 1 {
        return true;
    }
    // Otherwise check if all the parent certificates are correctly signed
    let mut uniq_votes = HashSet::with_capacity_and_hasher(cx.num_faults+1, Default::default());
    if let CertType::Vote(v, h) = &p.cert.msg {
        // Check if vote message is the same as that in the proposal
        if *h != new_block.header.prev {
            log::warn!(
                    "The message of the vote is not the hash of the proposed block's prev");
            return false;
        } else if *v < cx.view-1 {
            log::warn!("Got a vote {:?} from an old view", p.cert);
            return false;
        }
    } else {
        // Invalid certificate
        log::warn!("Invalid certificate: {:?}", p.cert);
        return false;
    }

    log::debug!("Checking certificate: {:?}", p.cert);
    let data = util::io::to_bytes(&p.cert.msg);
    for v in &p.cert.votes {
        // check signature
        let pk = match cx.pub_key_map.get(&v.origin) {
            None => {
                log::warn!("Invalid vote origin");
                return false;
            }
            Some(x) => x,
        };
        if !pk.verify(&data, &v.auth) {
            log::warn!("Invalid vote signature: {:?}", v);
            return false;
        }
        // Add this unique vote
        uniq_votes.insert(v.origin);
    }
    if uniq_votes.len() < cx.num_faults {
        log::warn!("Insufficient unique votes in certificate: {:?}", p.cert);
        return false;
    }
    // Is it extending the last known parent?
    if new_block.header.prev != cx.last_seen_block.hash {
        log::warn!("Parent undelivered");
        return false;
        // TODO add delivery
    }
    true
}

pub async fn on_receive_proposal(p: Arc<Propose>, cx: &mut Context) -> bool {
    // Use decision to start commit timers in the reactor
    let decision = false;
    let new_block = p.block.clone().unwrap();

    log::debug!("Received a proposal of height: {}", new_block.header.height);

    if cx.storage.is_delivered_by_hash(&new_block.hash) {
        log::debug!("We have already processed this block last time");
        return decision;
    }
    
    // On receiving a proposal, check if it is in the same view
    // Check for the validity
    if !check_proposal(p.clone(), cx) {
        log::warn!("Proposal checking failed");
        return decision;
    }
    return on_new_valid_proposal(p, cx).await;
}
    
pub async fn on_new_valid_proposal(p: Arc<Propose>, cx: &mut Context) -> bool {
    let mut decision = false;
    let new_block = p.block.clone().unwrap();

    // Is the parent delivered?
    if !cx.storage.is_delivered_by_hash(&new_block.header.prev) 
    {
        log::warn!(
            "We do not have the parent for this block");
        // TODO: Request and Deliver blocks
        return decision;
    }

    cx.phase = Phase::CollectVote;

    // Everything looks fine, initiate voting and continue to process this
    // proposal
    let mut my_vote = Certificate::empty_cert();
    my_vote.msg = CertType::Vote(cx.view, new_block.hash);
    let sign_data = util::io::to_bytes(&my_vote.msg);
    match cx.my_secret_key.sign(&sign_data) {
        Err(e) => {
            panic!("Failed to sign a vote: {}", e);
        },
        Ok(vo) => {
            let v = Vote{
                origin: cx.myid,
                auth: vo,
            };
            my_vote.votes.push(v);
        },
    };

    // Add self vote to the map
    let mut new_map = HashMap::default();
    new_map.insert(cx.myid, my_vote.clone());
    cx.vote_map.insert(new_block.hash, new_map);

    decision = true;

    // Create a scope and send the proposal to other nodes
    let ship = cx.net_send.clone();
    let ship_nodes = cx.num_nodes;
    let ship_v = ProtocolMsg::VoteMsg(my_vote, p.as_ref().clone());
    let vote_ship = tokio::spawn(async move {
        let msg = Arc::new(ship_v);
        if let Err(e) = ship.send(
            (ship_nodes, msg))
        {
            log::warn!(
                "failed to send vote: {}", e);
        }
    });

    // Update the consensus context
    cx.storage.add_delivered_block(new_block.clone());
    cx.storage.clear(&new_block.body.tx_hashes);
    cx.height = new_block.header.height;
    if cx.last_seen_block.header.height < new_block.header.height-1 {
        cx.last_seen_cert = Arc::new(p.cert.clone());
    }
    cx.last_seen_block = new_block.clone();

    // wait for voting to finish?
    if let Err(e) = vote_ship.await {
        log::warn!(
            "Failed to send vote to the others:{}", e);
        return decision;
    }

    log::debug!("Sent a vote to all the nodes");
    decision
}

pub async fn do_propose(txs: Vec<Arc<Transaction>>, cx: &mut Context) -> Arc<Propose> {
    // Build the proposal
    let parent = &cx.last_seen_block;
    let mut new_block = Block::with_tx(txs);

    // Update block contents here
    new_block.header.author = cx.myid;
    new_block.header.prev = parent.hash;
    new_block.header.height = parent.header.height+1;
    
    // Update the hash at the end
    new_block.hash = new_block.compute_hash();
    
    // Sign the block hash 
    let proof = match cx.my_secret_key.sign(&new_block.hash) {
        Err(e) => {
            panic!("Failed to sign the new proposal: {}", e);
        },
        Ok(sig) => sig,
    };

    let mut new_block_cert = Certificate::empty_cert();
    new_block_cert.msg = CertType::Vote(cx.view, new_block.hash);
    let sign_data = util::io::to_bytes(&new_block_cert.msg);
    let sig = match cx.my_secret_key.sign(&sign_data) {
        Err(e) => {
            panic!("Failed to sign the new proposal: {}", e);
        },
        Ok(sig) => sig,
    };
    // Add self vote to the certificate map
    let self_vote = Vote{
        origin: cx.myid,
        auth: sig,
    };

    new_block_cert.votes.push(self_vote);

    let new_block_cert = new_block_cert;

    // The block is ready, build proposal
    let new_block_ref = Arc::new(new_block.clone());
    let mut p = Propose::new();
    p.proof = proof;
    p.block = Some(new_block_ref.clone());
    p.block_hash = new_block_ref.hash;
    p.cert = match cx.cert_map.get(&parent.hash) {
        None => {
            panic!("Must call propose only if the parent is certified");
        },
        Some(x) => x.as_ref().clone(),
    };
    log::debug!("Proposing with view: {}", cx.view);
    p.view = cx.view;

    // Ship the proposal
    let ship = cx.net_send.clone();
    let ship_num = cx.num_nodes;
    let ship_p = ProtocolMsg::RawNewProposal(p.clone(), new_block_ref.as_ref().clone());
    let broadcast = tokio::spawn(async move {
        if let Err(e) = ship.send(
            (ship_num, Arc::new(ship_p))
        ) {
            println!("Error broadcasting the block to all the nodes: {}", e);
        }
    });

    // Update consensus context
    cx.storage.add_delivered_block(new_block_ref.clone());

    cx.phase = Phase::CollectVote;

    // Q) Can the leader commit immediately? 
    // A) NOOOO! I learnt it painfully! If the leader commits now, then it must
    // also acknowledge the client now, which becomes a problem!
    // Commit normally, and tell the client after 2\Delta
    let mut new_vote_map = HashMap::default();
    new_vote_map.insert(cx.myid,new_block_cert);
    cx.vote_map.insert(new_block_ref.hash, new_vote_map);
    cx.height = new_block_ref.header.height;
    // The leader remains the same
    cx.last_seen_block = new_block_ref.clone();
    cx.last_committed_block_ht = cx.height;
    // The view remains the same
    broadcast.await.expect("failed to broadcast the proposal");

    Arc::new(p)
}