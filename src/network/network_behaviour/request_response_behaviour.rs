// use libp2p::{Swarm, request_response};
// use crate::logger;
// use crate::network::network::Behaviour;
// use crate::state::APP;
use crate::network::network::{Request, Response};



pub async fn handle_event(event: libp2p::request_response::Event<Request, Response>) {
    match event {
        _ => {},
    }
}
