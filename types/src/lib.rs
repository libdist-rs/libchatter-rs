mod protocol;
pub use protocol::*;

mod msg;
pub use msg::*;
use tokio_util::codec::{Decoder, Encoder};
use std::sync::Arc;

pub type View = usize;

/// A wire trait tells us that the object can be encoded to/decoded from the
/// network.
pub trait WireReady: Send + Sync + Clone {
    /// How to decode from bytes
    fn from_bytes(data: &[u8]) -> Self;

    /// How to initialize self
    fn init(self) -> Self;

    // How to encode self to bytes
    fn to_bytes(&self) -> Vec<u8>;
}

impl<A> WireReady for Arc<A> 
where A:WireReady,
{
    fn from_bytes(data: &[u8]) -> Arc<A> {
        let a = A::from_bytes(data);
        Arc::new(a)
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.as_ref().to_bytes()
    }

    fn init(self) -> Self {
        let x = self.as_ref().clone();
        let y = x.init();
        Arc::new(y)
    }
}

/// A trait that defines how we can identify ourselves to others, and
/// how others can identify us
pub trait EnCodec<T,U>
where U: Encoder<T>+Send+'static
{
    fn encoder() -> U;
}

pub trait DeCodec<T>: Decoder<Item=T,Error=std::io::Error>+Send+'static {}