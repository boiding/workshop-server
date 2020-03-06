pub mod communication;

use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
};

use serde_json::{self};
use ws::{self, Message, WebSocket};

use self::communication::Message as WsMessage;
use crate::simulation::communication::Message as TeamsMessage;

pub struct WebSocketUpdate {
    socket_address: String,
}

impl WebSocketUpdate {
    pub fn new<S>(socket_address: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            socket_address: socket_address.into(),
        }
    }

    pub fn dispatch(&self, tx: Sender<TeamsMessage>, rx: Receiver<WsMessage>) {
        if let Ok(web_socket) = WebSocket::new(|out: ws::Sender| {
            let simulation_tx = tx.clone();
            move |msg: Message| {
                info!("Server got message '{}'. ", msg);
                if let Ok(command_text) = msg.as_text() {
                    if let Ok(command) = serde_json::from_str::<Command>(command_text) {
                        match command {
                            Command::Spawn { team } => {
                                if simulation_tx.send(TeamsMessage::Spawn((team, 5))).is_err() {
                                    error!("could not send a spawn message");
                                }
                            }
                        }
                    } else {
                        error!("could not serialize {}", msg);
                    }
                } else {
                    error!("could not read '{}' as text", msg)
                }
                out.ping(vec![])
            }
        }) {
            let sender = web_socket.broadcaster();
            let send_thread = thread::Builder::new()
                .name("repeater".to_string())
                .spawn(move || loop {
                    match rx.recv() {
                        Ok(message) => match message {
                            WsMessage::Update(payload) => {
                                sender.send(payload).unwrap();
                            }
                        },

                        Err(error) => {
                            error!("could not receive message: {}", error);
                        }
                    }
                })
                .unwrap();
            if let Err(error) = web_socket.listen(&self.socket_address) {
                error!("Websocket could not listen {:?}", error);
            }
            send_thread.join().unwrap();
        } else {
            error!("Failed to create WebSocket");
        }
    }
}

#[derive(Deserialize, Debug)]
pub enum Command {
    Spawn { team: String },
}
