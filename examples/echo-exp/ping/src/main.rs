use clap::{
    load_yaml, 
    App
};
use tokio::{
    io::{
        AsyncReadExt, 
        AsyncWriteExt
    }, 
    net::TcpStream, 
    time::interval
};
use std::{
    error::Error, 
    time::{
        Duration, 
        SystemTime
    }
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let yaml = load_yaml!("cli.yml");
    let m = App::from_yaml(yaml).get_matches();
    
    let server = m.value_of("server")
        .expect("Server value not specified");
    let message = m.value_of("message")
        .expect("message size not specified")
        .parse()
        .unwrap();
    let interval_dur:u64 = m.value_of("interval")
        .unwrap_or("1000")
        .parse()
        .unwrap();
    let count:usize = m.value_of("count")
        .expect("count not specified")
        .parse()
        .expect("invalid count");
    let total:usize = m.value_of("total")
        .expect("please specify the total number of iterations")
        .parse()
        .expect("Invalid total parameter found");
    let mut ser = TcpStream::connect(server)
        .await
        .expect("Failed to connect to the server");
    ser.set_nodelay(true).unwrap();
    let mut msg = vec![1;message];
    let mut times = Vec::new();
    let mut num_in_interval = Vec::with_capacity(total);
    let mut ticker = interval(
        Duration::from_millis(interval_dur)
    );
    ticker.tick().await;
    for _i in 0..total {
        let mut finished = false;
        let mut to_break = false;
        let mut sent:u128 = 0;
        for _i in 0..count {
            let start = SystemTime::now();
            tokio::select! {
                _x = ticker.tick(), if !finished => {
                    // println!("Timer ticked");
                    finished = true;
                    continue;
                },
                _x = ser.write_all(&msg) => {
                    sent += 1;
                    // println!("Sent a message");
                    let end = SystemTime::now();
                    let iter_time = end.duration_since(start)
                        .expect("time difference error").as_micros();
                    times.push(iter_time);
                    ser.read_exact(&mut msg).await.unwrap();
                    if finished {
                        to_break = true;
                    }
                },
            }
            if to_break {
                num_in_interval.push(sent);
                break;
            }
        }
    }
    statistics(times, num_in_interval);
    Ok(())
}

fn statistics(ping_times: Vec<u128>, interval_send_count: Vec<u128>) {
    println!("Ping time Data Points");
    for i in ping_times {
        println!("DP[Time]: {}", i);
    }
    println!("Number of messages sent in an interval");
    for num in interval_send_count {
        println!("DP[Int]: {}", num);
    }
}