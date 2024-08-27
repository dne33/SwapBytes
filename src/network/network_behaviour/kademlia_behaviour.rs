use libp2p::{Swarm, kad, PeerId};
use crate::logger;
use crate::network::network::Behaviour;
use futures::channel::oneshot;
use std::collections::{HashMap, HashSet};
use crate::APP;



pub async fn handle_event(
    event: libp2p::kad::Event,
    _swarm: &mut Swarm<Behaviour>,
    _pending_start_providing: &mut HashMap<kad::QueryId, oneshot::Sender<()>>,
    _pending_get_providers: &mut HashMap<kad::QueryId, oneshot::Sender<HashSet<PeerId>>>,
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
                        match serde_cbor::from_slice::<String>(&value) {
                            Ok(username) => {
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
                            }
                            Err(e) => {
                                logger::error!("Error deserializing: {e:?}");
                            }
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

// 