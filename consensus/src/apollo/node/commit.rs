use types::{Propose, ProtocolMsg};

use super::context::Context;

pub async fn on_finish_propose(p: &Propose, cx: &mut Context) {
    let forward_leader = cx.next_of(p.new_block.header.author);
    let send_p = cx.net_send.clone();
    let p_copy = p.clone();
    let myid = cx.myid;
    let forward_handle = tokio::spawn(async move {
        if forward_leader == myid {
            return;
        }
        if let Err(e) = send_p.send(
            (
                forward_leader, 
                ProtocolMsg::NewProposal(p_copy)
            )
        ).await {
            println!("Failed to forward proposal to the next leader: {}", e);
        }
    });
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

    // let cli_send = if cx.is_client_apollo_enabled {
    //     let cli_block = p.new_block.clone();
    //     let cli_send_p = cx.cli_send.clone();
    //     tokio::spawn(async move {
    //         if let Err(e) = cli_send_p.send(cli_block).await {
    //             print!("Error sending to the clients: {}", e);
    //         }
    //     })
    // } else {
    //     let cli_block = block;
    //     let cli_send_p = cx.cli_send.clone();
    //     tokio::spawn(async move {
    //         if let Err(e) = cli_send_p.send(*cli_block).await {
    //             print!("Error sending to the clients: {}", e);
    //         }
    //     })
    // };
    let cli_block = if cx.is_client_apollo_enabled {
        p.new_block.clone()
    } else {
        block.clone()
    };
    let cli_send_p = cx.cli_send.clone();
    let cli_send = tokio::spawn(async move {
        if let Err(e) = cli_send_p.send(cli_block).await {
            print!("Error sending to the clients: {}", e);
        }
    });

    if let Err(e) = cli_send.await {
        println!("Failed to send the block to the client: {}", e);
    }
    if let Err(e) = forward_handle.await {
        println!("Failed to forward the block to the next leader: {}", e);
    }
}