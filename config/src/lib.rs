mod client;
pub use client::*;

mod node;
pub use node::*;

mod error;
pub use error::*;

fn is_valid_replica(r:types::Replica, n:usize) -> bool {
    n>r as usize
}

pub static mut SLEEP_TIME: u64 = 10;