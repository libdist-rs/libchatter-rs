#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub mod client;
pub mod replica;

mod read_all;
// use crossfire::mpsc::{RxFuture, SharedFutureBoth, TxFuture};
pub use read_all::*;

pub mod peer;

// type Sender<T> = TxFuture<T, SharedFutureBoth>;
// type Receiver<T> = RxFuture<T, SharedFutureBoth>;