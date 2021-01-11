use serde::{Serialize, Deserialize};
use crate::{Replica, Vote, View};
use crypto::hash::Hash;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CertType {
    Blame(Replica, View),
    Vote(Hash),
    QuitView(View),
    DEFAULT,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Certificate {
    pub msg: CertType,
    pub votes: Vec<Vote>,
}

impl Certificate {
    pub fn empty_cert() -> Self {
        Certificate {
            votes: Vec::new(),
            msg:CertType::DEFAULT,
        }
    }
}

impl std::default::Default for Certificate {
    fn default() -> Self {
        Certificate::empty_cert()
    }
}