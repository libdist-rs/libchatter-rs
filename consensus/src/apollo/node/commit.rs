use types::{Block, Propose, ProtocolMsg};
use std::sync::Arc;
use std::borrow::Borrow;

use super::context::Context;

pub async fn on_finish_propose(
    p_arc: Arc<Propose>, 
    is_forward: bool,
    cx: &mut Context
) 
{
    log::debug!(target:"consensus","On finish propose: {:?}", p_arc);
    
    let new_block = p_arc.block.clone().unwrap();
    log::trace!(target:"consensus","On finish: {:?} from [{}]", 
        new_block.header, cx.myid);
    log::debug!(target:"consensus", "Last seen {:?} from {}", 
        cx.last_seen_block.header, cx.myid);
    
    let p = p_arc.borrow() as &Propose;

    let forward_leader = cx.next_of(new_block.header.author);
    let send_p = cx.net_send.clone();
    let p_copy = p.clone();
    let myid = cx.myid;
    let new_block_ref = new_block.clone();
    let blk_ship = new_block_ref.clone();

    let forward_handle = tokio::spawn(async move {
        if forward_leader == myid {
            return;
        }
        let msg = if is_forward {
            let blk_to_send = (blk_ship.borrow() as &Block).clone();
            Arc::new(ProtocolMsg::RawNewProposal(p_copy, blk_to_send))
        } else {
            Arc::new(ProtocolMsg::Relay(p_copy))
        };
        if let Err(e) = send_p.send(
            (
                forward_leader, 
                msg
            )
        ) {
            println!("Failed to forward proposal to the next leader: {}", e);
        }
    });
    cx.height = new_block.header.height;
    // change the leader
    cx.last_leader = new_block.header.author;

    assert_eq!(cx.last_seen_block.hash, new_block.header.prev, 
        "blocks must be delivered before this step");
    assert_eq!(cx.last_seen_block.header.height+1, 
        new_block.header.height, "blocks must be processed in order");
    cx.last_seen_block = new_block_ref.clone();
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
    // if !cx.storage.all_delivered_blocks_by_ht.contains_key(
        // &commit_height) 
    if !cx.storage.is_delivered_by_ht(commit_height)
    {
        panic!("Could not find missing parent for block:{:?}",commit_height);
    };
    
    // commit block
    cx.last_committed_block_ht = commit_height;
    let block = cx.storage
        .delivered_block_from_ht(commit_height)
        .expect("we committed this block. It must be delivered");
    cx.storage.add_committed_block(block.clone());

    let is_client_apollo_enabled = cx.is_client_apollo_enabled;
    let cli_block = if cx.is_client_apollo_enabled {
        new_block_ref
    } else {
        block
    };
    let cli_send_p = cx.cli_send.clone();
    let ship_p= p_arc.clone();
    let cli_send = tokio::spawn(async move {
        if is_client_apollo_enabled {
            let res = cli_send_p.send(ship_p);
            if let Err(e) = res {
                print!("Error sending to the clients: {}", e);
            }
        } else {
            let res = cli_send_p.send(ship_p);
            if let Err(e) = res {
                print!("Error sending to the clients: {}", e);
            }
        }
        // let cli_block = (cli_block.borrow() as &Block).clone();
        
    });

    // The server must wait for the client to get the blocks, it can proceed
    // to handling the next proposal
    
    if let Err(e) = cli_send.await {
        println!("Failed to send the block to the client: {}", e);
    }
    if let Err(e) = forward_handle.await {
        println!("Failed to forward the block to the next leader: {}", e);
    }
}