use types::{Block};
// use tokio_codec::{Decoder, Encoder};
use tokio_util::codec::{Decoder, Encoder, length_delimited::LengthDelimitedCodec};
use bytes::{Bytes, BytesMut};
use std::io::{
    Error,
};

use crate::io::to_bytes;
pub struct Codec (pub LengthDelimitedCodec);

impl Codec {
    pub fn new() -> Self {
        Codec(LengthDelimitedCodec::new())
    }
}

impl Encoder<Block> for Codec {
    type Error = Error;

    fn encode(&mut self, item: Block, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let data = to_bytes(&item);
        let buf = Bytes::from(data);
        return self.0.encode(buf, dst);
    }
}

// The client only needs to decode blocks, and we use a length delimited decoder
// to decode a block sent from the server
impl Decoder for Codec {
    type Item = Block;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.0.decode(src)? {
            Some(data) => Ok(Some(Block::from_bytes(data.to_vec()))),
            None => Ok(None),
        }
    }
}