use libp2p::{Swarm, request_response};
use libp2p_request_response::Message;
use crate::logger;
use crate::network::network::Behaviour;
use crate::state::{APP, Request_Item};
use crate::network::network::{Request, Response};



pub async fn handle_event(event: libp2p::request_response::Event<Request, Response>) {
    match event {

        request_response::Event::InboundFailure { error, ..} => {
            log::info!("Inbound Error {error}")
        }

        request_response::Event::OutboundFailure { error, ..} => {
            log::info!("Outbound failiure {error}");
        }

        request_response::Event::Message { peer, message } => {
            match message {
                Message::Request { request, channel, .. } => {
                    log::info!("Received request: {:?}", request);
                    let mut app = APP.lock().unwrap();
                    let new_request = Request_Item {
                        peer_id: peer,
                        request_string: request.request,
                        response_channel: channel,
                    };
                    app.current_requests.push(new_request);
                },

                Message::Response { response, .. } => {
                    log::info!("Received response: {:?}", response);

                    if let Err(e) = std::fs::write("new_".to_owned() + &response.filename, response.data) {
                        log::error!("Error writing: {:?} Error: {:?}", &response.filename, e);
                    } else {
                        log::info!("File {:?} received and saved successfully", &response.filename);
                    }
                },
            }
        }
        _ => {}
    }
}
