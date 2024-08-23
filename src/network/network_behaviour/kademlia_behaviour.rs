use libp2p::{Swarm, kad, PeerId};
use crate::logger;
use crate::network::network::Behaviour;
use crate::state::APP;
use futures::channel::oneshot;
use std::collections::{HashMap, HashSet};



pub async fn handle_event(
    event: libp2p::kad::Event,
    swarm: &mut Swarm<Behaviour>,
    pending_start_providing: &mut HashMap<kad::QueryId, oneshot::Sender<()>>,
    pending_get_providers: &mut HashMap<kad::QueryId, oneshot::Sender<HashSet<PeerId>>>,
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
                                    username,
                                )
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
            _ => {}
    }
}

// 