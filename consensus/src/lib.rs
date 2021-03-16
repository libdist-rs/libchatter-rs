use std::time::SystemTime;
use fnv::FnvHashMap as HashMap;
use crypto::hash::Hash;

pub fn statistics(
    now: SystemTime, 
    start:SystemTime, 
    latency_map:HashMap<Hash, (SystemTime, SystemTime)>
)
{
    let mut idx = 0 ;
    let mut total_time = 0;
    log::info!("DP[Start]: {:?}", start);
    log::info!("DP[End]: {:?}", now);
    for (_hash, (begin, end)) in latency_map {
        let time = end.duration_since(begin)
            .expect("time differencing errors")
            .as_millis();
        log::trace!("{}: {}", idx, time);
        idx += 1;
        total_time += time;
    }
    log::info!("DP[Throughput]: {}", 
        (idx as f64)/(now.duration_since(start)
            .expect("time differencing errors")
            .as_secs_f64())
    );
    log::info!("DP[Latency]: {}", 
                (total_time as f64)/(idx as f64));
}