use types::{CertType, Certificate, Replica, synchs::ProtocolMsg};
use std::{collections::HashSet, sync::Arc};
use super::{context::Context, phase::Phase};

/// We have decided to switch the view
/// Send the highest certified block
pub async fn do_status(cx: &mut Context) {
    log::debug!("Sending highest certificate {:?}", cx.last_seen_cert);

    cx.net_send.send(
        (cx.num_nodes as Replica, 
                Arc::new(
                    ProtocolMsg::StatusMsg(cx.last_seen_cert.as_ref().clone())
                )
            )
    ).unwrap();

    // Change the leader and before checking if I am the new leader
    cx.change_leader();
    log::debug!("The next leader is {}", cx.next_leader());

    // Change the phase
    cx.phase = if cx.myid == cx.next_leader() {
        cx.event_queue.insert(Phase::StatusWait, std::time::Duration::from_millis(2*cx.delay));
        Phase::StatusWait
    } else {
        Phase::ProposeWait
    };
}

pub async fn on_recv_status(cert: Certificate, cx: &mut Context) {
    // Check if the certificate is valid
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

    let mut unique_votes = HashSet::new();
    for vote in &cert.votes {
        if cx.vote_map.contains_key(&block_hash) {
            continue;
        }
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

    let block = cx.storage.delivered_block_from_hash(&block_hash).unwrap();
    if block.header.height > cx.last_seen_block.header.height {
        cx.last_seen_cert = Arc::new(cert);
        cx.last_seen_block = block;
    }
}