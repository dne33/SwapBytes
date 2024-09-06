use libp2p::gossipsub;
use crate::logger;
use crate::state::APP;

pub async fn handle_event(event: libp2p::gossipsub::Event) {
    match event {
        gossipsub::Event::Message {
            message,
            ..
        } =>  {
            logger::info!("In the swarm behaviour for receiving");
        
            let msg = String::from_utf8_lossy(&message.data);
            
            let topic_name = message.topic.as_str();

            let final_msg = format!("{msg}");
            
            let mut app = APP.lock().unwrap();
            
            // Check length as room names are restricted to 64 chars
            if topic_name.to_string().len() <= 64 {
                // Insert the message into the appropriate room vector based on the topic
                app.public_messages.entry(topic_name.to_string())
                                .or_insert_with(Vec::new)
                                .push(final_msg.clone());
            // Else it's a private message and can be stored as such
            } else {
                // Insert the message into the appropriate users vector based on the topic
                app.private_messages.entry(topic_name.to_string())
                                .or_insert_with(Vec::new)
                                .push(final_msg.clone());
            }
            
            // Log the received message
            logger::info!("Received message: {}", final_msg);
        },
        _ => {}
    }
}