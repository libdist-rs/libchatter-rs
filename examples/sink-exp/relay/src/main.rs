use clap::{load_yaml, App};
use tokio::{io::{AsyncReadExt}, net::{TcpListener}};
use tokio_stream::{StreamExt};
use tokio_util::codec::{FramedRead, FramedWrite};
use types::{Block};
use util::codec::{EnCodec, tx::{Codec as TxCodec}};
use std::{error::Error, time::SystemTime};
use futures::{SinkExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let yaml = load_yaml!("cli.yml");
    let m = App::from_yaml(yaml).get_matches();

    let port: u16 = m.value_of("port")
        .expect("please provide port number")
        .parse()
        .expect("please provide a valid port number");

    // let payload: usize = m.value_of("payload")
        // .expect("please provide a payload")
        // .parse()
        // .expect("please provide a valid payload");

    let count: usize = m.value_of("count")
        .expect("please provide a count")
        .parse()
        .expect("please provide a valid count");

    let blocksize:usize = m.value_of("block_size")
        .expect("please provide a blocksize")
        .parse()
        .expect("please provide a valid blocksize");

    let listener = TcpListener::bind(
        format!("0.0.0.0:{}",port))
        .await
        .expect("Failed to bind to the port");

    let mut indicator = vec![0 as u8;1];
    let (one,_) = listener.accept().await.expect("Failed to accept a connection");
    one.set_nodelay(true).unwrap();
    let (two,_) = listener.accept().await.expect("Failed to accept a connection");
    two.set_nodelay(true).unwrap();

    let (mut r1,w1) = one.into_split();
    let (mut r2,w2) = two.into_split();

    r1.read_exact(&mut indicator)
        .await
        .expect("failed to read the indicator byte");
    println!("Got indicator byte {:?} from the first reader", indicator);
    let mut is_reversed = false;
    // the one who sends 0 is the streamer
    if indicator[0] != 0 {
        is_reversed = true;
    }
    r2.read_exact(&mut indicator)
        .await.expect("failed to read indicator byte");
    println!("Got indicator byte {:?} from the second reader", indicator);
    // Read and discard

    let (r1, _w1, _r2, w2) = if is_reversed {
        (r2,w2,r1,w1)
    } else {
        (r1,w1,r2,w2)
    };

    let mut read1 = FramedRead::new(r1, TxCodec::new());
    let mut write2 = FramedWrite::new(w2, EnCodec::new());

    // write2.send();
    let mut txs = Vec::new();
    let mut rtimes = Vec::new();
    let mut wtimes = Vec::new();
    for _i in 0..count*blocksize {
        let time = SystemTime::now();
        rtimes.push(time);
        let tx = read1.next()
            .await.expect("Failed to read tx from the source")
            .expect("Failed to read tx from the source again");
        let time = SystemTime::now();
        rtimes.push(time);
        txs.push(tx);
        if txs.len() < blocksize {
            continue;
        }
        println!("got {:?} blocks", txs);
        wtimes.push(SystemTime::now());
        let b = txs
            .drain(0..blocksize)
            .map(|c| std::sync::Arc::new(c))
            .collect();
        let blk = Block::with_tx(b);
        write2.send(blk).await.expect("failed to send the block to the sink");
        wtimes.push(SystemTime::now());
    }

    let mut sum:u128 = 0;
    let mut highest:u128 = 0;
    let mut lowest:u128 = std::u128::MAX;
    for i in 0..rtimes.len() {
        if i == 0 {
            continue;
        }
        let diff = rtimes[i].duration_since(rtimes[i-1]).expect("time differencing error");
        let nv = diff.as_nanos();
        sum += nv;
        if nv < lowest {
            lowest = nv;
        }
        if nv > highest {
            highest = nv;
        }
        // println!("RTime:{:?}", nv);
    }
    println!("Stats");
    println!("Highest: {}", highest);
    println!("Lowest: {}", lowest);
    println!("Average: {}", sum/((rtimes.len()-1) as u128));
    let mut sum:u128 = 0;
    let mut highest:u128 = 0;
    let mut lowest:u128 = std::u128::MAX;
    for i in 0..wtimes.len() {
        if i == 0 {
            continue;
        }
        let diff = wtimes[i].duration_since(wtimes[i-1]).expect("time differencing error");
        let nv = diff.as_nanos();
        sum += nv;
        if nv < lowest {
            lowest = nv;
        }
        if nv > highest {
            highest = nv;
        }
        // println!("WTime:{:?}", nv);
    }
    println!("Stats");
    println!("Highest: {}", highest);
    println!("Lowest: {}", lowest);
    println!("Average: {}", sum/((wtimes.len()-1) as u128));
    Ok(())
}
