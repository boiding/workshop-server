pub mod communication;

use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

use hyper::{Body, Client, Method, Request};

use self::communication::Message as HeartbeatMessage;
use super::simulation::communication::Message as TeamsMessage;

pub struct Heartbeat {
    sleep_duration: Duration,
    rx: Receiver<HeartbeatMessage>,
    tx: Sender<TeamsMessage>,
}

impl Heartbeat {
    pub fn new(
        sleep_duration: Duration,
        rx: Receiver<HeartbeatMessage>,
        tx: Sender<TeamsMessage>,
    ) -> Self {
        Self {
            sleep_duration,
            rx,
            tx,
        }
    }

    pub async fn monitor(&mut self) {
        let client = Client::new();

        loop {
            thread::sleep(self.sleep_duration);
            if let Err(error) = self.tx.send(TeamsMessage::Heartbeat) {
                error!("could not send heartbeat: {:?}", error);
            } else if let Ok(message) = self.rx.recv() {
                match message {
                    HeartbeatMessage::Check(servers) => {
                        for (team_name, uri) in servers {
                            info!("heartbeat for {} at {}", team_name, uri);
                            let request = Request::builder()
                                .method(Method::HEAD)
                                .uri(uri)
                                .body(Body::empty())
                                .unwrap();
                            if let Ok(response) = client.request(request).await {
                                info!("{} {}", team_name, response.status());
                                if self
                                    .tx
                                    .send(TeamsMessage::HeartbeatStatus((team_name, true)))
                                    .is_err()
                                {
                                    error!("recieved heartbeat but could not notify simulation");
                                }
                            } else {
                                error!("{} disconnected", team_name);
                                if self
                                    .tx
                                    .send(TeamsMessage::HeartbeatStatus((team_name, false)))
                                    .is_err()
                                {
                                    error!(
                                        "recieved disconnection but could not notify simulation"
                                    );
                                }
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
