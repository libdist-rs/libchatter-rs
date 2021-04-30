mod propose;
pub use propose::*;

mod proto;
pub use proto::*;

pub type Transaction = super::Transaction;
pub type Certificate = super::Certificate;
pub type CertType = super::CertType;
pub type Replica = crate::Replica;
pub type Vote = super::Vote;
pub type Block = super::Block;
pub const GENESIS_BLOCK: Block = super::GENESIS_BLOCK;
pub type Height = crate::Height;
pub type Storage = super::Storage<Block, Transaction>;
pub type View = crate::View;
pub type Payload = super::Payload;