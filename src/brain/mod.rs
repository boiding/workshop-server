pub mod communication;
pub mod model;

use std::sync::mpsc::{Receiver, Sender};

use futures::{Future, stream::Stream};
use hyper::{header::ContentType, Client, Method, Request};
use tokio_core::reactor::Core;

use self::communication::Message as BrainMessage;
use crate::simulation::communication::Message as TeamsMessage;

pub struct Brain {
    rx: Receiver<BrainMessage>,
    tx: Sender<TeamsMessage>,
}

impl Brain {
    pub fn new(rx: Receiver<BrainMessage>, tx: Sender<TeamsMessage>) -> Self {
        Self { rx, tx }
    }

    pub fn think(&mut self) {
        let mut core = Core::new().unwrap(); // TODO handle error
        let client = Client::new(&core.handle());

        loop {
            if let Ok(message) = self.rx.recv() {
                match message {
                    BrainMessage::Pick(servers) => {
                        for (team_name, uri, payload) in servers {
                            info!("picking brain of {} at {}", team_name, uri);
                            let (team_tx, success_team_name) = (self.tx.clone(), team_name.clone());
                            let mut request = Request::new(Method::Post, uri);
                            request.headers_mut().set(ContentType::json());
                            request.set_body(payload);
                            let work = client
                                .request(request)
                                .and_then(|response|{
                                    response.body().concat2()
                                })
                                .map(|chunk| String::from_utf8(chunk.to_vec()).unwrap(/* TODO: error handling */))
                                .map(|source| {
                                    //serde_json::from_str(&source).unwrap(/* TODO: error handling */);
                                    info!("intent: {}", source);
                                    source
                                })
                                .map(move |_intent| {
                                    info!("picked brain of {}", success_team_name);
                                    team_tx
                                        .send(TeamsMessage::BrainUpdate(success_team_name))
                                        .unwrap();
                                })
                                .map_err(move |error| {
                                    error!(
                                        "did not receive brain update from {}: {}",
                                        team_name, error
                                    );
                                });

                            match core.run(work) {
                                _ => (), /* Everything is fine */
                            }
                        }
                    }
                }
            } else {
                error!("could not receive message")
            }
        }
    }
}
