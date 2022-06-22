use std::ops::Add;

use fnv::FnvHashMap;
use serde::{Serialize, Deserialize};

type Port = u16;
type IpAddr = std::net::IpAddr;
type Address = std::net::SocketAddr;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TcpConfig<Id> 
where
    Id: std::cmp::Eq + std::hash::Hash,
{
    my_id: Id,

    // Seed connections
    seed_connections: FnvHashMap<Id, Address>,
}

impl<Id> TcpConfig<Id> 
where
    Id: std::cmp::Eq+ std::hash::Hash + Copy,
{

    pub fn new(id: Id) -> Self {
        let my_conn = Address::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0,0,0)), 0);
        let mut map = FnvHashMap::default();
        map.insert(id.clone(), my_conn);
        Self { my_id: id, seed_connections: map }
    }

    pub fn set_port(&mut self, port: Port) {
        let addr = self
                                        .seed_connections
                                        .get_mut(&self.my_id)
                                        .unwrap();
        *addr = Address::new(addr.ip(), port);
    }

    pub fn set_ip(&mut self, ip: IpAddr) {
        let addr = self
                                        .seed_connections
                                        .get_mut(&self.my_id)
                                        .unwrap();
        *addr = Address::new(ip, addr.port());
    }

    pub fn add_peer(&mut self, id: Id, addr: Address) {
        self.seed_connections.insert(id, addr);
    }

    pub fn add_peers(&mut self, peers: FnvHashMap<Id, Address>) {
        for (id, addr) in peers {
            self.add_peer(id, addr);
        }
    }

    pub fn get_peers(&self) -> &FnvHashMap<Id, Address> {
        &self.seed_connections
    }

    pub fn get_id(&self) -> &Id {
        &self.my_id
    }

    pub fn get_my_addr(&self) -> Address {
       *self.seed_connections
            .get(&self.my_id)
            .unwrap()
    }
}

    // // SAFETY: Keep these strings alive until the config is alive
    // //
    // // Holds the Ips of all the nodes
    // ip_map: HashMap<usize, IP>,