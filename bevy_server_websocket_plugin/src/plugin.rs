use bevy::prelude::*;
use websocket::sync::Server;
use websocket::OwnedMessage;
use std::{thread, net::SocketAddr, time::Duration};
use crossbeam_channel::unbounded;
use crossbeam::channel::{Sender, Receiver};  
use std::sync::{Arc,Mutex};

use super::client_list::ClientStore;
use super::events;
use super::Message;

///The plugin for manageing web socket connections with the Bevy Game engine
pub struct WebSocketPlugin{
    pub ip:String,
    pub port:u16,
}


impl Plugin for WebSocketPlugin{
    ///Adds initializes the network manager and registers ServerMessageEvent,
    ///PlayerDisceonnectedEvent and PlayerConnectedEvent
    fn build(&self, app : &mut App){
       app.add_systems(Startup, spawn_network_manager).
           add_systems(FixedUpdate, (listen_for_messages, listen_for_client_connections)).
           add_event::<events::ServerMessageEvent>().
           add_event::<events::PlayerDisconectedEvent>().
           add_event::<events::PlayerConnectedEvent>();
            
    }
}

// spawns the network manager in the game world, it also sets up the network manager
fn spawn_network_manager(mut commands: Commands){
    let network_manager = NetworkManager::new();
    network_manager.setup();
    commands.spawn(network_manager);
}

// listens for messages and re emmits them as Bevy envents
fn listen_for_messages(mut network_manager_query : Query<&mut NetworkManager>,
                       mut message_event_writer: EventWriter<events::ServerMessageEvent>,
                       mut disconnect_event_writer: EventWriter<events::PlayerDisconectedEvent>){
    
    let network_manager = network_manager_query.single_mut();
    while let Ok(data) = network_manager.receiver_from_client.try_recv(){
        let ip = data.0;
        let message = data.1;
        match message.clone(){
            OwnedMessage::Text(text)=>{
                let message = Message::Text(text);
                message_event_writer.send(events::ServerMessageEvent(ip,message)); 
            }
            OwnedMessage::Binary(bin)=>{
                let message = Message::Binary(bin);
                message_event_writer.send(events::ServerMessageEvent(ip,message));
            }
            OwnedMessage::Close(_closing_data)=>{
               network_manager.sender_to_client.lock().unwrap().remove(&ip);
                disconnect_event_writer.send(events::PlayerDisconectedEvent(ip)); 
            }
            _=>{
            } 
        }
    }
}

// Waits for a new client to join and emits their ip as an event upon joining
fn listen_for_client_connections(mut network_manager_query : Query<&mut NetworkManager>,
                                 mut client_connected_event_writer: EventWriter<events::PlayerConnectedEvent>){
    
    let network_manager = network_manager_query.single_mut();

    while let Ok(id) = network_manager.client_connected_reciever.try_recv(){
        client_connected_event_writer.send(events::PlayerConnectedEvent(id));
    }
}




///Handles the player connections, messages and channesl
#[derive(Component,Clone)]
pub struct NetworkManager{
    client_connected_reciever: Receiver<SocketAddr>,
    client_connected_sender: Sender<SocketAddr>,
    sender_to_server : Sender<(SocketAddr,OwnedMessage)>,
    receiver_from_client : Receiver<(SocketAddr,OwnedMessage)>,
    sender_to_client : Arc<Mutex<ClientStore<SocketAddr,Sender<OwnedMessage>>>>,
}

impl NetworkManager{
    fn new()->Self{
        let (sender_to_server, receiver_from_client) = unbounded();
        let (client_connected_sender, client_connected_reciever) = unbounded();
        NetworkManager{
            client_connected_sender,
            client_connected_reciever,
            sender_to_server,
            receiver_from_client,
            sender_to_client: Arc::new(Mutex::new(ClientStore::new())),
        }
    }
   
    ///allows to send a message to a specific client
    ///* `message` - the message that is being sent of type Message can be binary or string
    ///* `client_id` - the client's ip address 
    pub fn send_message_to_client(&self, message : Message, client_id : SocketAddr){
        let binding = self.sender_to_client.lock().unwrap();
        let client = binding.get(&client_id);
        match client{
            Some(cli) => {let _ =cli.send(OwnedMessage::from(message).clone());}
            None =>{}
        }
    }
    
    /// allows to send a message to all clients registered in the network manager
    ///* `message` - the message that is being sent of type Message can be binary or string
    pub fn send_message_to_all(&self, message:Message){
        let binding = self.sender_to_client.lock().unwrap();
        for client in binding.iter(){
            let _ = client.send(OwnedMessage::from(message.clone()));
        }
    }

   
    ///Sets up the socket on a seperate thread. It also sets up all the functions for adding any
    ///new connections.
    pub fn setup(&self){
        let network_manager_clone = self.clone(); 
        
        thread::spawn(move ||{
            let server = Server::bind("127.0.0.1:8080").unwrap();
            
            for request in server.filter_map(Result::ok){
                let network_manager_clone = network_manager_clone.clone(); 
                thread::spawn(move ||{
                    let client = request.accept().unwrap();
                    let network_manager = network_manager_clone.clone(); 

                    let (sender_to_client, receiver_from_server) = unbounded::<OwnedMessage>();

                    let ip = client.peer_addr().unwrap();
                    let _ = network_manager_clone.client_connected_sender.send(ip);
                    {
                        let mut clients = network_manager.sender_to_client.lock().unwrap();
                        clients.push(ip,sender_to_client.clone());
                    }

                    let (mut receiver, mut sender) = client.split().unwrap();
                    let sender_to_server = network_manager.sender_to_server.clone();

                    //nested thread spawning for new players connecting
                    thread::spawn(move || {
                        let receiver_from_server = receiver_from_server.clone();
                        loop{
                            let message = receiver_from_server.try_recv();
                            match message{
                                Ok(msg)=>{
                                    match sender.send_message(&msg){
                                        Ok(_)=>{}
                                        Err(err) =>{println!("error while sending message {}", err);}
                                    }
                                }
                                Err(crossbeam_channel::TryRecvError::Empty)=>{
                                    thread::sleep(Duration::from_millis(10));
                                }
                                Err(crossbeam_channel::TryRecvError::Disconnected)=>{
                                    break;
                                }
                            }
                        }
                    });

                   // message sendign logic 
                    for message in receiver.incoming_messages() {
                        let message = message.unwrap();
                        if let OwnedMessage::Close(_closing_data) = message.clone(){
                            let _ = sender_to_server.send((ip,message));
                            thread::sleep(Duration::from_secs(1));
                            let _ = receiver.shutdown();
                            break;
                        }
                        let _ = sender_to_server.send((ip,message));
                    }
                });
            }
        });
    }}
