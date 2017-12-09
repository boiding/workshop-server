pub mod communication;

use std::sync::mpsc::{Receiver};
use std::thread;

use self::communication::Message as WsMessage;
use ws::{self, WebSocket, Message};

pub struct WebSocketUpdate {
    socket_address: String
}

impl WebSocketUpdate {
    pub fn new<S>(socket_address: S) -> WebSocketUpdate where S: Into<String> {
        WebSocketUpdate { socket_address: socket_address.into() }
    }

    pub fn dispatch(&self, rx: Receiver<WsMessage>) {
        if let Ok(web_socket) = WebSocket::new(|out: ws::Sender| {
            move |msg: Message| {
                info!("Server got message '{}'. ", msg);
                out.broadcast("")
            }
        }) {
            let sender = web_socket.broadcaster();
            let send_thread = thread::spawn(move||{
                let message = rx.recv().unwrap();
                match message {
                    WsMessage::Update(payload) => {
                        sender.send(payload).unwrap();
                    }
                }
            });
            if let Err(error) = web_socket.listen(&self.socket_address) {
                error!("Websocket could not listen {:?}", error);
            }
            send_thread.join().unwrap();
        } else {
            error!("Failed to create WebSocket");
        }
    }
}
