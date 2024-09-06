use libp2p::request_response;
use libp2p_request_response::Message;
use crate::logger;
use crate::state::{APP, RequestItem};
use crate::network::network::{Request, Response};

/// Handles events from the request-response protocol.
///
/// Processes different types of events such as inbound and outbound failures, and incoming messages.
pub async fn handle_event(event: libp2p::request_response::Event<Request, Response>) {
    match event {
        // Handles inbound failures by logging the error
        request_response::Event::InboundFailure { error, .. } => {
            logger::info!("Inbound Error: {}", error);
        }

        // Handles outbound failures by logging the error
        request_response::Event::OutboundFailure { error, .. } => {
            logger::info!("Outbound Failure: {}", error);
        }

        // Handles incoming messages
        request_response::Event::Message { peer, message } => {
            match message {
                // Handles requests by logging the request and adding it to the current requests list
                Message::Request { request, channel, .. } => {
                    logger::info!("Received request: {:?}", request);
                    let mut app = APP.lock().unwrap();
                    let new_request = RequestItem {
                        peer_id: peer,
                        request_string: request.request,
                        response_channel: channel,
                    };
                    app.current_requests.push(new_request);
                },

                // Handles responses by logging the response and saving the file data
                Message::Response { response, .. } => {
                    logger::info!("Received response: {:?}", response);

                    // Write the response data to a file
                    if let Err(e) = std::fs::write("new_".to_owned() + &response.filename, response.data) {
                        logger::error!("Error writing file {:?}: {:?}", &response.filename, e);
                    } else {
                        logger::info!("File {:?} received and saved successfully", &response.filename);
                    }
                },
            }
        }
        _ => {}
    }
}
