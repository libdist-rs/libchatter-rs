use futures_codec::{Decoder, Encoder, LengthCodec};
use types::{Transaction};

use futures_codec::{Bytes, BytesMut};
use std::io::{
    Error,
};
pub struct Codec (pub LengthCodec);

impl Codec {
    pub fn new() -> Self {
        Codec(LengthCodec{})
    }
}

// ========================================================
// We encode transactions, and decode transactions
// ========================================================

impl Encoder for Codec {
    type Item = Transaction;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let buf = Bytes::from(item.data);
        return self.0.encode(buf, dst);
    }
}

impl Decoder for Codec {
    type Item = Transaction;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.0.decode(src)? {
            Some(in_data) => Ok(Some(
                Transaction{data:in_data.to_vec()}
            )),
            None => Ok(None),
        }
    }
}