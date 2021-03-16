use futures::future::BoxFuture;
use futures::io::{ReadHalf, WriteHalf};
use libp2p::{swarm::{KeepAlive, NegotiatedSubstream, ProtocolsHandler, ProtocolsHandlerEvent, ProtocolsHandlerUpgrErr, SubstreamProtocol, protocols_handler::OutboundUpgradeSend}};
use types::{
    Block, 
};
use core::task::Poll;
use futures::prelude::*;
use std::{collections::VecDeque, task::Context};

use super::{
    Error, 
    IEvent, 
    OEvent, 
    protocol::*, 
    protocol, 
};
use std::io;

// Protocol handler that talks to the server and creates consensus events
pub struct Handler {
    // Some configuration details
    // stay connected
    keep_alive: KeepAlive,
    // stream to use to process IEvents (new transactions)
    events: VecDeque<IEvent>,
    // current state
    state: State,
    // inbound OEvents (new blocks) from the connected servers
    inbound: Option<InResult>,
    // outbound: Sending transactions to the server
    outbound: Option<SendResult>,
    outstream: Option<WriteHalf<NegotiatedSubstream>>,
}

enum State {
    Initializing,
    Waiting,
    DuplexReady,
}

impl Handler {
    pub fn new() -> Self {
        Handler {
            keep_alive:KeepAlive::Yes,
            events: VecDeque::new(),
            state: State::Initializing,
            inbound: None,
            outbound:None,
            outstream: None,
        }
    }
}

impl ProtocolsHandler for Handler {
    type InEvent = IEvent;
    type OutEvent = OEvent;
    type Error = Error;
    type InboundProtocol = Protocol;
    type OutboundProtocol = Protocol;
    type OutboundOpenInfo = ();
    type InboundOpenInfo = ();

    fn listen_protocol(&self) -> SubstreamProtocol<Protocol, ()> {
        SubstreamProtocol::new(Protocol{}, ())
    }

    // Inject events coming from the outside
    fn inject_event(&mut self, event: Self::InEvent) {
        // Send the transaction to the peer
        self.events.push_back(event);
    }

    fn connection_keep_alive(&self) -> KeepAlive {
        self.keep_alive
    }

    /// What to do on establishing a connection with a node?
    /// Note that the server will never proactively contact us
    fn inject_fully_negotiated_inbound(
        &mut self,
        stream: NegotiatedSubstream,
        _info: Self::InboundOpenInfo
    ) {
        println!("Fully negotiated inbound");
        let (rd, wr) = stream.split();
        self.inbound = Some(protocol::recv_block(rd).boxed());
        self.outstream = Some(wr);
        self.state = State::DuplexReady;
    }

    // What to do on successfully negotiating an outbound connection?
    fn inject_fully_negotiated_outbound(
        &mut self,
        stream: NegotiatedSubstream,
        _info: Self::OutboundOpenInfo
    ) {
        println!("Fully negotiated outbound");
        let (rd, wr) = stream.split();
        self.outstream = Some(wr);
        self.inbound = Some(protocol::recv_block(rd).boxed());
        self.state = State::DuplexReady;
    }

    // What to do when a dial upgrade fails?
    fn inject_dial_upgrade_error(
        &mut self,
        _info: Self::OutboundOpenInfo,
        _error: ProtocolsHandlerUpgrErr<<Self::OutboundProtocol as OutboundUpgradeSend>::Error>
    ) {
        println!("Dial upgrade failed");
        self.keep_alive = KeepAlive::No;
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>
    ) -> Poll<ProtocolsHandlerEvent<Self::OutboundProtocol, Self::OutboundOpenInfo, Self::OutEvent, Self::Error>> 
    {
        // Make the connection duplex
        if let State::Initializing = self.state {
            println!("Establishing duplex link, have one way link now");
            self.state = State::Waiting;
            return Poll::Ready(ProtocolsHandlerEvent::OutboundSubstreamRequest{
                protocol: self.listen_protocol(),
            });
            // return Poll::Ready(Pro)
            // return Poll::Pending;
        }
        if let State::Waiting = self.state {
            println!("Sent substream request. waiting for accept from the server");
            return Poll::Pending;
        }
        // If blocks are ready, send them to the outside world first
        if let Some(fut) = self.inbound.as_mut() {
            match fut.poll_unpin(cx) {
                Poll::Pending => {},
                Poll::Ready(Err(e)) => {
                    self.inbound = None;
                    self.keep_alive = KeepAlive::No;
                    println!("Closing connection because {}", e);
                    return Poll::Ready(ProtocolsHandlerEvent::Close(Error::BlockReadFailed(e)));
                }
                Poll::Ready(Ok((stream,b))) => {
                    // Received a block, wait for the next
                    self.inbound = Some(protocol::recv_block(stream).boxed());
                    return Poll::Ready(
                        ProtocolsHandlerEvent::Custom(
                            OEvent::NewBlock(b)
                        )
                    );
                }
            }
        }
        // No new blocks yet from the server yet. 
        // Send transactions if we any new transactions.
        if let Some(fut) = self.outbound.as_mut() {
            match fut.poll_unpin(cx) {
                Poll::Pending => {
                    return Poll::Pending;
                }
                Poll::Ready(Err(e)) => {
                    println!("Closing connection because {}", e);
                    return Poll::Ready(
                        ProtocolsHandlerEvent::Close(Error::TxWriteFailed(e))
                    );
                }
                Poll::Ready(Ok(stream)) => {
                    if let Some(IEvent::NewTx(tx)) = self.events.pop_front() {
                        println!("Trying to send transactions {:?}", tx);
                        self.outbound = Some(
                            protocol::send_tx(stream, tx).boxed()
                        );
                    } else {
                        // We have finished sending a block, and we dont have
                        // new blocks to send
                        self.outbound = None;
                        self.outstream = Some(stream);
                    }
                    return Poll::Pending;
                }
            }
        }
        if let (None, Some(stream)) = 
            (self.outbound.as_mut(), self.outstream.take()) 
        {
            if let Some(IEvent::NewTx(tx)) = self.events.pop_front() {
                println!("Trying to send transactions {:?}", tx);
                self.outbound = Some(
                    protocol::send_tx(stream, tx).boxed()
                );
            } else {
                self.outstream = Some(stream);
            }
        }
        Poll::Pending
    }
}

type SendResult = BoxFuture<'static, Result<WriteHalf<NegotiatedSubstream>, io::Error>>;
type InResult = BoxFuture<'static, Result<(ReadHalf<NegotiatedSubstream>, Block), io::Error>>;

