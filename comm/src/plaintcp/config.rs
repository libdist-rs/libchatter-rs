use serde::{Serialize, Deserialize};
use crate::Config;
use fnv::FnvHashMap as HashMap;

type PORT = u16;

/// The config struct has a lifetime as long as the bytes from which it is deserialized is valid
/// NOTE: In general, try to use &' str if most of the struct is strings, otherwise use Strings
/// Pros: Zero copy after loading the bytes from which it is deserialized.
/// Cons: Some extra memory may also be kept alive if the bytes buffer is large
#[derive(Debug, Serialize, Deserialize)]
pub struct TcpConfig<'a> {
    // The TcpConfig will wait for `ip_map.len() - faults` connections to be ready before proceeding
    faults: usize,

    // The port in which I should listen to, the caller will listen at `0.0.0.0:<my_port>`
    pub(crate) my_port: PORT,

    // SAFETY: Keep these strings alive until the config is alive
    //
    // Holds the Ips of all the nodes
    #[serde(borrow)]
    ip_map: HashMap<usize, &'a str>,

    // My Id
    myid: &'a str,
}

impl<'a> Config for TcpConfig<'a> {
    type PeerId = &'a str;

    // The number of nodes is the number of elements in the `ip_map`
    fn get_num_nodes(&self) -> usize {
        self.ip_map.len()
    }

    // The number of faults is the value in `faults`
    fn get_num_faults(&self) -> usize {
        self.faults
    }

    // 
    fn get_id(&self) -> Self::PeerId {
        self.myid
    }    
}

impl<'a> TcpConfig<'a> {
    pub fn get_my_port(&self) -> PORT {
        self.my_port
    }

    pub fn set_my_port(&mut self, port: PORT) {
        self.my_port = port;
    }
}