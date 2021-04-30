use std::sync::Arc;

use types::optsync::ProtocolMsg;
use crate::node::{
    context::Context,
    proposal::on_receive_proposal,
    vote::on_vote,
    commit::on_commit,
};

pub(crate) async fn process_msg(cx: &mut Context, protmsg: ProtocolMsg) {
    log::debug!("Received protocol message: {:?}", protmsg);
    let decision;
    if let ProtocolMsg::NewProposal(p) = protmsg {
        log::debug!("Received a proposal: {:?}", p);
        let p = Arc::new(p);
        decision = on_receive_proposal(p.clone(), cx).await;
        log::debug!("Decision for the incoming proposal is {}", decision);
        if decision {
            cx.commit_queue.insert(p, cx.d2);
        }
    }
    else if let ProtocolMsg::VoteMsg(v,mut p) = protmsg {
        log::debug!("Received a vote for a proposal: {:?}", v);
        decision = on_vote(v, &mut p, cx).await;
        if decision {
            log::debug!("Optimistically committing block");
            // commit p.block_hash and all its children
            on_commit(Arc::new(p),cx).await;
        }
    }
}