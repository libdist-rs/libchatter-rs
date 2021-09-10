use serde::Serialize;
use serde_json::to_vec_pretty;
use toml::to_string_pretty;
use serde_yaml::to_writer;
use std::fs::File;
use std::io::{self,prelude::*};

pub enum FileType {
    JSON,
    BIN,
    TOML,
    YAML,
}

pub fn file_to_ips(filename:String) -> Vec<String> {
    let f = File::open(filename).expect("Failed to open the file");
    let mut ips = Vec::new();
    for line in io::BufReader::new(f).lines() {
        if let Ok(s) = line {
            ips.push(s.trim().to_string());
        }
    }
    ips
}

pub fn to_bytes(obj: &impl Serialize) -> Vec<u8> {
    let bytes = bincode::serialize(obj).expect("Failed to serialize object");
    bytes
}

pub fn write_json(filename:String, obj :&impl Serialize) {
    let mut f = File::create(filename)
        .unwrap();
    let bytes = to_vec_pretty(obj)
        .unwrap();
    let bytes_slice = bytes
        .as_slice();
    f.write_all(bytes_slice)
        .unwrap();
}

pub fn write_bin(filename:String, obj :&impl Serialize) {
    let mut f = File::create(filename)
        .unwrap();
    let bytes = bincode::serialize(obj).expect("Failed to serialize object");
    f.write_all(&bytes)
        .unwrap();
}

pub fn write_toml(filename:String, obj:&impl Serialize) {
    let mut f = File::create(filename)
        .unwrap();
    let bytes = to_string_pretty(obj)
        .unwrap();
    f.write_all(bytes.as_bytes())
        .unwrap();
}

pub fn write_yaml(filename:String, obj:&impl Serialize) {
    let f = File::create(filename)
        .unwrap();
    to_writer(f, obj)
        .unwrap();
}