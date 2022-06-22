use fnv::FnvHashMap;
use serde::{Serialize, Deserialize};
use simple_logger::SimpleLogger;
use crate::{NetResult, plaintcp::{TcpContext, TcpCommunication}, Message};
use super::TcpConfig;

type Id = usize;

const BASE_PORT: u16 = 7_000;

fn get_ids(num_nodes: usize) -> Vec<Id> {
    (0..num_nodes).collect()
}

fn test_local_node_configs(num_nodes:usize) -> Vec<TcpConfig<Id>>
where
    Id: std::cmp::Eq+ std::hash::Hash,
{
    let ids = get_ids(num_nodes);
    let mut configs: Vec<_> = (0..num_nodes)
        .map(|i| {
            let mut conf = TcpConfig::new(ids[i]);
            conf.set_port(BASE_PORT + (i as u16));
            conf
        })
        .collect();
    let mut address_map: FnvHashMap<_, _> = FnvHashMap::default();
    for i in 0..num_nodes {
        let addr = configs[i].get_my_addr();
        address_map.insert(ids[i], addr);
    }
    for i in 0..num_nodes {
        configs[i].add_peers(address_map.clone());
    }
    configs
}

#[derive(Serialize, Deserialize, Debug)]
enum TestMsg {
    Ping(u128),
    Pong(u128),
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
    let logger = SimpleLogger::new();
    SimpleLogger::env(logger).init().unwrap();
    const N:usize = 4;
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let handle = rt.handle();
    let configs = test_local_node_configs(N);
    let mut comm: Vec<_> = (0..N)
        .map(|i| {
            TcpCommunication::<TestMsg, TestMsg, Id>::init(
                configs[i].clone(),
                TcpContext::default(),
                handle.clone(),
          )
        }).collect();
    let _x : Vec<_> = comm.iter_mut()
        .map(|comm| {
            comm.start()
        }).collect();
    for job in _x {
        assert!(job.is_err());
    }
    rt.block_on(async {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    });
    Ok(())
}