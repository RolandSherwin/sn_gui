use libp2p::Multiaddr;
use sn_client::Client;
use std::sync::{Arc, Mutex};

type ArcMut<T> = Arc<Mutex<T>>;

#[derive(Default)]
pub(crate) struct Configuration {
    peer_id: ArcMut<Option<Multiaddr>>,
    local: ArcMut<bool>,
}

#[derive(Default)]
pub(crate) struct Network {
    client: Option<Client>,
}
