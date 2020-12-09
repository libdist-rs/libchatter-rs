// use futures::prelude::*;
use clap::{load_yaml, App};
use config::Client;
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
        "json" => Client::from_json(str),
        "dat" => Client::from_bin(str),
        "toml" => Client::from_toml(str),
        "yaml" => Client::from_yaml(str),
        _ => panic!("Invalid config file extension"),
    };
    config
        .validate()
        .expect("The decoded config is not valid");

    let metrics:u64 = m.value_of("metrics").unwrap_or("500000").parse().unwrap();
    println!("Successfully decoded the config file");
    let (net_send,net_recv) = net::client::start(config.clone()).await;
    consensus::synchs::client::start(
        &config, net_send, net_recv, metrics).await;
    Ok(())
}
