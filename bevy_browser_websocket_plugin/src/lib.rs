pub mod plugin;
use bevy::prelude::*;

/// The message that the event can handle
#[derive(Clone)]
pub enum Message{
    ///Text data like string or json
    Text(String),
    ///Binary data for less overhaed
    Binary(Vec<u8>)
}

/// The message event that will be output by the network manager when a message is received
#[derive(Event)]
pub struct ClientMessageEvent(pub Message);
