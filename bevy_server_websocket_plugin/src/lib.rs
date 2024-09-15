pub mod plugin;
pub mod events;
mod client_list;
use websocket::OwnedMessage;

///Message type which will be useable troughout bevy engine's event system
#[derive(Clone)]
pub enum Message{
    ///The string data that can be sent across the web like string or json
    Text(String),
    ///The bindary data that can be sent across the web in the form of Vec<u8>
    Binary(Vec<u8>)
}


/// by default OwnedMessage is the data type that is used troughout rust's websockets but 
/// my solution uses Message for internal event handeling so this is a convenience method to change
/// the owned message to Message
impl From<OwnedMessage> for Message{
    fn from(message: OwnedMessage)->Self{
        match message{
           OwnedMessage::Text(txt) => Message::Text(txt),
            OwnedMessage::Binary(bin) => Message::Binary(bin),
            _ => Message::Text("unsuported type".into())
        }

    }
}

/// used to convert Message to OwnedMessage which can be used with rust's websocket
impl From<Message> for OwnedMessage{
     fn from(message: Message)->OwnedMessage{
        match message{
            Message::Text(txt)=>OwnedMessage::Text(txt),
            Message::Binary(bin)=>OwnedMessage::Binary(bin)
        }
    }

}

