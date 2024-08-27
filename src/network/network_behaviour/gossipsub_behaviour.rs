use libp2p::gossipsub;
use crate::logger;
use crate::state::APP;

pub async fn handle_event(event: libp2p::gossipsub::Event) {
    match event {
        gossipsub::Event::Message {
            propagation_source: peer_id,
            message_id: _,
            message,
        } =>  {
            logger::info!("In the swarm behaviour for receiving");
        
            // Convert the message data to a string
            let msg = String::from_utf8_lossy(&message.data);
            
            // Get the topic name as a string
            let topic_name = message.topic.as_str();

            // Format the final message with topic, message, and peer ID
            let final_msg = format!("{msg} from {peer_id} on topic {topic_name}");
            
            // Lock the app to update the messages
            let mut app = APP.lock().unwrap();
            // Insert the message into the appropriate room/messages vector based on the topic
            app.messages.entry(topic_name.to_string())
                .or_insert_with(Vec::new)
                .push(final_msg.clone());
            
            // Log the received message
            logger::info!("Received message: {}", final_msg);
        },
        _ => {}
    }
}