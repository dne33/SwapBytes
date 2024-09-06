use libp2p::{Swarm, kad};
use crate::logger;
use crate::network::network::Behaviour;
use std::collections::HashSet;
use libp2p::gossipsub::IdentTopic;
use crate::APP;

/// Handles Kademlia (kad) events and updates the swarm and application state accordingly.
///
/// This function processes various query results from the Kademlia protocol, including
/// record retrieval and storage operations.
pub async fn handle_event(
    event: libp2p::kad::Event, swarm: &mut Swarm<Behaviour>
) {
    match event {
        // Handle outbound query progress
        kad::Event::OutboundQueryProgressed { result, .. } => {
            match result {
                // Handle successful record retrieval
                kad::QueryResult::GetRecord(Ok(
                    kad::GetRecordOk::FoundRecord(kad::PeerRecord {
                        record: kad::Record { key, value, .. },
                        ..
                    })
                )) => {
                    // Attempt to deserialize the record value into a username
                    if let Ok(username) = serde_cbor::from_slice::<String>(&value) {
                        logger::info!(
                            "Got record {:?} {:?}", 
                            std::str::from_utf8(key.as_ref()).unwrap(),
                            username.clone(),
                        );
                        
                        // Update application state with the retrieved username
                        let mut app = APP.lock().unwrap();
                        app.usernames.insert(std::str::from_utf8(key.as_ref()).unwrap().to_string(), username);
                        
                        // Remove peer without username if it exists
                        if !app.peers_no_username.is_empty() && app.peers_no_username.iter().any(|&i| i.to_string() == std::str::from_utf8(key.as_ref()).unwrap().to_string()) {
                            let index = app.peers_no_username.iter().position(|x| *x.to_string() == std::str::from_utf8(key.as_ref()).unwrap().to_string()).unwrap();
                            app.peers_no_username.remove(index);
                        }
                    } 
                    // Attempt to deserialize the record value into a list of rooms
                    else if let Ok(room_store) = serde_cbor::from_slice::<Vec<String>>(&value) {                        
                        let mut app = APP.lock().unwrap();
                        let mut room_set: HashSet<String> = app.rooms.clone().iter().cloned().collect();
                        
                        for room in room_store {
                            // Add new rooms to the application state and subscribe to gossipsub topics
                            if room_set.insert(room.clone()) {
                                let topic = IdentTopic::new(room.clone());
                                
                                if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&topic) {
                                    logger::error!("Failed to subscribe to gossipsub topic {}: {}", room.clone(), e);
                                } else {
                                    logger::info!("Subscribed to gossipsub topic: {}", room.clone());
                                    app.public_messages.insert(room.clone(), Vec::new());
                                    app.rooms.push(room);
                                }
                            }
                        }
                    } else {
                        logger::error!("Error deserializing: Invalid data format");
                    }
                }
                
                // Handle other query results
                kad::QueryResult::GetRecord(Ok(_)) => {}
                kad::QueryResult::GetRecord(Err(err)) => {
                    logger::info!("Failed to get record {:?}", err);
                }
                kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key })) => {
                    logger::info!("Successfully put record {:?}", std::str::from_utf8(key.as_ref()).unwrap());
                }
                kad::QueryResult::PutRecord(Err(err)) => {
                    logger::error!("Failed to put record: {:?}", err);
                }
                _ => {}
            }
        }
        _ => {}
    }
}