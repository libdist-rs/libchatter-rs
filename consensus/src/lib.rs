use std::{
    collections::HashMap, 
    time::SystemTime
};
use crypto::hash::Hash;

pub mod apollo;
pub mod synchs;
pub mod dummy;

pub fn statistics(
    now: SystemTime, 
    start:SystemTime, 
    latency_map:HashMap<Hash, (SystemTime, SystemTime)>
)
{
    let mut idx = 0 ;
    let mut total_time = 0;
    for (_hash, (begin, end)) in latency_map {
        let time = end.duration_since(begin)
            .expect("time differencing errors")
            .as_millis();
        log::trace!(target:"statistics", "{}: {}", idx, time);
        idx += 1;
        total_time += time;
    }
    log::info!(target:"statistics", "DP[Throughput]: {}", 
        (idx as f64)/(now.duration_since(start)
            .expect("time differencing errors")
            .as_secs_f64())
    );
    log::info!(target:"statistics", "DP[Latency]: {}", 
                (total_time as f64)/(idx as f64));
}