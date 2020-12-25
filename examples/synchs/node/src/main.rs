use clap::{
    load_yaml, 
    App
};
use config::Node;
use std::error::Error;

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
    println!("Successfully decoded the config file");

    let (send, recv) = net::replica::client::start(&config).await;
    let out = net::replica::synchs::start(&config).await;
    let (send_protocol, recv_protocol) = match out {
        Some((x, y)) => (x,y),
        None => {
            println!("Failed to connected to the servers");
            return Ok(());
        },
    };
    // Start the dummy consensus protocol
    consensus::synchs::node::reactor(&config,
        send_protocol, 
        recv_protocol, 
        send, 
        recv).await;
    Ok(())
}

