use tokio::sync::RwLock;
use std::{sync::Arc, time::Duration};
use fnv::FnvHashMap as HashMap;

const DEFAULT_RETRY_DURATION: std::time::Duration 
    = std::time::Duration::from_millis(1);

const NUM_RETRIES: usize = 10_000;
    
#[derive(Debug, Clone)]
/// This tcp context assumes that the all peers have the same IP and that the IPs do not change
/// New peers with new IPs may join, but every peer has the same IP
pub struct TcpContext {
    /// The amount of time I need to wait before attempting another connection attempt
    retry_duration: std::time::Duration,

    /// The number of times to attempt re-connections before assuming that the node has died
    retries: usize,

    /// The set of active connections
    connections: Arc<RwLock<HashMap<String, ()>>>,
}

impl std::default::Default for TcpContext {
    fn default() -> Self {
        Self { 
            retry_duration: DEFAULT_RETRY_DURATION,  
            retries: NUM_RETRIES,
            connections: Arc::default(),
        }
    }
}

impl TcpContext {
    /// Returns the number of times a connection will be re-attempted after no or lost connection 
    pub fn get_retries(&self) -> usize {
        self.retries
    }

    /// Returns the wait time before an attempt at reconnecting with the peer
    pub fn get_retry_duration(&self) -> Duration {
        self.retry_duration
    }
}