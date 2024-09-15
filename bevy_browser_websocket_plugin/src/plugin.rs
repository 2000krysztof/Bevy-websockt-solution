use bevy::prelude::*;
use wasm_bindgen::prelude::*; 
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use std::sync::{Arc, Mutex};
use super::ClientMessageEvent;
use super::Message;


/// Exposes the browser console log to rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

/// A macro to be able to print any type int the console
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}


///The network manager which will be instantiated in order to establish a connection
///It also handles the message events and gives the ability to send a message
#[wasm_bindgen]
#[derive(Component)]
pub struct NetworkManager{
   socket:Option<WebSocket>,

   //the reason why this is a message buffer and not a channels is because this is working with
   //WASM where channels behave differently due to the lack of true concurency
   message_buffer : Arc<Mutex<Vec<Message>>>
}


//This is here to satisfy the Send and Sync requirecments for the network manager to safely pass it
//between threads. Althoe in WASM this will run on a single thread this must still be here to
//satisfy the borrow checker
unsafe impl Send for NetworkManager{}
unsafe impl Sync for NetworkManager{}

/// Spawns the network manager and handles the messsage evenet loop.
/// It also registers the ClientMessageEvent
pub struct WebSocketPlugin;


impl Plugin for WebSocketPlugin{
    fn build(&self, app: &mut App) {
       app.add_event::<ClientMessageEvent>().
           add_systems(Startup, spawn_network_manager).
           add_systems(Update, emit_messages_as_events);
    }
}
//Spawns the network manager in the game world
fn spawn_network_manager(mut commands:  Commands){
       commands.spawn(NetworkManager::new());
}

//Accepts the raw messages from the network manager and converts it's data to a Client Message
//Event which is then re emitted to the game
fn emit_messages_as_events(mut query_network_manager:Query<&mut NetworkManager>,mut event_writer: EventWriter<ClientMessageEvent>){
    let network_manager = query_network_manager.single_mut();
    for message in network_manager.message_buffer.lock().unwrap().iter(){
        event_writer.send(ClientMessageEvent(message.clone().to_owned()));
    }
    network_manager.message_buffer.lock().unwrap().clear();
}


#[wasm_bindgen] 
impl NetworkManager{

    fn new() -> NetworkManager{
        let mut netowork_manager = NetworkManager{socket:None, message_buffer:Arc::new(Mutex::new(Vec::new()))};
        let _ = netowork_manager.start_websocket("ws://localhost:8080");
        netowork_manager
    }

    ///Starts a websocket and defines the methods for handeling incoming messages
    #[wasm_bindgen]
    pub fn start_websocket(&mut self, url : &str) -> Result<(), JsValue> {
        let ws: WebSocket = WebSocket::new(url)?;
        self.socket = Some(ws.clone());
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        let messages_ref = Arc::clone(&self.message_buffer);

        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {

            let messages_ref = messages_ref.clone();
            
            //abuf

            if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                let mut messages = messages_ref.lock().unwrap();
                let array = js_sys::Uint8Array::new(&abuf);
                let vec = array.to_vec();
                let message = Message::Binary(vec);
            
                messages.push(message); 

            //blob

            } else if let Ok(blob) = e.data().dyn_into::<web_sys::Blob>() {
                let message_ref = messages_ref.clone();  
                let fr = web_sys::FileReader::new().unwrap();
                let fr_c = fr.clone();
                let onloadend_cb = Closure::<dyn FnMut(_)>::new(move |_e: web_sys::ProgressEvent| {
                    let mut messages = message_ref.lock().unwrap();
                    let array = js_sys::Uint8Array::new(&fr_c.result().unwrap());
                    let vec = array.to_vec();
                    let message = Message::Binary(vec);
                    messages.push(message); 
                });
                fr.set_onloadend(Some(onloadend_cb.as_ref().unchecked_ref()));
                fr.read_as_array_buffer(&blob).expect("blob not readable");
                onloadend_cb.forget();

            //text

            } else if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let mut messages = messages_ref.lock().unwrap();
                match txt.as_string(){
                    Some(text) =>{messages.push(Message::Text(text));}
                    None =>{}
                }
            //Other

            } else {
                console_log!("message event, received Unknown: {:?}", e.data());
            }

        });
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
            console_log!("error event: {:?}", e);
        });

        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        let cloned_ws = ws.clone();
        let onopen_callback = Closure::<dyn FnMut()>::new(move || {
            console_log!("socket opened");
            match cloned_ws.send_with_str("ping") {
                Ok(_) => console_log!("message successfully sent"),
                Err(err) => console_log!("error sending message: {:?}", err),
            }
            match cloned_ws.send_with_u8_array(&[0, 1, 2, 3]) {
                Ok(_) => console_log!("binary message successfully sent"),
                Err(err) => console_log!("error sending message: {:?}", err),
            }
        });
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
        
        Ok(())
    }

    ///Sends bindary message to the host. The message itself is a Vec<u8>
    #[wasm_bindgen]
    pub fn send_message_abuf(&mut self, message:Vec<u8>){
        match &self.socket{
            Some(socket)=>{
                let result = socket.send_with_u8_array(&message);
                match result{
                    Ok(_) =>{console_log!("test message sent successfully")}
                    Err(_err) =>{console_log!("error while sending test message ")}
                }

            },
            None=>{console_log!("socket not initialized")}
        }
    }

    ///Sends string messages to the host. The message istself is an &str
    #[wasm_bindgen]
    pub fn send_message_txt(&mut self, message:&str){
        match &self.socket{
            Some(socket)=>{
                let result = socket.send_with_str(message);
                match result{
                    Ok(_) =>{console_log!("test message sent successfully")}
                    Err(_err) =>{console_log!("error while sending test message ")}
                }

            },
            None=>{console_log!("socket not initialized")}
        }
    }
    
}

impl NetworkManager{
    /// DO NOT USE !!!!!
    /// Use EvenetReader<ClientMessageEvent> instead.
    /// Allows for the access of the raw message buffer to read the messages.
    /// Made for use only by another internal module.
    pub fn get_message_buffer(&self) -> Vec<Message>{
        self.message_buffer.lock().unwrap().to_vec().clone()
    }
}
