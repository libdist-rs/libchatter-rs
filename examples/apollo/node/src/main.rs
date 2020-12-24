// use futures::prelude::*;
use clap::{load_yaml, App};
use config::Node;
// use types::Replica;
use std::{error::Error};

fn main() -> Result<(), Box<dyn Error>> {
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
        "json" => Node::from_json(str),
        "dat" => Node::from_bin(str),
        "toml" => Node::from_toml(str),
        "yaml" => Node::from_yaml(str),
        _ => panic!("Invalid config file extension"),
    };
    config
        .validate()
        .expect("The decoded config is not valid");
    if let Some(f) = m.value_of("ip") {
        config.update_config(util::io::file_to_ips(f.to_string()));
    }
    let config = config;
    let mut is_client_apollo_enabled = false;
    if let Some(_x) = m.value_of("special_client") {
        is_client_apollo_enabled = true;
    } 
    println!("Successfully decoded the config file");

    let cli_net_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let prot_net_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let core_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .unwrap();
    let (send, recv) = cli_net_rt.block_on(
        net::replica::client::start(&config)
    );
    // let (send, recv) = net::replica::client::start(&config).await;
    // let out = net::replica::start(&config).await;
    let out = prot_net_rt.block_on(
        net::replica::start(&config)
    );
    let (send_protocol, recv_protocol) = match out {
        Some((x, y)) => (x,y),
        None => {
            println!("Failed to connected to the servers");
            return Ok(());
        },
    };
    // Start the dummy consensus protocol
    core_rt.block_on(
        consensus::apollo::node::reactor(
                &config,
        send_protocol, 
        recv_protocol, 
        send, 
        recv,
                is_client_apollo_enabled
            )
    );
    Ok(())
}

