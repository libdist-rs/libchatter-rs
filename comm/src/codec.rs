use std::io;
use tokio_util::codec::{LengthDelimitedCodec, Decoder, Encoder};
use bytes::{Bytes, BytesMut};
use crate::Message;

/// Encoding and decoding messages on the network
pub struct Decodec<O> (pub LengthDelimitedCodec, std::marker::PhantomData<O>);
impl<O> Decodec<O> {
    pub fn new() -> Self {
        Decodec(LengthDelimitedCodec::new(),std::marker::PhantomData::<O>)
    }
}

impl<Msg> Decoder for Decodec<Msg> 
where 
    Msg: Message,
{
    type Item = Msg;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.0.decode(src)? {
            Some(in_data) => {Ok(
                Some(Msg::from_bytes(&in_data))
            )},
            None => Ok(None),
        }
    }
}

#[derive(Debug)]
pub struct EnCodec<I> (pub LengthDelimitedCodec, std::marker::PhantomData<I>);

impl<I> EnCodec<I> {
    pub fn new() -> Self {
        EnCodec(LengthDelimitedCodec::new(),std::marker::PhantomData::<I>)
    }
}

impl<I> std::clone::Clone for EnCodec<I> {
    fn clone(&self) -> Self {
        EnCodec::new()
    }
}

impl<I> Encoder<I> for EnCodec<I> 
where I:Message,
{
    type Error = io::Error;

    fn encode(&mut self, item: I, dst:&mut BytesMut) -> Result<(),Self::Error> {
        let data = I::to_bytes(&item);
        let buf = Bytes::from(data);
        return self.0.encode(buf, dst);
    }
}