use std::net::SocketAddr;

use tokio::net::TcpStream;
use types::{Block, Certificate, Propose, Transaction, Vote};

#[derive(Debug, Clone)]
pub enum ApolloEvent {
    // A timer that we set for x seconds has timed out
    TimeOut(u64),
    // Some replica is blaming someone
    Blame(Vote),
    // Certified blame
    BlameCertified(Certificate),
    // We have a proposal with a new block
    Proposal(Propose),
    // Pool is full
    PoolFull,
}

#[derive(Debug)]
pub enum ClientEvent {
    // We have a new transaction
    Tx(Transaction),
    // New client joined
    Client(TcpStream, SocketAddr),
    // NewBlock
    BlockAck(Block),
}