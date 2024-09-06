use libp2p::{Swarm, kad};
use crate::logger;
use crate::network::network::Behaviour;
use std::collections::HashSet;
use libp2p::gossipsub::IdentTopic;

use crate::APP;


pub async fn handle_event(
    event: libp2p::kad::Event, swarm: &mut Swarm<Behaviour>
) {
    match event {
        kad::Event::OutboundQueryProgressed { result, .. } => {
                match result {
                    kad::QueryResult::GetRecord(Ok(
                        kad::GetRecordOk::FoundRecord(kad::PeerRecord {
                            record: kad::Record { key, value, .. },
                            ..
                        })
                    )) => {
                        if let Ok(username) = serde_cbor::from_slice::<String>(&value) {
                                logger::info!(
                                    "Got record {:?} {:?}", 
                                    std::str::from_utf8(key.as_ref()).unwrap(),
                                    username.clone(),
                                ); 
                                let mut app = APP.lock().unwrap();
                                app.usernames.insert(std::str::from_utf8(key.as_ref()).unwrap().to_string(), username);
                                if app.peers_no_username.len() != 0 && app.peers_no_username.iter().any(|&i| i.to_string() == std::str::from_utf8(key.as_ref()).unwrap().to_string()) {
                                    let index = app.peers_no_username.iter().position(|x| *x.to_string() == std::str::from_utf8(key.as_ref()).unwrap().to_string()).unwrap();
                                    app.peers_no_username.remove(index);
                                }
                            } else if let Ok(room_store) = serde_cbor::from_slice::<Vec<String>> (&value) {
                               logger::info!(
                                    "Got room store", 
                                ); 
                                let mut app = APP.lock().unwrap();
                                let mut room_set: HashSet<String> = app.rooms.clone().iter().cloned().collect(); // Create a HashSet from the target vector
                                for room in room_store {
                                    if room_set.insert(room.clone()) { // If insertion was successful (i.e., item was not in the set)
                                        // Create a gossipsub topic from the combined string
                                        let topic = IdentTopic::new(room.clone());
                                        // Subscribe to the gossipsub topic
                                        if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&topic) {
                                            logger::error!("Failed to subscribe to gossipsub topic {}: {}", room.clone(), e);
                                        } else {
                                            logger::info!("Subscribed to gossipsub topic: {}", room.clone());                  
                                            app.public_messages.insert(room.clone(), Vec::new());
                                            app.rooms.push(room);
                                        } 
                                    }
                                }
                            } else  {
                                logger::error!("Error deserializing: Invalid data format");
                            }
                        }
                    
                    kad::QueryResult::GetRecord(Ok(_)) => {}
                    kad::QueryResult::GetRecord(Err(err)) => {
                        logger::info!("Failed to get record {err:?}");
                    }
                    kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key })) => {
                        logger::info!("Successfully put record {:?}", std::str::from_utf8(key.as_ref()).unwrap());
                    }
                    kad::QueryResult::PutRecord(Err(err)) => {
                        logger::error!("Failed to put record: {err:?}");
                    }
                    _ => {}
                }
            }
            _ => {logger::info!{"huh {:?}", event}}
    }
}