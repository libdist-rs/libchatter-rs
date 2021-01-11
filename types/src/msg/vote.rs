use serde::{Serialize, Deserialize};
use crate::{protocol::*, View};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blame (Replica, View);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vote {
    pub origin: Replica,
    pub auth: Vec<u8>,
}
