/// This is the stream->Relay->Sink reactor implementation for the Apollo paper
/// 
/// The goal of this experiment is to figure out the fundamental limitations of
/// a consensus protocol
/// 
/// The experiment consists of three parameters:
/// - B: The block size
/// - P: The payload size
/// - N: The number of nodes
/// - T: The number of transactions
/// 
/// STREAMER -->-->--( CLUSTER OF ALL NODES ) -->-->-- SINK 

pub mod stream;
pub mod sink;
pub mod relay;