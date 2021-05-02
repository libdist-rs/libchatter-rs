// Artemis module

mod block;
pub use block::*;

mod proto;
pub use proto::*;

mod ucr;
pub use ucr::*;

mod client;
pub use client::*;

pub type Height = crate::Height;
pub type Transaction = super::Transaction;
pub type Vote = super::Vote;
pub type Replica = crate::Replica;
pub type Storage = super::Storage<Block, Transaction>;
pub type Payload = super::Payload;
/// View number refers to which leader we are at, right now
pub type View = crate::View;
/// Round number refers to the UCR round
pub type Round = usize;