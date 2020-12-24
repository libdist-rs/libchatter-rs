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
use tokio_util::codec::{Encoder, Decoder};

pub const LIBP2P_MULTIADDR_FMT:&str = "/ip4/0.0.0.0/tcp";

pub type View = u64;

/// A wire trait tells us that the object can be encoded to/decoded from the
/// network.
pub trait WireReady<T>: Decoder + Encoder<T> {}