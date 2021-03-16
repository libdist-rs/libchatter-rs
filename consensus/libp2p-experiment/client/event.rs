use types::{Block, Transaction};

#[derive(Debug, Clone)]
pub enum OEvent {
    NewBlock(Block),
}

#[derive(Debug,Clone)]
pub enum IEvent {
    NewTx(Transaction),
}