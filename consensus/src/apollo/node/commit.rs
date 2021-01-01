use types::{Block, Propose, ProtocolMsg};
use std::sync::Arc;
use std::borrow::Borrow;

use super::context::Context;

pub async fn on_finish_propose(p: &Propose, cx: &mut Context) {
    // println!("On finish: {:?} from [{}]", p.new_block.header, cx.myid);
    // println!("Last seen {:?} from {}", cx.last_seen_block.header, cx.myid);
    let forward_leader = cx.next_of(p.new_block.header.author);
    let send_p = cx.net_send.clone();
    let p_copy = p.clone();
    let myid = cx.myid;
    let new_block = &p.new_block;
    let new_block_ref = Arc::new(new_block.clone());

    let forward_handle = tokio::spawn(async move {
        if forward_leader == myid {
            return;
        }
        if let Err(e) = send_p.send(
            (
                forward_leader, 
                Arc::new(ProtocolMsg::NewProposal(p_copy))
            )
        ).await {
            println!("Failed to forward proposal to the next leader: {}", e);
        }
    });
    cx.height = new_block.header.height;
    cx.storage.all_delivered_blocks_by_hash.insert(
        new_block.hash, new_block_ref.clone());
    cx.storage.all_delivered_blocks_by_ht.insert(
        new_block.header.height, new_block_ref.clone());
    // change the leader
    cx.last_leader = new_block.header.author;

    assert_eq!(cx.last_seen_block.hash, new_block.header.prev, "blocks must be delivered before this step");
    assert_eq!(cx.last_seen_block.header.height+1, 
        new_block.header.height, "blocks must be processed in order");
    cx.last_seen_block = Arc::new(new_block.clone());
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
    if !cx.storage.all_delivered_blocks_by_ht.contains_key(
        &commit_height) 
    {
        println!("Could not find missing parent for block:{:?}",commit_height);
        return;
    };
    // commit block
    cx.last_committed_block_ht = commit_height;
    let block = cx.storage.all_delivered_blocks_by_ht
        .get(&commit_height)
        .expect("we committed this block. It must be delivered");
    cx.storage.committed_blocks_by_ht.insert(commit_height, block.clone());
    cx.storage.committed_blocks_by_hash.insert(block.hash, block.clone());

    let cli_block = if cx.is_client_apollo_enabled {
        new_block_ref
    } else {
        block.clone()
    };
    let cli_send_p = cx.cli_send.clone();
    let cli_send = tokio::spawn(async move {
        let cli_block = (cli_block.borrow() as &Block).clone();
        let res = cli_send_p.send(cli_block).await;
        if let Err(e) = res {
            print!("Error sending to the clients: {}", e);
        }
    });

    // The server need not wait for the client to get the blocks, it can proceed
    // to handling the next proposal
    
    if let Err(e) = cli_send.await {
        println!("Failed to send the block to the client: {}", e);
    }
    if let Err(e) = forward_handle.await {
        println!("Failed to forward the block to the next leader: {}", e);
    }
}