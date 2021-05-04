use types::apollo::Vote;

use super::context::Context;

pub async fn on_receive_blame(v: Vote, _bc: &mut Context) {
    panic!("Received a blame message: {:?}", v);
}