 // TODO: Discover all the nodes
// mDNS or custom UDP broadcast
// TODO: 
// 1. One node acts a central Peer manager
// 2. All nodes register with the central Peer Manager
// 3. Peer Manager on connecting with n-1 nodes sends the connectivity information
// let listener = ..::bind("0.0.0.0:0");
// let (ip, port) = if let SocketAddr::V4(c) = listener.local_addr() {
    // c.ip(), c.port()
// } else {
    // panic!();
// };