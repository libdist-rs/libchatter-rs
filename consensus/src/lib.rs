use crossfire::mpsc::{RxFuture, SharedFutureBoth, TxFuture};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub mod apollo;
pub mod synchs;
pub mod dummy;

// mod start
type Sender<T> = TxFuture<T, SharedFutureBoth>;
type Receiver<T> = RxFuture<T, SharedFutureBoth>;
