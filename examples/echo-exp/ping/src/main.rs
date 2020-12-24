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
    collections::HashSet, 
    error::Error, 
    time::SystemTime
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let yaml = load_yaml!("cli.yml");
    let m = App::from_yaml(yaml).get_matches();

    let server = m.value_of("server").expect("Server value not specified");
    let message = m.value_of("message")
        .expect("message size not specified")
        .parse()
        .unwrap();
    // let interval = m.value_of("interval").expect("interval not specified");
    let count:usize = m.value_of("count")
        .expect("count not specified")
        .parse()
        .expect("invalid count");
    let mut ser = TcpStream::connect(server).await.expect("Failed to connect to the server");
    let mut msg = vec![1;message];
    let mut times:HashSet<u128> = HashSet::new();
    for _i in 0..count {
        let start = SystemTime::now();
        ser.write_all(&msg).await.unwrap();
        let end = SystemTime::now();
        times.insert(end.duration_since(start)
            .expect("time difference error").as_micros());
        ser.read_exact(&mut msg).await.unwrap();
    }
    for i in times {
        println!("Time: {}", i);
    }
    Ok(())
}
