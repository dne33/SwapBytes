use libp2p::{Swarm, mdns};
use crate::logger;
use crate::network::network::Behaviour;
use crate::state::APP;
use libp2p::gossipsub::IdentTopic;
use std::cmp::Ordering;

/// Handles mDNS events and updates the swarm and application state accordingly.
///
/// This function processes discovered peers and expired peers from mDNS events.
pub async fn handle_event(event: libp2p::mdns::Event, swarm: &mut Swarm<Behaviour>) {
    match event {
        // Handle discovered peers
        mdns::Event::Discovered(list) => {
            for (peer_id, multiaddr) in list {
                logger::info!("mDNS discovered peer: {}", peer_id);
                
                // Add the discovered peer to gossipsub and kademlia
                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
                
                // Update the application state with the discovered peer
                let mut app = APP.lock().unwrap();
                app.peers.push(peer_id.clone());
                app.peers_no_username.push(peer_id.clone());
                app.connected_peers += 1;
                
                // Create a gossipsub topic for direct messaging between peers
                let peer_id_str = peer_id.to_string();
                let my_peer_id = swarm.local_peer_id().clone().to_string();
                // Construct the message key using the sorted peer IDs
                let mut peer_ids = vec![peer_id_str.clone(), my_peer_id.clone()];
                peer_ids.sort(); // Sort alphabetically
                let topic_name = peer_ids.join("_");
                // Create a gossipsub topic from the combined string
                let topic = IdentTopic::new(topic_name.clone());
                
                // Subscribe to the created gossipsub topic
                if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&topic) {
                    logger::error!("Failed to subscribe to gossipsub topic {}: {}", topic_name, e);
                } else {
                    logger::info!("Subscribed to gossipsub topic: {}", topic_name);
                    app.private_messages.insert(topic_name, Vec::new());
                }  
            }
        }

        // Handle expired peers
        mdns::Event::Expired(list) => {
            for (peer_id, _) in list {
                logger::info!("mDNS peer has expired: {}", peer_id);
                
                // Remove the expired peer from gossipsub
                swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                
                // Update the application state to reflect the expired peer
                let mut app = APP.lock().unwrap();
                app.connected_peers -= 1;
            }
        }
    }
}
