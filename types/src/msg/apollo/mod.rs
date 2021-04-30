pub type Block = super::Block;
pub type ClientMsg = super::ClientMsg;
pub const GENESIS_BLOCK: Block = super::GENESIS_BLOCK;

pub type Height = crate::Height;
pub type Transaction = super::Transaction;
pub type Vote = super::Vote;
pub type Propose = super::Propose;
pub type ProtocolMsg = super::ProtocolMsg;
pub type Replica = crate::Replica;
pub type Storage = super::Storage<Block, Transaction>;
pub type Payload = super::Payload;