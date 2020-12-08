#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

mod client;
pub use client::*;

mod node;
pub use node::*;

mod error;
pub use error::*;

fn is_valid_replica(r:types::Replica, n:usize) -> bool {
    n>r as usize
}