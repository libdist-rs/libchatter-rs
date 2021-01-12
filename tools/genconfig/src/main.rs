// A tool that builds config files for all the nodes and the clients for the
// protocol.

use crypto::{ed25519, secp256k1};
use config::{Node, Client};
use clap::{load_yaml, App};
use types::{Replica};
use crypto::Algorithm;
use std::{collections::HashMap};
use util::io::*;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let m = App::from_yaml(yaml).get_matches();
    let num_nodes:usize =  m.value_of("num_nodes")
        .expect("number of nodes not specified")
        .parse::<usize>()
        .expect("unable to convert number of nodes into a number");
    let num_faults:usize = match m.value_of("num_faults") {
        Some(x) => x.parse::<usize>()
            .expect("unable to convert number of faults into a number"),
        None => (num_nodes-1)/2,
    };
    let delay:u64 = m.value_of("delay")
        .expect("delay value not specified")
        .parse::<u64>()
        .expect("unable to parse delay value into a number");
    let base_port: u16 = m.value_of("base_port")
        .expect("base_port value not specified")
        .parse::<u16>()
        .expect("failed to parse base_port into a number");
    let blocksize: usize = m.value_of("block_size")
        .expect("no block_size specified")
        .parse::<usize>()
        .expect("unable to convert blocksize into a number");
    let client_base_port:u16 = m.value_of("client_base_port")
        .expect("no client_base_port specified")
        .parse::<u16>()
        .expect("unable to parse client_base_port into an integer");
    let t:Algorithm = m.value_of("algorithm")
        .unwrap_or("ED25519")
        .parse::<Algorithm>()
        .unwrap_or(Algorithm::ED25519);
    let out = m.value_of("out_type")
        .unwrap_or("json");
    let target = m.value_of("target")
        .expect("target directory for the config not specified");
    let payload:usize = m.value_of("payload")
        .unwrap_or("0")
        .parse()
        .unwrap();
    let mut client = Client::new();
    client.block_size = blocksize;
    client.crypto_alg = t.clone();
    client.num_nodes = num_nodes;
    client.num_faults = num_faults;

    let mut node:Vec<Node> = Vec::with_capacity(num_nodes);

    let mut pk = HashMap::new();
    let mut ip = HashMap::new();

    for i in 0..num_nodes {
        node.push(Node::new());

        node[i].delta = delay;
        node[i].id = i as Replica;
        node[i].num_nodes = num_nodes;
        node[i].num_faults = num_faults;
        node[i].block_size = blocksize;
        node[i].payload = payload;
        node[i].client_port = client_base_port+(i as u16);

        node[i].crypto_alg = t.clone();
        match t {
            Algorithm::ED25519 => {
                let kp = ed25519::Keypair::generate();
                pk.insert(i as Replica, kp.public().encode().to_vec());
                node[i].secret_key_bytes = kp.encode().to_vec();

            }
            Algorithm::SECP256K1 => {
                let kp = secp256k1::Keypair::generate();
                pk.insert(i as Replica, kp.public().encode().to_vec());
                node[i].secret_key_bytes = kp.secret().to_bytes().to_vec();
            }
            _ => (),
        };
        ip.insert(i as Replica, 
        format!("{}:{}", "127.0.0.1", base_port+(i as u16))
        );
        client.net_map.insert(i as Replica, 
        format!("127.0.0.1:{}", client_base_port+(i as u16))
        );
    }

    for i in 0..num_nodes {
        node[i].pk_map = pk.clone();
        node[i].net_map = ip.clone();
    }

    client.server_pk = pk;

    // Write all the files
    for i in 0..num_nodes {
        match out {
            "json" => {
                let filename = format!("{}/nodes-{}.json",target,i);
                write_json(filename, &node[i]);
            },
            "binary" => {
                let filename = format!("{}/nodes-{}.dat",target,i);
                write_bin(filename, &node[i]);
            },
            "toml" => {
                let filename = format!("{}/nodes-{}.toml",target,i);
                write_toml(filename, &node[i]);
            },
            "yaml" => {
                let filename = format!("{}/nodes-{}.yml",target,i);
                write_yaml(filename, &node[i]);
            },
            _ => (),
        }
        node[i].validate()
            .expect("failed to validate node config");
    }

    // Write the client file
    match out {
        "json" => {
            let filename = format!("{}/client.json",target);
            write_json(filename, &client);
        },
        "binary" => {
            let filename = format!("{}/client.dat",target);
            write_bin(filename, &client);
        },
        "toml" => {
            let filename = format!("{}/client.toml",target);
            write_toml(filename, &client);
        },
        "yaml" => {
            let filename = format!("{}/client.yml",target);
            write_yaml(filename, &client);
        },
        _ => (),
    }
    client.validate()
        .expect("failed to validate the client config");
}