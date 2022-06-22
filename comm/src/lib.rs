#[macro_use]
extern crate derive_builder;

mod traits;
pub use traits::*;

mod error;
pub use error::*;

pub type NetResult<T> = std::result::Result<T, NetError>;

mod plaintcp;