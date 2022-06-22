use std::net::SocketAddr;

use tokio::runtime::Handle;

use crate::NetResult;

use super::TcpContext;

pub struct Connection {
    handle: Handle,
    peer: SocketAddr,
    ctx: TcpContext,
}

impl Connection {
    pub fn new(ctx: TcpContext, handle: Handle, peer_socket: SocketAddr) -> Self {
        Self { handle, peer: peer_socket, ctx }
    }

    pub async fn start(&mut self) -> NetResult<()> {
        log::info!("Attempting connection to {}", self.peer);
        let mut retries:usize = 0;
        while retries < self.ctx.get_retries()  {
            let sock = if self.peer.is_ipv4() {
                tokio::net::TcpSocket::new_v4()?
            } else {
                tokio::net::TcpSocket::new_v6()?
            };
            let stream_res = sock.connect(self.peer).await;
            if stream_res.is_err() {
                log::warn!("Connection with {} failed. Retrying...", self.peer);
                retries += 1;
                tokio::time::sleep(self.ctx.get_retry_duration()).await;
                continue;
            }
            log::info!("Connected to {}", self.peer);
            let stream = stream_res.unwrap();
            stream.set_nodelay(true)?;
            retries = 0;
            // Create a framed reader
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            todo!("Connection logic");
        }
        Ok(())
    }
}