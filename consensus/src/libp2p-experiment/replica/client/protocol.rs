use futures::{AsyncRead, AsyncWrite};
use futures_codec::{FramedRead, FramedWrite, LengthCodec};
use libp2p::{InboundUpgrade, OutboundUpgrade, core::UpgradeInfo, swarm::NegotiatedSubstream};
use types::{Block, Transaction};
use std::{io, iter};
use util::codec::ftx::{Codec as TxCodec};
use util::codec::fblock::{Codec as BlkCodec};
use futures::prelude::*;


pub struct Protocol{}

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


pub async fn recv_tx<S>(stream:S) -> io::Result<(S, Transaction)>
where
    S: AsyncRead + Unpin
{
    // read a transaction from the stream, using a codec
    let mut reader = FramedRead::new(
        stream,
        TxCodec{0:LengthCodec{}},
    );
    match reader.next().await {
        Some(Ok(tx)) => {
            let x = reader.into_inner();
            return Ok((x, tx));
        }
        Some(Err(e)) => Err(e),
        None => Err(io::Error::new(io::ErrorKind::Other, "EOF")),
    }
}

pub async fn send_block<S>(stream:S, b: Block) -> io::Result<S>
where
    S: AsyncWrite + Unpin
{
    // send a block and return the stream
    let mut writer = FramedWrite::new(
        stream,
        BlkCodec{0:LengthCodec{}}
    );
    match writer.send(b).await? {
        () => {
            let x = writer.into_inner();
            return Ok(x);
        }
    }
}
