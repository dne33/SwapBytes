use libp2p::{Swarm, mdns};
use crate::logger;
use crate::network::network::Behaviour;
use crate::state::APP;
use libp2p::gossipsub::IdentTopic;
use std::cmp::Ordering;



pub async fn handle_event(event: libp2p::mdns::Event, swarm: &mut Swarm<Behaviour>) {
    match event {
        mdns::Event::Discovered(list) => {
            for (peer_id, multiaddr) in list {
                    logger::info!("mDNS discover peer: {peer_id}");
                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
                    let mut app = APP.lock().unwrap();
                    app.peers.push(peer_id.clone());
                    app.peers_no_username.push(peer_id.clone());
                    app.connected_peers += 1;  
                    drop(app);
                    // Create the gossipsub topic by combining `my_peer_id` and `peer_id` alphabetically
                    // This will make it easy to send DMs to each person
                    let peer_id_str = peer_id.to_string();
                    let my_peer_id = swarm.local_peer_id().clone().to_string();
                    let topic_name = match my_peer_id.cmp(&peer_id_str) {
                        Ordering::Less => format!("{}_{}", my_peer_id, peer_id_str),
                        _ => format!("{}_{}", peer_id_str, my_peer_id),
                    };

                    // Create a gossipsub topic from the combined string
                    let topic = IdentTopic::new(topic_name.clone());
                    
                    // Subscribe to the gossipsub topic
                    if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&topic) {
                        logger::error!("Failed to subscribe to gossipsub topic {}: {}", topic_name.clone(), e);
                    } else {
                        logger::info!("Subscribed to gossipsub topic: {}", topic_name);
                    }  
            }
        }


        mdns::Event::Expired(list) => {
            for (peer_id, _) in list {
                logger::info!("mDNS peer has expired: {peer_id}");
                swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                let mut app = APP.lock().unwrap();
                app.connected_peers -= 1;
                
            }
        }
    }
}