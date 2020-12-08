use serde::{Serialize, Deserialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Algorithm {
    RSA,
    ED25519,
    SECP256K1,
}

impl FromStr for Algorithm {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "RSA" => Ok(Algorithm::RSA),
            "ED25519" => Ok(Algorithm::ED25519),
            "SECP256K1" => Ok(Algorithm::SECP256K1),
            _ => Err("no match"),
        }
    }
}

// Generate Keys
// openssl genrsa -out private.pem 2048 
// openssl pkcs8 -in private.pem -inform PEM -topk8 -out private.pk8 -outform
// DER -nocrypt 
// rm private.pem      # optional 
// Codec
// let mut bytes = std::fs::read("private.pk8").unwrap();
// let keypair = Keypair::rsa_from_pkcs8(&mut bytes);
// pk = keypair.public()
// pk.verify for verification
pub const RSA_PK_SIZE:usize = 64;
// Codec
// keypair.sign()
pub const RSA_PVT_SIZE:usize = 64;

// Generate Keys 
// use libp2p::identity::ed25519::Keypair::generate().{public(), secret()} to
// generate keys
// Codec
// use self.encode() to serialize
// use libp2p::identity::ed25519::PublicKey::decode() to deserialize
pub const ED25519_PK_SIZE:usize = 32;
// Codec
// use self.to_bytes() to serialize
// use libp2p::identity::ed25519::SecretKey::from_bytes to deserialize
pub const ED25519_PVT_SIZE:usize = 64;

// Generate Keys
// use libp2p::identity::secp256k1::Keypair::generate().{public(), secret()} to
// generate keys
// Codec
// use self.encode() to serialize
// use libp2p::identity::secp256k1::PublicKey::decode to deserialize
pub const SECP256K1_PK_SIZE:usize = 33;
// Codec
// use self.to_bytes() to serialize
// use libp2p::identity::secp256k1::SecretKey::from_bytes to deserialize
pub const SECP256K1_PVT_SIZE:usize = 32;