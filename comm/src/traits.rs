use serde::{Serialize, de::DeserializeOwned};
use async_trait::async_trait;
use crate::NetError;

pub trait Message: Send + Sync + Serialize + DeserializeOwned + 'static
{
    /// How to decode from bytes
    fn from_bytes(data: &[u8]) -> Self;

    // How to encode self to bytes
    fn to_bytes(&self) -> Vec<u8>;
}

/// Networking channel abstraction
pub trait NetSender<PeerId, Out> 
where
    Out: Message,
{
    /// This function sends message `msg` to `sender` asynchronously
    fn send(&self, sender: &PeerId, msg: Out);

    /// This function sends `msg` to `sender` synchronously
    fn blocking_send(&self, sender:&PeerId, msg: Out);
    
    /// This function sends `msg` to all **known** nodes asynchronously.
    /// Messages sent to nodes that are currently disconnected will be sent once they are re-connected.
    fn broadcast(&self, msg: Out);

    /// This function sends `msg` to all **known** nodes synchronously.
    /// Messages sent to nodes that are currently disconnected will be sent once they are re-connected.
    fn blocking_broadcast(&self, msg: Out);
}

#[async_trait]
pub trait NetReceiver<In> 
where
    In: Message,
{
    async fn recv(&mut self) -> NetError;
}