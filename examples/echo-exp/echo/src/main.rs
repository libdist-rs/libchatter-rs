use clap::{load_yaml, App};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener}};
use std::{error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let yaml = load_yaml!("cli.yml");
    let m = App::from_yaml(yaml).get_matches();

    let port: u16 = m.value_of("port")
        .expect("please provide port number")
        .parse()
        .expect("please provide a valid port number");

    let sock = TcpListener::bind(format!("0.0.0.0:{}",port))
        .await
        .expect("Failed to bind to port");
    let mut buf = [0 as u8; 1_000_000]; // Nearly 1 MB buffer
    loop {
        let conn = sock.accept().await;
        if let Err(e) = conn {
            println!("Error connecting: {}", e);
            break;
        }
        let (mut stream, from) = conn.unwrap();
        stream.set_nodelay(true).unwrap();
        println!("Connected to a sink: {}", from);
        loop {
            let x = stream.read(&mut buf).await;
            if let Err(e) = x {
                println!("Closing: {}", e);
                break;
            }
            if let Ok(y) = x{
                if let Err(e) = stream.write(&mut buf[0..y]).await  {
                    println!("Closing: {}", e);
                    break;
                }
            }
        }
    }
    Ok(())
}
