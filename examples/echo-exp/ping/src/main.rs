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
};
use std::{
    error::Error, 
    time::{
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
    let count:usize = m.value_of("count")
    .expect("count not specified")
    .parse()
    .expect("invalid count");
    let mut ser = TcpStream::connect(server)
    .await
    .expect("Failed to connect to the server");
    ser.set_nodelay(true).unwrap();
    let mut msg = vec![1;message];
    let mut times = Vec::new();
    for _i in 0..count {
        let start = SystemTime::now();
        let _x = ser.write_all(&msg).await.unwrap();
        ser.read_exact(&mut msg).await.unwrap();
        let end = SystemTime::now();
        let iter_time = end.duration_since(start)
        .expect("time difference error").as_micros();
        times.push(iter_time);
    }
    statistics(times);
    Ok(())
}

fn statistics(ping_times: Vec<u128>) {
    println!("Ping time Data Points");
    for i in ping_times {
        println!("DP[Time]: {}", i);
    }
}