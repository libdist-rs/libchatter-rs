use tokio::sync::mpsc::{
    Sender
};
use types::{
    Replica, 
    WireReady
};
use std::{
    collections::HashMap, 
    marker::PhantomData, 
    sync::Arc
};

pub struct Client<I,O> 
where I:WireReady,
O:WireReady,
{
    pub(crate) peers: HashMap<Replica, Sender<Arc<O>>>,
    phantom: PhantomData<(I,O)>,
}

impl<I,O> Client<I,O> 
where I:WireReady,
O:WireReady,
{
    /// Initialize a manager with what kind of network it is going to be:
    /// One of {Server, Client}
    pub fn new() -> Self {
        Self{
            peers: HashMap::new(),
            phantom: PhantomData,
        }
    }
}

pub struct Protocol<I,O> 
where I:WireReady,
O:WireReady,
{
    pub(crate) my_id: Replica,
    pub(crate) num_nodes: Replica,
    phantom: PhantomData<(I,O)>,
}

impl<I,O> Protocol<I,O> 
where I:WireReady,
O:WireReady,
{
    pub fn new(my_id: Replica, num_nodes: Replica) -> Self {
        Self{
            phantom: PhantomData,
            my_id,
            num_nodes,
        }
    }
}