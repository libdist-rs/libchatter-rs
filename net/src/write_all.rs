use std::{
    collections::HashMap, 
    task::{
        Context, 
        Poll, 
        Waker
    }
};
use core::pin::Pin;

use futures::Sink;
use tokio::io::AsyncWrite;
use tokio_util::codec::{Encoder, FramedWrite};
use types::Replica;

/// Given multiple readers, this module returns an iterator over the readers
/// returning the first available reader

pub struct WriteUnordered<T,E,I> 
    where T: AsyncWrite + Unpin,
    E: Encoder<I>,
{
    waker: Option<Waker>,
    writers: HashMap<Replica, FramedWrite<T,E>>,
    buffer: HashMap<Replica, Vec<I>>,
}

impl<T,E,I> WriteUnordered<T,E,I>
    where T:AsyncWrite + Unpin,
    E: Encoder<I>,
{
    pub fn new() -> Self {
        WriteUnordered::<T,E,I>{
            waker: None,
            buffer: HashMap::new(),
            writers: HashMap::new(),
        }
    }

    pub fn add_writers(&mut self, writers: Vec<(Replica,FramedWrite<T,E>)>) {
        for w in writers {
            self.writers.insert(w.0,w.1);
        }
    }
}

impl<T,E,I> Unpin for WriteUnordered<T,E,I>
    where T: AsyncWrite + Unpin,
    E: Encoder<I>,
{}

impl<T,E,I> Sink<I> for WriteUnordered<T,E,I>
    where T: AsyncWrite + Unpin,
    E: Encoder<I>,
{
    type Error = std::io::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Pending
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Pending
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Pending
    }

    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), Self::Error> {
        Ok(())
    }
}

// pub struct SharedState{}