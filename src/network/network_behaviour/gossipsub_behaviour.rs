use libp2p::gossipsub;
use crate::logger;
use crate::state::APP;

/// Handles events from the gossipsub protocol and updates the application state accordingly.
///
/// This function processes incoming messages from the gossipsub protocol and
/// categorizes them as either public or private messages based on the topic name.
pub async fn handle_event(event: libp2p::gossipsub::Event) {
    match event {
        // Handle incoming gossipsub messages
        gossipsub::Event::Message {
            message,
            ..
        } => {
            logger::info!("In the swarm behaviour for receiving");

            // Convert message data to a string
            let msg = String::from_utf8_lossy(&message.data);
            let topic_name = message.topic.as_str();
            let final_msg = format!("{}", msg);

            let mut app = APP.lock().unwrap();

            // Check if the topic name length is within the allowed limit for room names
            if topic_name.to_string().len() <= 64 {
                // Insert the message into the public messages vector for the room
                app.public_messages
                    .entry(topic_name.to_string())
                    .or_insert_with(Vec::new)
                    .push(final_msg.clone());
            } else {
                // Otherwise, treat it as a private message
                app.private_messages
                    .entry(topic_name.to_string())
                    .or_insert_with(Vec::new)
                    .push(final_msg.clone());
            }

            // Log the received message
            logger::info!("Received message: {}", final_msg);
        },
        _ => {}
    }
}