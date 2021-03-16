use types::{Block, Height, Propose, ProtocolMsg};
use std::sync::Arc;
use super::context::Context;
use futures::SinkExt;

/// This function forwards the proposal/relays to the other nodes
/// and updates the context (by committing blocks)
///
/// Note: Call this function after ensuring the proposal is delivered
/// Note: Must be only called once for every proposal
pub async fn on_finish_propose(
    p_arc: Arc<Propose>, 
    is_forward: bool,
    cx: &mut Context
) 
{
    log::debug!(target:"consensus","Finishing proposal: {:?}", p_arc);
    log::debug!(
        "Last seen {:?}", cx.last_seen_block.header);

    let new_block = p_arc.block.clone().unwrap();

    // Send or relay the block to the others
    let forward_leader = cx.next_of(new_block.header.author);
    let mut send_p = cx.net_send.clone();
    let p_copy = p_arc.clone();
    let myid = cx.myid;
    let ship_b_ref = new_block.clone();
    let forward_handle = tokio::spawn(async move {
        let p_copy = p_copy.as_ref().clone();
        if forward_leader == myid {
            return;
        }
        let msg = if is_forward {
            Arc::new(
                ProtocolMsg::RawNewProposal(
                        p_copy, 
                        ship_b_ref.as_ref().clone()
                    )
            )
        } else {
            Arc::new(
                // ProtocolMsg::RawRelay(
                ProtocolMsg::Relay(
                    p_copy, 
                    // ship_b_ref.as_ref().clone(),
                )
            )
        };
        if let Err(e) = send_p.send(
            (
                forward_leader, 
                msg
            )
        ).await {
            log::error!(
                "Failed to forward proposal to the next leader: {}", e);
        }
    });

    // Update the leader
    cx.last_leader = new_block.header.author;
    cx.prop_chain.insert(p_arc.block_hash, p_arc.clone());

    debug_assert_eq!(cx.last_seen_block.hash, new_block.header.prev, 
        "blocks {:?} {:?} must be delivered before this step", cx.last_seen_block, new_block);
    debug_assert_eq!(cx.last_seen_block.header.height+1, 
        new_block.header.height, "blocks must be processed in order");
    
    cx.last_seen_block = new_block.clone();

    let committed_block = do_commit(cx);
    if cx.is_client_apollo_enabled {
        send_client(p_arc, cx).await;
    } else if committed_block.is_some() {
        let committed_block = &committed_block.unwrap();
        if committed_block.header.height > 0 {
            let send_arc = cx.prop_chain.get(&committed_block.hash).unwrap();
            send_client(send_arc.clone(), cx).await;
        }
    }
    if let Err(e) = forward_handle.await {
        println!("Failed to forward the block to the next leader: {}", e);
    }

    log::debug!("Next leader is: {}", cx.next_leader());
}

pub fn do_commit(cx: &mut Context) -> Option<Arc<Block>> {
    log::debug!("Trying to commit blocks");

    let new_block = cx.last_seen_block.as_ref();
    let last_seen_height = new_block.header.height;

    // Do we have any blocks to commit?
    // Commit if new_block_ht >= f
    // Or return if new_block_ht < f 
    if last_seen_height < cx.num_faults as Height
    {
        log::info!("Nothing to commit");
        return None;
    }

    // assert_eq!(last_committed_height+cx.num_faults as u64, 
    //     new_block.header.height, 
    //     "There should be a difference of f+1 between the last committed block and the latest proposal");

    // Add all parents if not committed already
    let commit_height = last_seen_height - cx.num_faults as Height;

    let block=  match cx.storage.delivered_block_from_ht(commit_height)
    {
        None => panic!("Could not find missing parent for block:{:?}",commit_height),
        Some(x) => x,
    };

    // commit block
    cx.storage.add_committed_block(block.clone());
    Some(block)
}

/// Based on whether or not Apollo client is enabled or not, notify the client
/// of the proposal
pub async fn send_client(p_arc: Arc<Propose>, cx: &Context) {
    log::debug!("Sending {:?} to the client", p_arc);

    let mut cli_send_p = cx.cli_send.clone();
    let ship_p= p_arc.clone();
    let cli_send = tokio::spawn(async move {
        let res = cli_send_p.send(ship_p).await;
        if let Err(e) = res {
            log::error!(target:"consensus",
                "Error sending to the clients: {}", e);
        }
    });

    if let Err(e) = cli_send.await {
        println!("Failed to send the block to the client: {}", e);
    }
}