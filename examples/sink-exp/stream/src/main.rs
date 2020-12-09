use clap::{load_yaml, App};
use tokio::{io::AsyncWriteExt, net::{TcpStream}};
use futures::SinkExt;
use tokio_util::codec::FramedWrite;
use util::codec::EnCodec;
use std::{error::Error, time::SystemTime};
use std::fs::File;
use std::{io, io::BufRead};
use util::new_dummy_tx;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let yaml = load_yaml!("cli.yml");
    let m = App::from_yaml(yaml).get_matches();

    let total:u64 = m.value_of("stop")
        .expect("Number of commands to stream is not specified")
        .parse()
        .expect("invalid input. Please provide a number");

    let mut relays:Vec<String> = Vec::new();
    if let Some(ip) = m.value_of("relay") {
        relays.push(ip.to_string());
    } else if let Some(file) = m.value_of("servers") {
        // Read file line by line and treat them as ips
        let f = File::open(file).expect("failed to open the file");
        for line in io::BufReader::new(f).lines() {
            if let Ok(s) = line {
                relays.push(s.trim().to_string());
            }
        }
    } else {
        panic!("Please provide one of --servers or --relay");
    }
    tokio::time::sleep(Duration::from_millis(5000)).await;

    let indicator = vec![0 as u8;1];
    let mut senders = Vec::new();
    for addr in relays {
        let (_rd, mut wr) = TcpStream::connect(addr).await
            .expect("failed to connected one of the relays")
            .into_split();
        wr.write(&indicator).await.expect("failed to write the indicator byte");
        let writer = FramedWrite::new(wr, EnCodec::new());
        senders.push(writer);
    }
    let mut times = Vec::new();
    let latest = std::time::SystemTime::now();
    times.push(latest);
    for i in 0..total {
        let tx = new_dummy_tx(i);
        for w in 0..senders.len() {
            senders[w].send(tx.clone()).await
                .expect("Failed to send to one of the relayers");
        }
        // measure the time
        let time_now = SystemTime::now();
        times.push(time_now);
    }
    for i in 0..times.len() {
        if i == 0 {
            continue;
        }
        println!("Time:{:?}", times[i].duration_since(times[i-1]));
    }
    Ok(())
}
