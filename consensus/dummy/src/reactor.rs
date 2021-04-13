use tokio::sync::mpsc::{Sender, Receiver};
use types::{Block, Height, Propose, ProtocolMsg, Replica, Transaction};
use config::Node;
use std::sync::Arc;

pub async fn reactor(
    config:&Node,
    net_send: Sender<(Replica, ProtocolMsg)>,
    mut net_recv: Receiver<ProtocolMsg>,
    cli_send: Sender<Block>,
    mut cli_recv: Receiver<Transaction>
) {
    tokio::spawn(async move {
        loop {
             tokio::select! {
                 msg = net_recv.recv() => {
                     println!("Got a message from the server");
                     if let Some(m) = msg {
                         println!("Msg: {:?}", m);
                         continue;
                     }
                     if let None = msg {
                         break;
                     }
                 }
             }
         } 
     });
     let mut num_new_tx:u64 = 0;
     let mut height:Height = 1;
     let send_all = config.num_nodes;
     let mut txs = Vec::with_capacity(config.block_size);
     loop {
         // let txs = &mut txs;
         let tx_opt = cli_recv.recv().await;
         let tx = match tx_opt {
             Some(t) => t,
             None => break,
         };
         num_new_tx += 1;
         txs.push(Arc::new(tx));
         println!("Got new transactions [{}] from a client", num_new_tx);
         if txs.len() == config.block_size {
             let b = Block::with_tx(txs.drain(..).collect());
             if let Err(e) = cli_send.send(b.clone()).await {
                 println!("Failed to send the blocks to the clients");
                 println!("Error: {}", e);
             };
             let mut block = b;
             block.header.height = height;
             block.hash = block.compute_hash();
             let new_b = Propose::new(block.hash);
             if let Err(e) = net_send.send(
                 (send_all,ProtocolMsg::RawNewProposal(new_b, block))).await {
                 println!("Failed to send the blocks to the other servers");
                 println!("Error: {}", e);
             }
             height += 1;
         }
     }
}