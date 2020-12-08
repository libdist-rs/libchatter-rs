// use futures::prelude::*;
use clap::{load_yaml, App};
use config::Node;
use types::{Block};
// use types::Replica;
use std::{error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let yaml = load_yaml!("cli.yml");
    let m = App::from_yaml(yaml).get_matches();

    let conf_str = m.value_of("config")
        .expect("unable to convert config file into a string");
    let conf_file = std::path::Path::new(conf_str);
    let str = String::from(conf_str);
    let config = match conf_file
        .extension()
        .expect("Unable to get file extension")
        .to_str()
        .expect("Failed to convert the extension into ascii string") 
    {
        "json" => Node::from_json(str),
        "dat" => Node::from_bin(str),
        "toml" => Node::from_toml(str),
        "yaml" => Node::from_yaml(str),
        _ => panic!("Invalid config file extension"),
    };
    config
        .validate()
        .expect("The decoded config is not valid");

    println!("Successfully decoded the config file");

    let (send, mut recv) = net::replica::client::start(&config).await;
    let mut num_new_tx:u64 = 0;
    let txs = &mut Vec::with_capacity(config.block_size);
    loop {
        // let txs = &mut txs;
        let tx_opt = recv.recv().await;
        let tx = match tx_opt {
            Some(t) => t,
            None => break,
        };
        num_new_tx += 1;
        txs.push(tx);
        println!("Got new transactions [{}] from a client", num_new_tx);
        if txs.len() == config.block_size {
            let b = Block::with_tx(txs.drain(..).collect());
            if let Err(e) = send.send(b).await {
                println!("Failed to send the blocks to the clients");
                println!("Error: {}", e);
            };
        }
    }
    Ok(())
}

