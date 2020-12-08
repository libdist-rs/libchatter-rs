use types::Propose;

use super::context::Context;

pub async fn on_finish_propose(p: &Propose, cx: &mut Context) {
    // println!("Proposing block with hash: {:?}", p.new_block.hash);
    let new_block = &p.new_block;
    cx.height = new_block.header.height;
    cx.storage.all_delivered_blocks_by_hash.insert(
        new_block.hash, new_block.clone());
    cx.storage.all_delivered_blocks_by_ht.insert(
        new_block.header.height, new_block.clone());
    // change the leader
    cx.last_leader = new_block.header.author;

    assert_eq!(cx.last_seen_block.hash, new_block.header.prev, "blocks must be delivered before this step");
    assert_eq!(cx.last_seen_block.header.height+1, 
        new_block.header.height, "blocks must be processed in order");
    cx.last_seen_block = new_block.clone();
    // Do we have any blocks to commit?
    if cx.height < cx.num_faults as u64 {
        println!("Nothing to commit");
        return;
    }
    assert_eq!(cx.last_committed_block_ht+cx.num_faults as u64, 
        new_block.header.height, 
        "There should be a difference of f+1 between the last committed block and the latest proposal");
    // Add all parents if not committed already
    let commit_height = cx.last_committed_block_ht + 1;
    let block = match cx.storage.all_delivered_blocks_by_ht.get(
        &commit_height) 
    {
        None => {
            println!("Could not find missing parent for block:{:?}",commit_height);
            return;
        },
        Some(x) => x,
    };
    // commit block
    cx.last_committed_block_ht = commit_height;
    if let Err(e) = cx.cli_send.send(block.clone()).await {
        print!("Error sending to the clients: {}", e);
    }
}