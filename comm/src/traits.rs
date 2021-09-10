use serde::{Serialize, de::DeserializeOwned};

pub trait Config {
    type PeerId;

    /// Returns the number of nodes
    fn get_num_nodes(&self) -> usize;

    /// Returns the number of faults tolerated by the system
    fn get_num_faults(&self) -> usize;

    /// Returns the Id of the node
    /// The Id of a node satisfies 0 <= Id < num_nodes
    fn get_id(&self) -> Self::PeerId;
}

pub trait Message: Send + Sync + Serialize + DeserializeOwned + 'static
{
    /// How to decode from bytes
    fn from_bytes(data: &[u8]) -> Self;

    /// How to initialize self
    fn init(self) -> Self;

    // How to encode self to bytes
    fn to_bytes(&self) -> Vec<u8>;
}

pub trait Context {
}

pub trait Communication {
    type Config: Config;
    type SendMsg: Message;
    type RecvMsg: Message;
    type Context: Context;

    fn init(config: Self::Config, ctx: Self::Context) -> Result<Box<Self>, String>;
    fn send(&self, sender: &<Self::Config as Config>::PeerId, msg: Self::SendMsg);
    fn concurrent_send(&self, sender:usize, msg: Self::SendMsg);
    fn broadcast(&self, msg: Self::SendMsg);
    fn concurrent_broadcast(&self, msg: Self::SendMsg);
}