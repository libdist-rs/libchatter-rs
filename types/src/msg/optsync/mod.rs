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

pub type ProtocolMsg = super::synchs::ProtocolMsg;
pub type Propose = super::synchs::Propose;
pub type ClientMsg = super::synchs::ClientMsg;