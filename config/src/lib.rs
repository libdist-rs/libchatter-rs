mod client;
pub use client::*;

mod node;
pub use node::*;

mod error;
pub use error::*;

fn is_valid_replica(r:types::Replica, n:usize) -> bool {
    n>r as usize
}

/// The amount of time to sleep to be sure that all the other nodes of the group
/// are up and listening.
///
/// We dont have issues in initial synchronization even if the nodes are
/// sleeping, as long as all the sockets are open 
pub static mut SLEEP_TIME: u64 = 45;