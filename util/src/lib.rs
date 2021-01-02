use types::Transaction;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub mod io;
pub mod codec;
// pub mod channel;

pub fn new_dummy_tx(i:u64, payload:usize) -> Transaction {
    Transaction{
        data: i.to_be_bytes().to_vec(),
        request: vec![1;payload],
    }
}

pub const CHANNEL_SIZE:usize = 100_000;