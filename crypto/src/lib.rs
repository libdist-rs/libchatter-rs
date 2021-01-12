#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub mod hash;

mod crypto;
pub use crypto::*;

mod gen;
pub use gen::*;

pub mod ed25519;
pub mod error;
pub mod rsa;
pub mod secp256k1;