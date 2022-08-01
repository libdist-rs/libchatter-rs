use std::net::SocketAddr;

use fnv::FnvHashMap;
use serde::{Serialize, Deserialize};

use crate::Identifier;

/// Standard port numbers
type Port = u16;

/// `IpAddr` is a IPV4 or IPV6 address
type IpAddr = std::net::IpAddr;


/// `Address` is an internet SocketAddress (IPv4 or IPv6)
type Address = std::net::SocketAddr;

/// A TcpConfig contains Networking information about the current node
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TcpConfig<Id>
where
    Id: Identifier,
{
    // Identifier of whoever owns this config
    my_id: Id,

    // Known peers
    peers: FnvHashMap<Id, Address>,
}

impl<Id> TcpConfig<Id> 
where
    Id: Identifier
{

    pub fn new(id: Id) -> Self {
        let my_conn = Address::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0,0,0)), 0);
        let mut map = FnvHashMap::default();
        map.insert(id.clone(), my_conn);
        Self { my_id:id, peers: map }
    }

    pub fn add_peer(&mut self, id: Id, addr: Address) {
        self.peers.insert(id, addr);
    }

    pub fn add_peers(&mut self, peers: FnvHashMap<Id, Address>) {
        for (id, addr) in peers {
            self.add_peer(id, addr);
        }
    }

    pub fn get_my_addr(&self) -> &SocketAddr {
        self
            .peers
            .get(&self.my_id)
            .expect("Self key is expected in a TcpConfig, check TcpConfig construction!")
    }

    pub fn get_id(&self) -> &Id {
        &self.my_id
    }

    pub fn set_port(&mut self, port:Port) {
        self
            .peers
            .get_mut(&self.my_id)
            .expect("Self key is expected in a TcpConfig, check TcpConfig construction!")
            .set_port(port)
    }

    pub fn get_peers(&self) -> &FnvHashMap<Id, Address> {
        &self.peers
    }
}