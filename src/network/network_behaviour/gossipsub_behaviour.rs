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
            logger::info!("In the swarm behaviour for recieving");
            let msg = String::from_utf8_lossy(&message.data);
            let final_msg = format!( "{msg} : {peer_id}");
            let mut app = APP.lock().unwrap();
            app.messages.push(
                final_msg.clone()
            );
            
            logger::info!("Recieved message final_msg");
        },
        _ => {}
    }
}