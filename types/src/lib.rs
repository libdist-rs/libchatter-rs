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
use tokio_util::codec::{Decoder, Encoder};

pub type View = u64;

/// A wire trait tells us that the object can be encoded to/decoded from the
/// network.
pub trait WireReady: Send + Sync {
    /// How to decode from bytes
    fn from_bytes(data: &[u8]) -> Self;

    /// How to initialize self
    fn init(self) -> Self;
}

// /// A trait that defines how we can identify ourselves to others, and
// /// how others can identify us
pub trait EnCodec<T,U>
where U: Encoder<T>+Send+'static
{
    fn encoder() -> U;
}

pub trait DeCodec<T>: Decoder<Item=T,Error=std::io::Error>+Send+'static {}