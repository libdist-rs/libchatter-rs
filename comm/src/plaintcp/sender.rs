use tokio::sync::mpsc::UnboundedSender;

use crate::{Message, NetSender};

pub struct TcpSender<Msg>
where
    Msg: Message,
{
    sender: UnboundedSender<Msg>
}

impl<Msg> std::convert::From<UnboundedSender<Msg>> for TcpSender<Msg> 
where 
    Msg: Message,
{
    fn from(unb_sender: UnboundedSender<Msg>) -> Self {
        Self { sender: unb_sender }
    }
}

// impl<Msg> NetSender for TcpSender<Msg>
// where Msg: Message,
// {
//     fn send(&self, sender: &PeerId, msg: Out) {
        
//     }
// }