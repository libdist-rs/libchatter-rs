use std::{
    collections::VecDeque, 
    time::SystemTime
};
use fnv::FnvHashMap as HashMap;
use fnv::FnvHashSet as HashSet;
use config::Client;
use types::{Block, ClientMsg, Transaction};
use futures::channel::mpsc::channel;
use util::new_dummy_tx;
use crypto::hash::Hash;
use consensus::statistics;
use std::sync::Arc;
use util::codec::EnCodec;
use types::ClientMsgCodec as Codec;
use net::futures_manager::TlsClient as NClient;
use futures::{SinkExt, StreamExt};

struct Context {
    pending: usize,
    time_map: HashMap<Hash, SystemTime>,
    count_map:HashMap<Hash, usize>,
    finished_map:HashSet<Hash>,
    latency_map: HashMap<Hash, (SystemTime, SystemTime)>,
    num_cmds: u128,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("Pending", &self.pending)
            .field("count_map", &self.count_map.len())
            .field("time_map", &self.time_map.len())
            .field("latency_map", &self.latency_map.len())
            .field("num_cmds", &self.num_cmds)
            .finish()
    }
}

impl Context {
    pub fn new() -> Self {
        Self {
            pending: 0,
            time_map: HashMap::default(),
            count_map: HashMap::default(),
            finished_map: HashSet::default(),
            latency_map: HashMap::default(),
            num_cmds: 0,
        }
    }
}

pub async fn start(
    c:&Client, 
    metric: u64,
    window: usize,
) {
    let mut client_network = NClient::<ClientMsg, Transaction>::new(c.root_cert.clone());
    let servers = c.net_map.clone();
    let send_id = c.num_nodes as u16;
    let (mut net_send, mut net_recv) = 
        client_network.setup(servers, EnCodec::new(), Codec::new()).await;

    // Start with the sink implementation
    let (mut send, mut recv) = channel(util::CHANNEL_SIZE);
    let m = metric;
    let payload = c.payload;
    tokio::spawn(async move{
        let mut i = 0;
        loop {
            let tx = new_dummy_tx(i,payload);
            i += 1;
            if let Err(e) = send.send(Arc::new(tx)).await {
                log::info!("Closing tx producer channel: {}", e);
                std::process::exit(0);
            }
        }
    });
    let mut cx = Context::new();
    cx.pending = window;
    // let mut time_map = HashMap::new();
    // let mut count_map:HashMap<Hash, usize> = HashMap::new();
    // let mut finished_map:HashSet<Hash> = HashSet::new();
    // let mut latency_map = HashMap::new();
    // let mut num_cmds:u128 = 0;

    let start = SystemTime::now();
    let mut new_blocks = VecDeque::new();
    loop {
        tokio::select! {
            tx_opt = recv.next(), if cx.pending > 0 => {
                if let Some(x) = tx_opt {
                    let hash = crypto::hash::ser_and_hash(x.as_ref());
                    net_send.send((send_id, x)).await
                        .expect("Failed to send to the client");
                    cx.time_map.insert(hash, SystemTime::now());
                    cx.pending -= 1;
                    log::trace!(
                        "Sending transaction to the leader");
                } else {
                    log::info!("Finished sending messages");
                    std::process::exit(0);
                }
            },
            block_opt = net_recv.next() => {
                let now = SystemTime::now();
                log::debug!(target:"consensus",
                    "Got {:?} from the network", block_opt);
                // Got something from the network
                let b = if let Some((_, ClientMsg::NewBlock(p,_))) = block_opt {
                        p.block.clone().unwrap()
                } else {
                    panic!("Got invalid block from the nodes: {:?}", block_opt);
                };
                log::trace!("got a block:{:?}",b);
                new_blocks.push_back(b);
                while let Ok(Some((_, ClientMsg::NewBlock(p,_)))) = net_recv.try_next() {
                    new_blocks.push_back(p.block.clone().unwrap());
                }
                process_blocks(c, now, &mut new_blocks, &mut cx);
                log::debug!("Sending {} commands to the nodes", cx.pending);
            }
        }
        if cx.num_cmds > m as u128 {
            let now = SystemTime::now();
            statistics(now, start, cx.latency_map);
            return;
        }
    }
}

fn process_blocks(c:&Client, now: SystemTime, new_blocks: &mut VecDeque<Arc<Block>>, cx: &mut Context) {
    log::debug!(target:"block-processor", "Processing new {:?}", new_blocks);
    log::debug!(target:"block-processor", "Before processing: {:?}", cx);
    for b in new_blocks.into_iter() {
        // Check if the block is valid?
        if !cx.count_map.contains_key(&b.hash) {
            cx.count_map.insert(b.hash, 1);
            continue;
        }
        let ct = cx.count_map.get(&b.hash).unwrap().clone();
        if ct < c.num_faults {
            cx.count_map.insert(b.hash, ct+1);
            continue;
        }
        if cx.finished_map.contains(&b.hash) {
            continue;
        }
        cx.pending += c.block_size;
        cx.num_cmds += c.block_size as u128;
        for t in &b.body.tx_hashes {
            if let Some(old) = cx.time_map.get(t) {
                cx.latency_map.insert(t.clone(), (old.clone(),now));
            } else {
                log::warn!(
                    "transaction not found in time map");
                cx.num_cmds -= 1;
            }
        }
        cx.finished_map.insert(b.hash);
    }
    log::debug!(target:"block-processor", "After processing: {:?}", cx);
}