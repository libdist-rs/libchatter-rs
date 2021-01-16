use tokio::sync::mpsc::{
    UnboundedReceiver, 
    UnboundedSender, 
    error::SendError, 
    unbounded_channel
};

/// This is an implementation of a channel similar to that of diem.
///
/// We have full control over how the messages are received from the channel
pub fn channel<T> () -> (Sender<T>,Receiver<T>){
    let (tx,rx) = unbounded_channel::<T>();
    (Sender::<T>(tx), Receiver::<T>(rx))
}

pub struct Sender<T> (UnboundedSender<T>);

pub struct Receiver<T> (UnboundedReceiver<T>);

impl<T> Sender<T> {
    fn send(&self, message:T) -> Result<(), SendError<T>> {
        self.0.send(message)
    }
}

impl<T> Receiver<T> {
    pub async fn recv(&mut self) -> Option<T> {
        self.0.recv().await
    }
}