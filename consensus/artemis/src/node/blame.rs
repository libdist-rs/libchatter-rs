use types::artemis::Vote;
use super::*;

pub async fn on_receive_blame(v: Vote, _bc: &mut Context) {
    log::warn!("Received a blame message: {:?}", v);
    panic!("Currently unimplemented");
}