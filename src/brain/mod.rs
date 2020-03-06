pub mod communication;
pub mod model;

use tokio::sync::mpsc::{Receiver, Sender};
use hyper::{
    header::{HeaderName, CONTENT_TYPE},
    Body, Client, Method, Request,
};

use self::communication::Message as BrainMessage;
use super::simulation::communication::Message as TeamsMessage;

pub struct Brain {
    rx: Receiver<BrainMessage>,
    tx: Sender<TeamsMessage>,
}

impl Brain {
    pub fn new(rx: Receiver<BrainMessage>, tx: Sender<TeamsMessage>) -> Self {
        Self { rx, tx }
    }

    pub async fn think(&mut self) {
        let client = Client::new();

        loop {
            if let Some(message) = self.rx.recv().await {
                match message {
                    BrainMessage::Pick(servers) => {
                        for (team_name, uri, payload) in servers {
                            info!("picking brain of {} at {}", team_name, uri);
                            let request = Request::builder()
                                .method(Method::POST)
                                .header(CONTENT_TYPE, HeaderName::from_static("application/json"))
                                .uri(uri)
                                .body(Body::from(payload))
                                .unwrap();
                        //     if let Ok(_response) = client.request(request).into_future().await {
                        //         info!("picked brain of {}", team_name);
                        //         if self
                        //             .tx
                        //             .send(TeamsMessage::BrainUpdate(team_name))
                        //             .await
                        //             .is_err()
                        //         {
                        //             error!("could not send brain update for {}", team_name);
                        //         }
                        //     } else {
                        //         error!("did not receive brain update from {}", team_name);
                        //     }
                        }
                    }
                }
            } else {
                error!("could not receive message")
            }
        }
    }
}
