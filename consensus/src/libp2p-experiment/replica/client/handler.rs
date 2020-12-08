use std::{task::Context, collections::VecDeque};
use std::io;
use futures::future::BoxFuture;
use futures::io::{ReadHalf, WriteHalf};
use futures::prelude::*;

use libp2p::swarm::{KeepAlive, NegotiatedSubstream, ProtocolsHandler, ProtocolsHandlerEvent, ProtocolsHandlerUpgrErr, SubstreamProtocol, protocols_handler::OutboundUpgradeSend};
use types::Transaction;

use core::task::Poll;
use super::{protocol,protocol::Protocol, Error, IEvent, OEvent};


// The protocol handler that interacts with the client
pub struct Handler {
    // stream to process outside events
    events: VecDeque<IEvent>,
    // should we stay connected to the client
    keep_alive: KeepAlive,
    // state of this connection
    state: State,
    // incoming transactions from the client
    inbound: Option<InResult>,
    // out bound
    outbound: Option<SendResult>,
    outstream: Option<WriteHalf<NegotiatedSubstream>>,
}

enum State {
    Initializing,
    DuplexReady,
}

impl Handler {
    pub fn new() -> Self {
        Handler{
            events: VecDeque::new(),
            keep_alive: KeepAlive::Yes,
            state: State::Initializing,
            inbound: None,
            outbound: None,
            outstream: None,
        }
    }
}

impl ProtocolsHandler for Handler {
    /// When the client creates an OutEvent (Sending a transaction), an 
    /// in event is triggered for the server
    type InEvent = IEvent;
    type OutEvent = OEvent;
    type Error = super::Error;
    type InboundProtocol = Protocol;
    type OutboundProtocol = Protocol;
    type OutboundOpenInfo = ();
    type InboundOpenInfo = ();

    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol, Self::InboundOpenInfo> {
        SubstreamProtocol::new(Protocol{}, ())
    }

    // Inject events (new blocks) coming from the outside
    fn inject_event(&mut self, event: Self::InEvent) {
        // We have a new block event, add to processing
        self.events.push_back(event);
    }

    // Do we stay connected to the client, YES
    fn connection_keep_alive(&self) -> KeepAlive {
        self.keep_alive
    }

    // What do we do when a new client establishes a connection with us?
    fn inject_fully_negotiated_inbound(
        &mut self,
        stream: NegotiatedSubstream, 
        _info: Self::InboundOpenInfo) 
    {
        println!("Fully negotiated inbound");

        let (rh, wh) = stream.split();
        self.outstream = Some(wh);
        self.outbound = None;
        self.inbound = Some(
            protocol::recv_tx(rh).boxed()
        );
        self.state = State::DuplexReady;
    }

    /// What do we do we establish a connection with a new node?
    /// NOTE: We will never proactively contact clients
    fn inject_fully_negotiated_outbound(
        &mut self,
        stream: NegotiatedSubstream, 
        _info: Self::OutboundOpenInfo) 
    {
        println!("Fully negotiated outbound");

        let (rd, wr) = stream.split();
        self.outstream = Some(wr);
        self.outbound = None;
        self.inbound = Some(
            protocol::recv_tx(rd).boxed()
        );
        self.state = State::DuplexReady;
    }

    /// What do we do when the upgrade to a node fails
    fn inject_dial_upgrade_error(&mut self, 
        _info: Self::OutboundOpenInfo, 
        _error: ProtocolsHandlerUpgrErr<<Self::OutboundProtocol as OutboundUpgradeSend>::Error>) 
    {
        println!("Closing connection because {:?}", _error);
        self.keep_alive = KeepAlive::No;        
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>
    ) -> Poll<ProtocolsHandlerEvent<Self::OutboundProtocol, Self::OutboundOpenInfo, Self::OutEvent, Self::Error>> 
    {
        // We have a one way from self <- other, need to establish the other
        // direction to be duplex
        if let State::Initializing = self.state {
            println!("Establishing duplex link, have one way link now");
            return Poll::Pending;
        }
        // If we are not currently sending anything, and blocks are ready, 
        // then send the blocks to the clients
        if let Some(fut) = self.outbound.as_mut() {
            match fut.poll_unpin(cx) {
                Poll::Pending => {}, // continue to the rest of the code
                // An error from reading is ready, return this error outside
                Poll::Ready(Err(e)) => {
                    println!("Closing connection because {}", e);
                    self.outbound = None;
                    self.keep_alive = KeepAlive::No;
                    return Poll::Ready(ProtocolsHandlerEvent::Close(Error::BlockWriteFailed(e)));
                }
                // We have successfully written a block, send another block
                Poll::Ready(Ok(stream)) => {
                    // Do we have a new block?
                    if let Some(IEvent::NewBlock(b)) = self.events.pop_front() {
                        // Send the next block
                        self.outbound = Some(
                            protocol::send_block(stream, b).boxed()
                        );
                    } else {
                        self.outbound = None;
                        // We dont have any block to send yet.
                        // Reset outstream to the next time we have a block
                        self.outstream = Some(stream);
                    }
                }
            }
        }
        // We have not sent anything last time, since we were connected but
        // didnt have blocks.
        // Now, we have a new block and we are connected, send a new block
        if let (None, Some(stream)) = 
            (self.outbound.as_mut(),self.outstream.take()) 
        {
            // Do we have a new block?
            if let Some(IEvent::NewBlock(b)) = self.events.pop_front() {
                self.outbound = Some(
                    protocol::send_block(stream, b).boxed()
                );
            } else {
                self.outstream = Some(stream);
            }
        }
        // Check if we have new transactions from clients
        if let Some(fut) = self.inbound.as_mut() {
            return match fut.poll_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(e)) => {
                    // Error in receiving a transaction, return the error to the
                    // outside world
                    println!("Closing connection because {}", e);
                    Poll::Ready(
                        ProtocolsHandlerEvent::Close(Error::TxReadFailed(e))
                    )
                }
                Poll::Ready(Ok((stream, tx))) => {
                    // listen to the next incoming transaction
                    self.inbound = Some(protocol::recv_tx(stream).boxed());
                    Poll::Ready(
                        ProtocolsHandlerEvent::Custom(OEvent::NewTx(tx))
                    )
                }
            };
        }
        Poll::Pending
    }

}

type SendResult = BoxFuture<'static, Result<WriteHalf<NegotiatedSubstream>, io::Error>>;
type InResult = BoxFuture<'static, 
    Result<(ReadHalf<NegotiatedSubstream>, Transaction), io::Error>>;

