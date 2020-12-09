use std::time::Duration;

use tokio::sync::mpsc::{channel, Sender, Receiver};
use types::{Block};
// use crypto::hash::Hash;
use futures::stream::{FuturesUnordered, StreamExt};
// use super::context::Context;

/// This file implements the timers for Sync HotStuff.
///
/// We do it by running a green thread that manages time based message
/// You can cancel the timer by sending a message to the manager
/// You can add new blocks for waiting by sending the block with view number to
/// the manager. 
/// 
/// If block timers expire after cancellation, they will ignored.
/// 

pub enum InMsg {
    Start,
    Cancel,
    NewTimer(Block),
}

pub enum OutMsg {
    Timeout(Block),
}

/// The time is in milliseconds
pub async fn manager(time: u64) -> (Sender<InMsg>, Receiver<OutMsg>) {
    let (in_send, mut in_recv) = channel::<InMsg>(100_000);
    let (out_send, out_recv) = channel::<OutMsg>(100_000);
    let mut timers = FuturesUnordered::new();
    let send_copy = out_send.clone();
    tokio::spawn(async move {
        let new_send = send_copy;
        loop {
            let copy = new_send.clone();
            tokio::select! {
                x_opt = in_recv.recv() => {
                    match x_opt {
                        None => {break;},
                        Some(InMsg::Cancel) => {
                            break;
                        },
                        Some(InMsg::NewTimer(b)) => {
                            timers.push(async move {
                                tokio::time::sleep(
                                    Duration::from_millis(time)
                                ).await;
                                return b;
                            });
                        },
                        Some(InMsg::Start) => {
                            println!("Already running");
                        }
                    }
                },
                s = timers.next() => {
                    match s {
                        None => {},
                        Some(b) => {
                            if let Err(_e) = copy.send(OutMsg::Timeout(b)).await {
                                println!("Error: {} while sending timeout block", _e);
                                break;
                            }
                        },
                    }
                },
            }
        }
    });
    (in_send, out_recv)
}