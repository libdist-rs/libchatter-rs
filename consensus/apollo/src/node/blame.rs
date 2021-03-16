use types::{Vote};

use super::context::Context;

pub async fn on_receive_blame(v: Vote, _bc: &mut Context) {
    println!("Received a blame message: {:?}", v);
    println!("Currently unimplemented");
}