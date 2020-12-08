use futures_codec::{
    FramedRead, 
    FramedWrite, 
    LengthCodec
};
use libp2p::{
    core::{
        UpgradeInfo,
        InboundUpgrade,
        OutboundUpgrade,
    }, 
    swarm::{ 
        NegotiatedSubstream,
    },
};
use types::{
    Block, 
    Transaction
};
use std::{
    iter,
};
use futures::prelude::*;
use std::io;
use util::codec::ftx::{Codec as TxCodec};
use util::codec::fblock::{Codec as BlkCodec};

pub struct Protocol{

}

impl UpgradeInfo for Protocol {
    type Info = &'static [u8];
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(b"/apollo/client/1.0.0")
    }
}

impl InboundUpgrade<NegotiatedSubstream> for Protocol {
    type Output = NegotiatedSubstream;
    type Error = ();
    type Future = future::Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_inbound(self, stream:NegotiatedSubstream, _: Self::Info) -> Self::Future {
        future::ok(stream)
    }
}

impl OutboundUpgrade<NegotiatedSubstream> for Protocol {
    type Output = NegotiatedSubstream;
    type Error = ();
    type Future = future::Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_outbound(self, stream: NegotiatedSubstream, _: Self::Info) -> Self::Future {
        future::ok(stream)
    }
}

pub async fn send_tx<S>(stream:S, tx:Transaction) -> io::Result<S>
where
    S: AsyncWrite + Unpin 
{
    let mut writer = Box::new(FramedWrite::new(
                stream,
                TxCodec{0:LengthCodec{}}
            ));
    match writer.send(tx).await? {
        () => {
            let (x,_) = writer.release();
            return Ok(x);
        }
    }
}

pub async fn recv_block<S>(stream:S) -> io::Result<(S, Block)>
where
    S: AsyncRead + Unpin
{
    // read a block from the stream, maybe use a codec?
    let mut reader = FramedRead::new(
        stream,
        BlkCodec{0:LengthCodec{}}
    );
    match reader.next().await {
        Some(Ok(b)) => {
            let (x,_) = reader.release();
            return Ok((x, b));
        },
        Some(Err(e)) => Err(e),
        None => Err(io::Error::new(io::ErrorKind::Other, "EOF")),
    }
}