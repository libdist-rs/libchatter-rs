use types::BlockTrait;
use super::*;

/// Do commit is called to trigger committing of blocks
/// Caller needs to ensure that `cx.round > cx.num_faults()`
pub fn do_commit(cx: &mut Context) {
    log::debug!("Trying to commit");
    debug_assert!(cx.round() > cx.num_faults());

    // Get the r-f^th vote
    let commit_round = cx.round() - cx.num_faults();
    let v = cx.vote_chain.get(&commit_round).unwrap();    

    let mut com_hash = v.hash;
    // Commit com_hash and its parents
    while !cx.storage.is_committed_by_hash(&com_hash) {
        let b = cx.storage.delivered_block_from_hash(&com_hash).unwrap();
        log::debug!("Committing block - {} in round {}", b.get_height(), v.round);
        cx.storage.add_committed_block(b.clone());
        com_hash = b.blk.header.prev;
    }
}