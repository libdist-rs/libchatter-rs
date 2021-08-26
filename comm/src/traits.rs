use serde::Serialize;
use async_trait::async_trait;

pub trait ConfigCommon {
    /// Returns the number of nodes
    fn get_num_nodes(&self) -> usize;

    /// Returns the number of faults tolerated by the system
    fn get_num_faults(&self) -> usize;

    /// Returns the Id of the node
    /// The Id of a node satisfies 0 <= Id < num_nodes
    fn get_id(&self) -> usize;
}

/// Network configuration
pub trait Config: Serialize + ConfigCommon {
    const NUM_RETRIES: usize;
    const NO_DELAY: bool;
    const CONNECTION_SLEEP_TIME: u64;

    fn get_listen_addr(&self) -> String;
    fn get_node_addr(&self, id: &usize) -> String;
}

pub trait Message: Send + Sync + Serialize + 'static
{
    /// How to decode from bytes
    fn from_bytes(data: &[u8]) -> Self;

    /// How to initialize self
    fn init(self) -> Self;

    // How to encode self to bytes
    fn to_bytes(&self) -> Vec<u8>;
}

#[async_trait]
pub trait Communication {
    type Config;

    async fn init(config: Self::Config) -> Self;
    fn send(sender: usize, msg: impl Message);
    fn concurrent_send(sender:usize, msg: impl Message);
    fn broadcast(msg: impl Message);
    fn concurrent_broadcast(msg: impl Message);
}