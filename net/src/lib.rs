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
pub use read_all::*;