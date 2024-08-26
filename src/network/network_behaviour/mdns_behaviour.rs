use libp2p::{Swarm, mdns};
use crate::logger;
use crate::network::network::Behaviour;
use crate::state::APP;



pub async fn handle_event(event: libp2p::mdns::Event, swarm: &mut Swarm<Behaviour>) {
    match event {
        mdns::Event::Discovered(list) => {
            for (peer_id, multiaddr) in list {
                    logger::info!("mDNS discover peer: {peer_id}");
                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
                    let mut app = APP.lock().unwrap();
                    app.peers.push(peer_id);
                    app.connected_peers += 1;
                    
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