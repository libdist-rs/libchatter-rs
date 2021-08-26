use serde::{Serialize, Deserialize};
use crate::{Config, ConfigCommon};
use fnv::FnvHashMap as HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TcpConfig {
    n: usize,
    f: usize,
    ip_map: HashMap<usize, String>,
    listen_addr: String,
    myid: usize,
}

impl ConfigCommon for TcpConfig {
    fn get_num_nodes(&self) -> usize {
        self.n
    }

    fn get_num_faults(&self) -> usize {
        self.f
    }

    fn get_id(&self) -> usize {
        self.myid
    }
}

impl Config for TcpConfig {
    const NUM_RETRIES: usize = 300;
    const NO_DELAY: bool = true;
    const CONNECTION_SLEEP_TIME: u64 = 300;

    fn get_listen_addr(&self) -> String {
        self.listen_addr.clone()
    }

    fn get_node_addr(&self, node_id: &usize) -> String {
        self.ip_map[node_id].clone()
    }
}