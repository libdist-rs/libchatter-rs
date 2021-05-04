mod proto;
pub use proto::*;

mod client;
pub use client::*;

mod propose;
pub use propose::*;

pub type Block = super::Block;
pub const GENESIS_BLOCK: Block = super::GENESIS_BLOCK;

pub type Vote = crate::Vote;
pub type Height = crate::Height;
pub type Transaction = super::Transaction;
pub type Replica = crate::Replica;
pub type Storage = super::Storage<Block, Transaction>;
pub type Payload = super::Payload;
pub type Round = usize;