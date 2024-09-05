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
use libp2p_request_response::ResponseChannel;
use libp2p::kad::store::{MemoryStore, RecordStore};



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
    
    let mut app = APP.lock().unwrap();
    app.my_peer_id = Some(swarm.local_peer_id().clone());
    for room in &app.rooms {
        // Create a Gossipsub topic
        let topic = gossipsub::IdentTopic::new(room);
        // subscribes to our topic
        swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
    }
    drop(app);
    swarm
        .behaviour_mut()
        .kademlia
        .set_mode(Some(kad::Mode::Server));

    let (command_sender, command_receiver) = mpsc::channel(0);
    // let (event_sender, _) = mpsc::channel(0);

    Ok((
        Client {
            sender: command_sender,
        },
        EventLoop::new(swarm, command_receiver),
    ))
}

#[derive(Clone)]
pub struct Client {
    sender: mpsc::Sender<Command>,
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

    pub(crate) async fn submit_message(
        &mut self,
        message: String,
        topic: IdentTopic,
    ) {
        logger::info!("Submitting message: {:?}", message.clone());
        // Provide a default room name or handle the case where the room is not found
        self.sender
        .send(Command::SendMessage { message: message.clone(), topic})
        .await
        .expect("Message Sent.");
        logger::info!("message sent: {:?}", message.clone());

    }

    pub(crate) async fn send_request(
        &mut self,
        request: String,
        peer: PeerId
    ) {
        self.sender
            .send(Command::RequestFile { request, peer })
            .await
            .expect("Command receiver not to be dropped.");
    }

    pub(crate) async fn send_response(
        &mut self,
        filename: String,
        filepath: String,
        channel: ResponseChannel<Response>
    ) {
        self.sender
            .send(Command::RespondFile { filename, filepath, channel })
            .await
            .expect("Command receiver not to be dropped.");
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

    pub(crate) async fn get_rooms(
        &mut self,
    ) {
        self.sender
            .send(Command::GetRooms { })
            .await
            .expect("Rooms got.");
    }

    pub(crate) async fn create_room(
        &mut self,
        chat_name: String,
    ) {
        logger::info!("Creating Chat room: {:?}", chat_name.clone());

        self.sender
            .send(Command::CreateRoom { chat_name })
            .await
            .expect("Room Created.");
    }
}

pub(crate) struct EventLoop {
    swarm: Swarm<Behaviour>,
    command_receiver: mpsc::Receiver<Command>,
}

impl EventLoop {
    fn new(
        swarm: Swarm<Behaviour>,
        command_receiver: mpsc::Receiver<Command>,
    ) -> Self {
        Self {
            swarm,
            command_receiver,
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
        match event {
            SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(event)) => {
                gossipsub_behaviour::handle_event(event).await;
            },
            
             // Handle MDNS events
            SwarmEvent::Behaviour(BehaviourEvent::Mdns(event)) => {
                mdns_behaviour::handle_event(event, &mut self.swarm).await;
            },

            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(event)) => {
                kademlia_behaviour::handle_event(event, &mut self.swarm).await;
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
            
           

            SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(event)) => {
                request_response_behaviour::handle_event(event).await;
            },
        
            SwarmEvent::NewListenAddr { address, .. } => {
                log::info!("New listening address: {address}");
                let peer_id = self.swarm.local_peer_id().clone();
                self.swarm.behaviour_mut().kademlia.add_address(&peer_id, address);
            },
        
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
            Command::RequestFile { request, peer, } => {
                self
                    .swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&peer, Request { request });
            }
            Command::RespondFile { filename, filepath, channel } => {
                let data = std::fs::read(&filepath).unwrap_or_else(|_| Vec::new());
                
                log::info!("Data provided is empty: {:?}", data.is_empty());
                self.swarm
                    .behaviour_mut()
                    .request_response
                    .send_response(channel, Response {filename, data })
                    .expect("Connection to peer to be still open.");
            }
            Command::GetUsername { peer_id } => {
                // Get's a username based on a peer_id, ensuring it is added to the "app.username" hashmap for use throughout the app
                logger::info!("Getting username");
                let key = kad::RecordKey::new(&peer_id);
                self.swarm.behaviour_mut().kademlia.get_record(key);
            }
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
            Command::GetRooms {  } => {
                let key = kad::RecordKey::new(&"room_store".to_string());
                self.swarm.behaviour_mut().kademlia.get_record(key);
            }
            Command::CreateRoom { chat_name } => {
                // Get's a username based on a peer_id, ensuring it is added to the "app.username" hashmap for use throughout the app
                logger::info!("Creating Room");
                let serial_chat_name = serde_cbor::to_vec(&"room_store").unwrap();
                let key = kad::RecordKey::new(&"room_store".to_string());
                let record = self.swarm.behaviour_mut().kademlia.store_mut().get(&key);

                if !record.is_none() {
                    let mut room_store: Vec<String> = match serde_cbor::from_slice(&record.unwrap().value) {
                        Ok(room_store) => room_store,
                        Err(e) => {
                            logger::info!("Failed to deserialize room list: {:?}", e);
                            return;
                        }
                    };
                    if !room_store.contains(&chat_name.clone()) {
                         room_store.push(chat_name.clone());
                        let rooms_bytes = serde_cbor::to_vec(&room_store).unwrap();
                        let record = kad::Record {
                            key: kad::RecordKey::new(&key),
                            value: rooms_bytes,
                            publisher: None,
                            expires: None,
                        };
                        self.swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One).expect("Failed to store record locally");
                        logger::info!("No errors in storing room name");
                    } else {
                        logger::info!("Room Store already contains: {:?}", chat_name.clone());
                    }
                } else {
                    let mut room_store = Vec::new();
                    room_store.push(chat_name.clone());
                    let rooms_bytes = serde_cbor::to_vec(&room_store).unwrap();
    
                    let record = kad::Record {
                        key: kad::RecordKey::new(&key),
                        value: rooms_bytes,
                        publisher: None,
                        expires: None,
                    };

                    self.swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One).expect("Failed to store record locally");
                    logger::info!("No errors in storing room name");
                }
                
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
    RequestFile {
        request: String,
        peer: PeerId,
    },
    RespondFile {
        filename: String,
        filepath: String,
        channel: ResponseChannel<Response>
    },
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
    GetRooms {},
    CreateRoom {
        chat_name: String
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub request: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub filename: String,
    pub data: Vec<u8>,
}