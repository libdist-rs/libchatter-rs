use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Payload {
    pub response: Vec<u8>,
}

impl Payload {
    pub fn empty() -> Self {
        Self {
            response: Vec::new(),
        }
    }

    pub fn with_payload(size: usize) -> Self {
        let mut payload_data = Payload::empty(); 
        for i in 0..size {
            payload_data.response.push( i as u8);
        }
        payload_data
    }
}

