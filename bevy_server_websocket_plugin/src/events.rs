use bevy::prelude::*;
use std::net::SocketAddr;

/// Fires when a new client connects
/// It also holds the value of the socket address of what the player connected
#[derive(Event)]
pub struct PlayerConnectedEvent(pub SocketAddr);

/// Fiers when a client disconects
/// It also holds the value of the socket address of what player disconnected 
#[derive(Event)]
pub struct PlayerDisconectedEvent(pub SocketAddr);


/// Fires when a client sends a message
/// It holds the value of the message and the socket address of the player who sent it
#[derive(Event)]
pub struct ServerMessageEvent(pub SocketAddr,pub super::Message);
