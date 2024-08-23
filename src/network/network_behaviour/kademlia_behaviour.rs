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
        kad::Event::OutboundQueryProgressed {
            id,
            result: kad::QueryResult::StartProviding(_),
            ..
        } => {
        let sender: oneshot::Sender<()> =
            pending_start_providing
            .remove(&id)
            .expect("Completed query to be previously pending.");
        let _ = sender.send(());
        },
        kad::Event::OutboundQueryProgressed {
                id,
                result:
                    kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders {
                        providers,
                        ..
                    })),
                ..
            } => {
            if let Some(sender) = pending_get_providers.remove(&id) {
                sender.send(providers).expect("Receiver not to be dropped");

                // Finish the query. We are only interested in the first result.
                swarm
                    .behaviour_mut()
                    .kademlia
                    .query_mut(&id)
                    .unwrap()
                    .finish();
            }
        },
        kad::Event::OutboundQueryProgressed {
                result:
                    kad::QueryResult::GetProviders(Ok(
                        kad::GetProvidersOk::FinishedWithNoAdditionalRecord { .. },
                    )),
                ..
        } => {},
        _ => {}
    }
}

// 