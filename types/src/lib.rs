#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}


mod protocol;
pub use protocol::*;

mod msg;
pub use msg::*;

pub const LIBP2P_MULTIADDR_FMT:&str = "/ip4/0.0.0.0/tcp";