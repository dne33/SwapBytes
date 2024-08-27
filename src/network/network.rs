use futures::channel::{mpsc, oneshot};
use futures::prelude::*;
use futures::StreamExt;

use libp2p::{
    core::Multiaddr,
    kad,
    noise,
    request_response::{self, ProtocolSupport},
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp, yamux, PeerId,
    gossipsub, mdns, 
};
use libp2p::gossipsub::IdentTopic;
use libp2p::StreamProtocol;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::time::Duration;
use crate::network::network_behaviour::{mdns_behaviour, gossipsub_behaviour, kademlia_behaviour, request_response_behaviour};
use crate::state::APP;
use crate::logger;


pub(crate) async fn new() -> Result<(Client, EventLoop), Box<dyn Error>> {
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
            let kademlia = kad::Behaviour::new(
                key.public().to_peer_id(),
                kad::store::MemoryStore::new(key.public().to_peer_id()),
            );
            let request_response = request_response::cbor::Behaviour::new(
                [(
                    StreamProtocol::new("/file-exchange/1"),
                    ProtocolSupport::Full,
                )],
                request_response::Config::default(),
            );
            
            // build a gossipsub network behaviour
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub::Config::default(),
            )?;

            let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
            Ok(Behaviour { kademlia, request_response, gossipsub, mdns })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();
     // Create a Gossipsub topic
    let topic = gossipsub::IdentTopic::new("test-net");
    // subscribes to our topic
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
    swarm
        .behaviour_mut()
        .kademlia
        .set_mode(Some(kad::Mode::Server));

    let (command_sender, command_receiver) = mpsc::channel(0);
    // let (event_sender, _) = mpsc::channel(0);

    Ok((
        Client {
            sender: command_sender,
            topic: topic,
        },
        EventLoop::new(swarm, command_receiver),
    ))
}

#[derive(Clone)]
pub struct Client {
    sender: mpsc::Sender<Command>,
    topic: IdentTopic,
}

impl Client {
    /// Listen for incoming connections on the given address.
    pub(crate) async fn start_listening(
        &mut self,
        addr: Multiaddr,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(Command::StartListening { addr, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    // /// Dial the given peer at the given address.
    // pub(crate) async fn dial(
    //     &mut self,
    //     peer_id: PeerId,
    //     peer_addr: Multiaddr,
    // ) -> Result<(), Box<dyn Error + Send>> {
    //     let (sender, receiver) = oneshot::channel();
    //     self.sender
    //         .send(Command::Dial {
    //             peer_id,
    //             peer_addr,
    //             sender,
    //         })
    //         .await
    //         .expect("Command receiver not to be dropped.");
    //     receiver.await.expect("Sender not to be dropped.")
    // }

    // /// Advertise the local node as the provider of the given file on the DHT.
    // pub(crate) async fn start_providing(&mut self, file_name: String) {
    //     let (sender, receiver) = oneshot::channel();
    //     self.sender
    //         .send(Command::StartProviding { file_name, sender })
    //         .await
    //         .expect("Command receiver not to be dropped.");
    //     receiver.await.expect("Sender not to be dropped.");
    // }

    // /// Find the providers for the given file on the DHT.
    // pub(crate) async fn get_providers(&mut self, file_name: String) -> HashSet<PeerId> {
    //     let (sender, receiver) = oneshot::channel();
    //     self.sender
    //         .send(Command::GetProviders { file_name, sender })
    //         .await
    //         .expect("Command receiver not to be dropped.");
    //     receiver.await.expect("Sender not to be dropped.")
    // }

    // /// Request the content of the given file from the given peer.
    // pub(crate) async fn request_file(
    //     &mut self,
    //     peer: PeerId,
    //     file_name: String,
    // ) -> Result<Vec<u8>, Box<dyn Error + Send>> {
    //     let (sender, receiver) = oneshot::channel();
    //     self.sender
    //         .send(Command::RequestFile {
    //             file_name,
    //             peer,
    //             sender,
    //         })
    //         .await
    //         .expect("Command receiver not to be dropped.");
    //     receiver.await.expect("Sender not be dropped.")
    // }

    // /// Respond with the provided file content to the given request.
    // pub(crate) async fn respond_file(
    //     &mut self,
    //     file: Vec<u8>,
    //     channel: ResponseChannel<Response>,
    // ) {
    //     self.sender
    //         .send(Command::RespondFile { file, channel })
    //         .await
    //         .expect("Command receiver not to be dropped.");
    // }

    pub(crate) async fn submit_message(
        &mut self,
        message: String,
    ) {
        logger::info!("Submitting message: {:?}", message.clone());

        self.sender
            .send(Command::SendMessage { message, topic: self.topic.clone() })
            .await
            .expect("Message Sent.");
    }

    pub(crate) async fn push_username(
        &mut self,
        username: String,
    ) {
        logger::info!("Pushing username: {:?}", username.clone());

        self.sender
            .send(Command::PushUsername { username })
            .await
            .expect("username Pushed.");
    }

    pub(crate) async fn get_username(
        &mut self,
        peer_id: String,
    ) {
        logger::info!("Getting username for peer_id: {:?}", peer_id.clone());

        self.sender
            .send(Command::GetUsername { peer_id })
            .await
            .expect("username got.");
    }
}

pub(crate) struct EventLoop {
    swarm: Swarm<Behaviour>,
    command_receiver: mpsc::Receiver<Command>,
    pending_dial: HashMap<PeerId, oneshot::Sender<Result<(), Box<dyn Error + Send>>>>,
    pending_start_providing: HashMap<kad::QueryId, oneshot::Sender<()>>,
    pending_get_providers: HashMap<kad::QueryId, oneshot::Sender<HashSet<PeerId>>>,
    // pending_request_file:
    //     HashMap<OutboundRequestId, oneshot::Sender<Result<Vec<u8>, Box<dyn Error + Send>>>>,
}

impl EventLoop {
    fn new(
        swarm: Swarm<Behaviour>,
        command_receiver: mpsc::Receiver<Command>,
    ) -> Self {
        Self {
            swarm,
            command_receiver,
            // event_sender,
            pending_dial: Default::default(),
            pending_start_providing: Default::default(),
            pending_get_providers: Default::default(),
            // pending_request_file: Default::default(),
        }
    }

    pub(crate) async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_event(event).await,
                command = self.command_receiver.next() => match command {
                    Some(c) => self.handle_command(c).await,
                    None => return,
                },
            }
        }
    }

    async fn handle_event(&mut self, event: SwarmEvent<BehaviourEvent>) {
        logger::info!("Event happened {:?}", event);
        
        match event {
            SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(event)) => {
                gossipsub_behaviour::handle_event(event).await;
            },
            
             // Handle MDNS events
            SwarmEvent::Behaviour(BehaviourEvent::Mdns(event)) => {
                mdns_behaviour::handle_event(event, &mut self.swarm).await;
            },
            
           
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                logger::info!("Connection closed for peer: {peer_id}");
                self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                let mut app = APP.lock().unwrap();
                app.connected_peers -= 1;
                
                // Remove item from a list (https://stackoverflow.com/questions/26243025/how-to-remove-an-element-from-a-vector-given-the-element)
                if app.peers.iter().any(|x| *x == peer_id) {
                    let index = app.peers.iter().position(|x| *x == peer_id).unwrap();
                    app.peers.remove(index);
                }
                    
                if app.peers_no_username.contains(&peer_id) {
                    let index = app.peers_no_username.iter().position(|x| *x == peer_id).unwrap();
                    app.peers_no_username.remove(index);
                }
            },
            
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(event)) => {
                kademlia_behaviour::handle_event(event, &mut self.swarm, &mut self.pending_start_providing, &mut self.pending_get_providers).await;
            },

            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(event)) => {
                request_response_behaviour::handle_event(event).await;
            },
        
            SwarmEvent::NewListenAddr { address, .. } => {
                log::info!("New listening address: {address}");
                let peer_id = self.swarm.local_peer_id().clone();
                self.swarm.behaviour_mut().kademlia.add_address(&peer_id, address);
            },
            
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                if endpoint.is_dialer() {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Ok(()));
                    }
                }
            },
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Err(Box::new(error)));
                    }
                }
            },
            SwarmEvent::IncomingConnection { .. } => {},
            SwarmEvent::IncomingConnectionError { .. } => {},
            e => logger::error!("{e:?}"),
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::SendMessage { message, topic } => {
                let _ = self.swarm.behaviour_mut().gossipsub.publish(topic.clone(), message.clone().as_bytes());
                logger::info!("{} sent successfully.", message.clone());
            }

            Command::StartListening { addr, sender } => {
                let _ = match self.swarm.listen_on(addr) {
                    Ok(_) => sender.send(Ok(())),
                    Err(e) => sender.send(Err(Box::new(e))),
                };
            }
            // Command::Dial {
            //     peer_id,
            //     peer_addr,
            //     sender,
            // } => {
            //     if let hash_map::Entry::Vacant(e) = self.pending_dial.entry(peer_id) {
            //         self.swarm
            //             .behaviour_mut()
            //             .kademlia
            //             .add_address(&peer_id, peer_addr.clone());
            //         match self.swarm.dial(peer_addr.with(Protocol::P2p(peer_id))) {
            //             Ok(()) => {
            //                 e.insert(sender);
            //             }
            //             Err(e) => {
            //                 let _ = sender.send(Err(Box::new(e)));
            //             }
            //         }
            //     } else {
            //         todo!("Already dialing peer.");
            //     }
            // }
            // Command::StartProviding { file_name, sender } => {
            //     let query_id = self
            //         .swarm
            //         .behaviour_mut()
            //         .kademlia
            //         .start_providing(file_name.into_bytes().into())
            //         .expect("No store error.");
            //     self.pending_start_providing.insert(query_id, sender);
            // }
            // Command::GetProviders { file_name, sender } => {
            //     let query_id = self
            //         .swarm
            //         .behaviour_mut()
            //         .kademlia
            //         .get_providers(file_name.into_bytes().into());
            //     self.pending_get_providers.insert(query_id, sender);
            // }
            // Command::RequestFile {
            //     file_name,
            //     peer,
            //     sender,
            // } => {
            //     let request_id = self
            //         .swarm
            //         .behaviour_mut()
            //         .request_response
            //         .send_request(&peer, Request(file_name));
            //     self.pending_request_file.insert(request_id, sender);
            // }
            // Command::RespondFile { file, channel } => {
            //     self.swarm
            //         .behaviour_mut()
            //         .request_response
            //         .send_response(channel, Response(file))
            //         .expect("Connection to peer to be still open.");
            // }
            Command::PushUsername { username } => {
                logger::info!("Attempting to add username");
                let serial_username = serde_cbor::to_vec(&username).unwrap();
                let record = kad::Record {
                    key: kad::RecordKey::new(&self.swarm.local_peer_id().to_string()),
                    value: serial_username,
                    publisher: None,
                    expires: None,
                };

                self.swarm.behaviour_mut().kademlia
                    .put_record(record, kad::Quorum::One)
                    .expect("Failed to store record locally");
                logger::info!("No errors in storing username");

            }
            Command::GetUsername { peer_id } => {
                // Get's a username based on a peer_id, ensuring it is added to the "app.username" hashmap for use throughout the app
                logger::info!("Getting username");
                let key = kad::RecordKey::new(&peer_id);
                self.swarm.behaviour_mut().kademlia.get_record(key);
            }
        
        }
    }
}

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub request_response: request_response::cbor::Behaviour<Request, Response>,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

#[derive(Debug)]
enum Command {
    StartListening {
        addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    // Dial {
    //     peer_id: PeerId,
    //     peer_addr: Multiaddr,
    //     sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    // },
    // StartProviding {
    //     file_name: String,
    //     sender: oneshot::Sender<()>,
    // },
    // GetProviders {
    //     file_name: String,
    //     sender: oneshot::Sender<HashSet<PeerId>>,
    // },
    // RequestFile {
    //     file_name: String,
    //     peer: PeerId,
    //     sender: oneshot::Sender<Result<Vec<u8>, Box<dyn Error + Send>>>,
    // },
    // RespondFile {
    //     file: Vec<u8>,
    //     channel: ResponseChannel<Response>,
    // },
    SendMessage {
        message: String,
        topic: IdentTopic,
    },
    PushUsername {
        username: String,
    },
    GetUsername {
        peer_id: String
    },
}

// #[derive(Debug)]
// pub(crate) enum Event {
//     InboundRequest {
//         request: String,
//         channel: ResponseChannel<Response>,
//     },
// }

// Simple file exchange protocol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request(String);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response(Vec<u8>);