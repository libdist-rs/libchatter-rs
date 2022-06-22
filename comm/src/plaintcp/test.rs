use serde::{Serialize, Deserialize};

use crate::{NetResult, plaintcp::{TcpContext, TcpCommunication}, Message};

use super::TcpConfig;

fn test_local_node_configs(num_nodes:usize) -> Vec<TcpConfig>
{
    (0..num_nodes)
        .map(|_i| {
            TcpConfig::default()
        })
        .collect()
}

#[derive(Serialize, Deserialize, Debug)]
enum TestMsg {
}

impl Message for TestMsg {
    fn from_bytes(data: &[u8]) -> Self {
        bincode::deserialize(data)
            .expect("Deserialization of TestMsg failed")
    }

    fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self)
            .expect("Serialization of TestMsg failed")
    }
}

#[test]
fn test_ctx() -> NetResult<()>
{
    const N:usize = 2;
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let handle = rt.handle();
    let configs = test_local_node_configs(N);
    let _comm: Vec<_> = (0..N)
        .map(|i| {
            TcpCommunication::<TestMsg, TestMsg>::init(
                configs[i].clone(),
                TcpContext::default(),
                handle.clone(),
          )
        }).collect();
    _comm.iter()
        .map(|comm| {
            comm.start()
        });
    Ok(())
}