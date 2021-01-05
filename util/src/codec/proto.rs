use bytes::{Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};
use types::ProtocolMsg;

use std::{io, sync::Arc, borrow::Borrow};

use crate::io::to_bytes;

#[derive(Debug)]
pub struct Codec (pub LengthDelimitedCodec);

impl Codec {
    pub fn new() -> Self {
        Codec(LengthDelimitedCodec::new())
    }
}

impl Decoder for Codec {
    type Item = ProtocolMsg;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.0.decode(src)? {
            Some(in_data) => {Ok(
                Some(ProtocolMsg::from_bytes(in_data.to_vec()))
            )},
            None => Ok(None),
        }
    }
}

impl Encoder<ProtocolMsg> for super::EnCodec {
    type Error = io::Error;
    
    fn encode(&mut self, item: ProtocolMsg, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let data = to_bytes(&item);
        let buf = Bytes::from(data);
        return self.0.encode(buf, dst);
    }
}

impl Encoder<Arc<ProtocolMsg>> for super::EnCodec {
    type Error = io::Error;
    
    fn encode(&mut self, item: Arc<ProtocolMsg>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let bor:&ProtocolMsg = item.borrow();
        let data = to_bytes(bor);
        let buf = Bytes::from(data);
        return self.0.encode(buf, dst);
    }
}

impl std::clone::Clone for Codec {
    fn clone(&self) -> Self {
        Codec::new()
    }
}