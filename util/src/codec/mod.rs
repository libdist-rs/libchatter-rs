use tokio_util::codec::LengthDelimitedCodec;

pub mod tx;
pub mod block;

pub mod fblock;
pub mod ftx;

pub mod proto;
pub mod synchs;

#[derive(Debug)]
pub struct EnCodec (pub LengthDelimitedCodec);

impl EnCodec {
    pub fn new() -> Self {
        EnCodec(LengthDelimitedCodec::new())
    }
}
