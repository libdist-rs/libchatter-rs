use super::context::Context;

pub async fn do_commit(cx: &mut Context) {
    log::debug!("Trying to commit blocks");

    // Add all parents if not committed already
    let commit_round = cx.round() - cx.num_faults();
    let p = cx.prop_chain_by_round.get(&commit_round).unwrap();
    let mut hash = p.block_hash;
    while !cx.storage.is_committed_by_hash(&hash) {
        let b_rc = cx.storage.delivered_block_from_hash(&hash).unwrap();
        cx.storage.add_committed_block(b_rc.clone());
        hash = b_rc.header.prev;
    }

    if !cx.is_client_apollo_enabled() {
        let p = p.clone();
        cx.multicast_client(p).await;
    }
}