use types::Transaction;

pub mod io;
pub mod codec;

pub fn new_dummy_tx(i:u64, payload:usize) -> Transaction {
    log::trace!(target:"util", "Creating a dummy transaction with payload {}", payload);
    let t = Transaction{
        data: i.to_be_bytes().to_vec(),
        request: vec![1;payload],
    };
    log::trace!(target:"util", "Created dummy transaction {:?}", t);
    t
}

pub const CHANNEL_SIZE:usize = 100_000;