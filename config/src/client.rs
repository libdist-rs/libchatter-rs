use serde::{
    Serialize, 
    Deserialize
};
use types::Replica;
use crypto::Algorithm;
use std::collections::HashMap;
use super::{
    ParseError,
    is_valid_replica
};
use std::fs::File;
use std::io::prelude::*;
use serde_json::from_reader;
use toml::from_str;

#[derive(Debug, Serialize, Deserialize, Clone,PartialEq)]
pub struct Client {
    pub net_map:HashMap<Replica, String>,
    pub crypto_alg:Algorithm,
    pub server_pk:HashMap<Replica, Vec<u8>>,

    pub num_nodes: usize,
    pub num_faults: usize,
    pub block_size:usize,
    pub payload:usize,
}

impl Client {
    pub fn validate(&self) -> Result<(), ParseError> {
        if self.net_map.len() != self.num_nodes {
            return Err(ParseError::InvalidMapLen(self.num_nodes, self.net_map.len()));
        }
        if 2*self.num_faults >= self.num_nodes {
            return Err(ParseError::IncorrectFaults(self.num_faults, self.num_nodes));
        }
        for repl in &self.net_map {
            if !is_valid_replica(*repl.0, self.num_nodes) {
                return Err(ParseError::InvalidMapEntry(*repl.0));
            }
        }
        match self.crypto_alg {
            Algorithm::ED25519 => {
                for repl in &self.server_pk {
                    if !is_valid_replica(*repl.0, self.num_nodes) {
                        return Err(ParseError::InvalidMapEntry(*repl.0));
                    }
                    if repl.1.len() != crypto::ED25519_PK_SIZE {
                        return Err(ParseError::InvalidPkSize(repl.1.len()));
                    }
                }
            }
            Algorithm::SECP256K1 => {
                for repl in &self.server_pk {
                    if !is_valid_replica(*repl.0, self.num_nodes) {
                        return Err(ParseError::InvalidMapEntry(*repl.0));
                    }
                    if repl.1.len() != crypto::SECP256K1_PK_SIZE {
                        return Err(ParseError::InvalidPkSize(repl.1.len()));
                    }
                }
            }
            Algorithm::RSA => {
                // Because unimplemented
                return Err(ParseError::Unimplemented("RSA"));
            }
        }
        Ok(())
    }

    pub fn new() -> Client {
        Client {
            net_map: HashMap::new(),
            block_size: 0,
            crypto_alg: Algorithm::ED25519,
            num_faults: 0,
            num_nodes:0,
            server_pk: HashMap::new(),
            payload:0,
        }
    } 

    pub fn from_json(filename:String) -> Client {
        let f = File::open(filename)
            .unwrap();
        let c: Client = from_reader(f)
            .unwrap();
        return c;
    }

    pub fn from_toml(filename:String) -> Client {
        let mut buf = String::new();
        let mut f = File::open(filename)
            .unwrap();
        f.read_to_string(&mut buf)
            .unwrap();
        let c:Client = from_str(&buf)
            .unwrap();
        return c;
    }

    pub fn from_yaml(filename:String) -> Client {
        let f = File::open(filename)
            .unwrap();
        let c:Client = serde_yaml::from_reader(f)
            .unwrap();
        return c;
    }

    pub fn from_bin(filename:String) -> Client {
        let mut buf = Vec::new();
        let mut f = File::open(filename)
            .unwrap();
        f.read_to_end(&mut buf)
            .unwrap();
        let bytes:&[u8] = &buf;
        let c:Client = flexbuffers::from_slice(bytes)
            .unwrap();
        return c;
    }

    pub fn update_config(&mut self, ips: Vec<String>) {
        let mut idx = 0;
        for ip in ips {
            // Put others ips in the config
            self.net_map.insert(idx, ip);
            idx += 1;
        }
        log::info!(target:"config", "Talking to servers: {:?}", self.net_map);
    }
}