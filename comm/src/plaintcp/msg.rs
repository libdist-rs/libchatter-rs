use std::net::SocketAddr;
use crate::{NetError, Identifier};

pub enum NetMsg<Id, SendMsg, RecvMsg> 
where
    Id: Identifier,
{
    NewMsg(Id, RecvMsg),
    SendMsg(SendMsg),

    // Networking events
    NewConnection(SocketAddr),
    ConnectionError(SocketAddr, NetError),
    SendingFailed(SendMsg),
}


pub enum PeerConnectionMsg<SendMsg, RecvMsg> 
{
    // Used by the peer thread
    NewMsg(RecvMsg),
    SendMsg(SendMsg),

    // Used by the connection thread 
    ConnectionError(NetError),
    SendingFailed(SendMsg, NetError),
}

