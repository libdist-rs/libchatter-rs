use crate::{Communication, Config, ConfigCommon};
use super::TcpConfig;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};
use tracing::{debug, error, warn};
use async_trait::async_trait;
use std::time::Duration;
use fnv::FnvHashMap as HashMap;

#[derive(Debug)]
pub struct TcpCommunication 
{
    config: TcpConfig,
    listener: TcpListener,
}

impl TcpCommunication {
    async fn incoming_connections(
        num_conn: usize, server_sock: tokio::net::TcpListener
    ) -> (HashMap<usize, TcpStream>, TcpListener)
    {
        let mut connections_incoming = HashMap::default();
        while connections_incoming.len() < num_conn {
            let result = server_sock.accept().await;
            if let Err(e) = result {
                error!("Server socker error: {}", e);
                continue;
            }
            let (mut stream, sock_addr) = result.unwrap();
            stream.set_nodelay(TcpConfig::NO_DELAY);
            debug!("New incoming connection from {}", sock_addr);
            
            let result = stream.read_u64_le().await;
            if let Err(e) = result {
                error!("Error reading Id from incoming connection: {}", e);
                continue;
            }
            let other_id = result.unwrap() as usize;
            connections_incoming.insert(other_id, stream);
        }
        (connections_incoming, server_sock)
    }
    async fn setup(config: TcpConfig) -> Self {
        // First open a listening socket
        let server_sock = tokio::net::TcpListener::bind(
                config.get_listen_addr()
            ).await.expect(
                format!("Failed to bind to address {}", config.get_listen_addr()).as_str()
            );
        // Each node connects to nodes whose oId is less than Id, and waits for incoming connections from nodes whose oId is greater than Id
        let incoming_connections = config.get_num_nodes() - config.get_id() - 1;
        let connect_job = tokio::spawn(
            TcpCommunication::incoming_connections(
                incoming_connections, server_sock
            )
        );
        let mut outgoing_connections = HashMap::default();
        for other_id in 0..config.get_id() {
            let sock_addr = config.get_node_addr(&other_id);
            let total_attempts = TcpConfig::NUM_RETRIES;
            let mut attempt = 0;
            while attempt < total_attempts {
                let sock_addr = sock_addr.as_str();
                let result = TcpStream::connect(sock_addr)
                    .await;
                if let Err(e) = result {
                    warn!("Failed to connect to node {} with error {}, Retrying attempt #{}", other_id, e, attempt);
                    attempt += 1;
                    // Sleep before trying again
                    tokio::time::sleep(Duration::from_millis(TcpConfig::CONNECTION_SLEEP_TIME)).await;
                    continue;
                }
                let mut conn = result.unwrap();
                let result = conn.set_nodelay(TcpConfig::NO_DELAY);
                if let Err(e) = result {
                    error!("Failed to set connection no delay parameter with error {} for node {}", e, other_id);
                    warn!("Falling back to slow mechanism");
                }
                // We received a u64_le when receiving connections, we also need to send one here, so the other end can identify who the source is.
                // TODO: Prove that I am the correct Id
                let result = conn.write_u64_le(config.get_id() as u64).await;
                if let Err(e) = result {
                    error!("Failed to set ")
                }
                outgoing_connections.insert(other_id, conn);
                break;
            }
        }
        let (mut incoming, listener) = connect_job
            .await.expect("Failed to receive incoming connections");
        incoming.extend(outgoing_connections);

        // TODO: Setup channels

        Self{
            config,
            listener,
        }
    }
}

#[async_trait]
impl Communication for TcpCommunication {
    type Config = TcpConfig;

    async fn init(config: Self::Config) -> Self {
        TcpCommunication::setup(config).await
    }

    fn send(sender: usize, msg: impl crate::Message) {
        unimplemented!();
    }

    fn concurrent_send(sender:usize, msg: impl crate::Message) {
        unimplemented!();
    }

    fn broadcast(msg: impl crate::Message) {
        unimplemented!();
    }

    fn concurrent_broadcast(msg: impl crate::Message) {
        unimplemented!();
    }
}