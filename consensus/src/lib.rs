use std::{
    collections::HashMap, 
    time::SystemTime
};
use crypto::hash::Hash;

// use crossfire::mpsc::{
    // RxFuture, 
    // SharedFutureBoth, 
    // TxFuture
// };
// use tokio::sync::mpsc::Sender;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub mod apollo;
pub mod synchs;
pub mod dummy;

// mod start
// type Sender<T> = Sender<T>;
// type Receiver<T> = Receiver<T>;

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
        // println!("{}: {}", idx, time);
        idx += 1;
        total_time += time;
    }
    // println!("Statistics:");
    println!("DP[Throughput]: {}", 
        (idx as f64)/(now.duration_since(start)
            .expect("time differencing errors")
            .as_secs_f64())
    );
    println!("DP[Latency]: {}", 
                (total_time as f64)/(idx as f64));
}