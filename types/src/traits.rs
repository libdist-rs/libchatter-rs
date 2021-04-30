use crypto::hash::Hash;
use crate::{Height, Replica};
use std::sync::Arc;

/// Block trait, anything that claims itself to be a block must satisfy these traits
pub trait BlockTrait 
{
    /// A method to get the hash of the function
    fn get_hash(&self) -> Hash;

    /// A method that returns the height of this block
    fn get_height(&self) -> Height;

    /// Return the node id that created this block
    fn get_author(&self) -> Replica;
}

/// Transaction trait, anything that can compute its own hash
pub trait TxTrait
{
    /// A method to get the hash of this transaction
    fn get_hash(&self) -> Hash;
}

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