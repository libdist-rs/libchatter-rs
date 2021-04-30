use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};
use std::io;
use bytes::{Bytes, BytesMut};
use types::WireReady;

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
where I:WireReady,
{
    type Error = io::Error;

    fn encode(&mut self, item: I, dst:&mut BytesMut) -> Result<(),Self::Error> {
        let data = I::to_bytes(&item);
        let buf = Bytes::from(data);
        return self.0.encode(buf, dst);
    }
}

// impl<I> Encoder<Arc<I>> for EnCodec<Arc<I>>
// where I:WireReady,
// {
//     type Error = io::Error;

//     fn encode(&mut self, item: Arc<I>, dst:&mut BytesMut) -> Result<(),Self::Error> {
//         let data = I::to_bytes(&item);
//         let buf = Bytes::from(data);
//         return self.0.encode(buf, dst);
//     }
// }

pub struct Decodec<O> (pub LengthDelimitedCodec, std::marker::PhantomData<O>);
impl<O> Decodec<O> {
    pub fn new() -> Self {
        Decodec(LengthDelimitedCodec::new(),std::marker::PhantomData::<O>)
    }
}

impl<O> Decoder for Decodec<O> 
where O:WireReady,
{
    type Item = O;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.0.decode(src)? {
            Some(in_data) => {Ok(
                Some(O::from_bytes(&in_data))
            )},
            None => Ok(None),
        }
    }
}

impl<O> std::clone::Clone for Decodec<O> 
{
    fn clone(&self) -> Self {
        Decodec::new()
    }
}
