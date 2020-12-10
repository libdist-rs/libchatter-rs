use clap::{load_yaml, App};
use config::Client;
use std::{error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let yaml = load_yaml!("cli.yml");
    let m = App::from_yaml(yaml).get_matches();

    let conf_str = m.value_of("config")
        .expect("unable to convert config file into a string");
    let conf_file = std::path::Path::new(conf_str);
    let str = String::from(conf_str);
    let mut config = match conf_file
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
    if let Some(f) = m.value_of("ip") {
        config.update_config(util::io::file_to_ips(f.to_string()));
    }
    let config = config;
    println!("Successfully decoded the config file");
    let metrics:u64 = m.value_of("metrics").unwrap_or("500000")
        .parse().unwrap();
    let window:usize = m.value_of("window").unwrap_or("1000")
        .parse().unwrap();
    println!("Successfully decoded the config file");
    let (net_send,net_recv) = net::client::start(config.clone()).await;
    consensus::apollo::client::start(
        &config, net_send, net_recv, metrics, window).await;
    Ok(())
}
