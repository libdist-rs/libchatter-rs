use serde::{Serialize, Deserialize};

use super::{PORT, IP};

/// The config struct has a lifetime as long as the bytes from which it is deserialized is valid
/// NOTE: In general, try to use &' str if most of the struct is strings, otherwise use Strings
/// Pros: Zero copy after loading the bytes from which it is deserialized.
/// Cons: Some extra memory may also be kept alive if the bytes buffer is large
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TcpConfig {
    // The port in which I should listen to, the caller will listen at `0.0.0.0:<my_port>`
    // If None is used, a default port requested from the OS is assigned
    my_port: Option<PORT>,

    // My Ip: If some(ip) is specified, then the ip is used. Eg. 127.0.0.1
    // Otherwise, "0.0.0.0" is used.
    my_ip: Option<IP>,
}

    // // SAFETY: Keep these strings alive until the config is alive
    // //
    // // Holds the Ips of all the nodes
    // ip_map: HashMap<usize, IP>,