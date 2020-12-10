use bytes::{BytesMut, Bytes};
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};
use types::{Transaction};

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

// ========================================================
// We encode transactions, and decode transactions
// ========================================================

impl Encoder<Transaction> for super::EnCodec {
    type Error = Error;

    fn encode(&mut self, item: Transaction, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let buf = Bytes::from(to_bytes(&item));
        return self.0.encode(buf, dst);
    }
}

impl Decoder for Codec {
    type Item = Transaction;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.0.decode(src)? {
            Some(in_data) => Ok(Some(
                Transaction::from_bytes(&in_data)
            )),
            None => Ok(None),
        }
    }
}