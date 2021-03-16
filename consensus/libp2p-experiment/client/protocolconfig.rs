use libp2p::Multiaddr;

#[derive(Debug,Clone)]
pub struct ProtocolConfig {
    pub peers: Vec<Multiaddr>,
    pub blocksize: usize,
}