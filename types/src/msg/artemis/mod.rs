// Artemis module

mod block;
pub use block::*;

mod propose;
pub use propose::*;

mod proto;
pub use proto::*;

pub type Height = crate::Height;
pub type Transaction = super::Transaction;
pub type Vote = super::Vote;
pub type Replica = crate::Replica;
pub type Storage = super::Storage<Block, Transaction>;
pub type Payload = super::Payload;
pub type View = crate::View;