use tokio::sync::RwLock;
use std::sync::Arc;
use fnv::FnvHashMap as HashMap;

const DEFAULT_RETRY_DURATION: std::time::Duration 
    = std::time::Duration::from_millis(1);
    
#[derive(Debug, Default)]
/// This tcp context assumes that the all peers have the same IP and that the IPs do not change
/// New peers with new IPs may join, but every peer has the same IP
pub struct TcpContext {
    /// The amount of time I need to wait before attempting another connection attempt
    retry_duration: std::time::Duration,

    /// The set of active connections
    connections: Arc<RwLock<HashMap<String, ()>>>,
}