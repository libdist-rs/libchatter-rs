use tokio::io::AsyncRead;
use tokio_stream::{
    Stream, 
    StreamMap
};
use tokio_util::codec::{
    Decoder, 
    FramedRead
};

/// Given multiple readers, this module returns an iterator over the readers
/// returning the first available reader

pub fn combine_streams<T,D>(readers: Vec<FramedRead<T,D>>) -> 
    impl Stream<Item=(i32, Result<D::Item, D::Error>)>
    where T: AsyncRead + Unpin,
        D: Decoder,
{
    let mut combined = StreamMap::new();
    let mut idx = 0;
    for r in readers {
        combined.insert(idx, r);
        idx += 1;
    }
    combined
}