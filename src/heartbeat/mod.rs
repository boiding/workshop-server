pub mod communication;

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use futures::Future;
use hyper::{Client, Method, Request};
use tokio_core::reactor::Core;

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

    pub fn monitor(&mut self) {
        let mut core = Core::new().unwrap(); // TODO handle error?
        let client = Client::new(&core.handle());

        let team_rx_mutex = Arc::new(Mutex::new(self.tx.clone()));
        loop {
            thread::sleep(self.sleep_duration);
            if let Err(error) = self.tx.send(TeamsMessage::Heartbeat) {
                error!("could not send heartbeat: {:?}", error);
            } else if let Ok(message) = self.rx.recv() {
                match message {
                    HeartbeatMessage::Check(servers) => {
                        for (team_name, uri) in servers {
                            info!("heartbeat for {} at {}", team_name, uri);
                            let (success_team_rx_mutex, success_team_name) =
                                (team_rx_mutex.clone(), team_name.clone());
                            let (failure_team_rx_mutex, failure_team_name) =
                                (team_rx_mutex.clone(), team_name.clone());
                            let request = Request::new(Method::Head, uri);
                            let work = client
                                .request(request)
                                .map(move |response| {
                                    info!("{} {}", success_team_name, response.status());
                                    success_team_rx_mutex
                                        .lock()
                                        .unwrap()
                                        .send(TeamsMessage::HeartbeatStatus((
                                            success_team_name,
                                            true,
                                        )))
                                        .unwrap();
                                })
                                .map_err(move |_| {
                                    error!("{} disconnected", failure_team_name);
                                    failure_team_rx_mutex
                                        .lock()
                                        .unwrap()
                                        .send(TeamsMessage::HeartbeatStatus((
                                            failure_team_name,
                                            false,
                                        )))
                                        .unwrap();
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
