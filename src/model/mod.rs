pub mod communication;

use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::sync::mpsc::{Receiver, Sender};

use hyper::{self, Uri};
use serde_json;

use self::communication::Message;
use super::register::model::{TeamRepository, RegistrationAttempt, UnregistrationAttempt};
use super::heartbeat::communication::Message as HeartbeatMessage;
use super::websocket::communication::Message as WsMessage;

pub struct Simulation {
    team_repository: Teams,
}

impl Simulation {
    pub fn new() -> Simulation {
        Simulation { team_repository: Teams::new() }
    }

    pub fn start(
        &mut self,
        rx: Receiver<Message>,
        heartbeat_tx: Sender<HeartbeatMessage>,
        ws_tx: Sender<WsMessage>,
    ) {
        loop {
            match rx.recv() {
                Ok(message) => {
                    match message {
                        Message::Register(registration) => {
                            let attempt = self.team_repository.register(registration);
                            match attempt {
                                RegistrationAttempt::Success => {
                                    info!("successfully registered a server")
                                }
                                RegistrationAttempt::Failure(reason) => {
                                    error!("problem registering a server: \"{:?}\"", reason)
                                }
                            }
                        }
                        Message::Unregister(unregistration) => {
                            let attempt = self.team_repository.unregister(unregistration);
                            match attempt {
                                UnregistrationAttempt::Success => {
                                    info!("successfully unregistered a server")
                                }
                                UnregistrationAttempt::Failure(reason) => {
                                    error!("problem unregistering a server: \"{:?}\"", reason)
                                }
                            }
                        }
                        Message::Heartbeat => {
                            let servers = self.team_repository
                                .teams
                                .iter()
                                .map(|(name, team)| (name.clone(), team.heartbeat_uri().unwrap()))
                                .collect();

                            if let Err(error) = heartbeat_tx.send(HeartbeatMessage::Check(servers)) {
                                error!("could not send heartbeat check message: {}", error);
                            }
                        }
                        Message::HeartbeatStatus((name, connected)) => {
                            match self.team_repository.teams.get_mut(&name) {
                                Some(team) => team.set_connection_status(connected),
                                None => {
                                    info!(
                                        "received heartbeat status for {} while unregistered",
                                        name
                                    )
                                }
                            }
                        }
                    }
                }

                Err(error) => {
                    error!("could not receive message: {}", error);
                }
            }


            if let Ok(json) = serde_json::to_string(&self.team_repository) {
                if let Err(error) = ws_tx.send(WsMessage::Update(json)) {
                    error!("could not send update message: {}", error);
                }
            } else {
                error!("could not serialize team_repository");
            }
        }
    }
}

#[derive(Serialize)]
pub struct Teams {
    pub teams: HashMap<String, Team>,
}

impl Teams {
    pub fn new() -> Teams {
        Teams { teams: HashMap::new() }
    }

    pub fn available(&self, ip_address: &str, port: u16) -> bool {
        self.teams
            .iter()
            .filter(|&(_name, team)| {
                team.ip_address == ip_address && team.port == port
            })
            .count() == 0
    }
}

#[derive(Serialize)]
pub struct Team {
    name: String,
    ip_address: String,
    port: u16,
    connected: bool,
}

impl Team {
    pub fn new<S>(name: S, ip_address: S, port: u16) -> Team
    where
        S: Into<String>,
    {
        Team {
            name: name.into(),
            ip_address: ip_address.into(),
            port,
            connected: false,
        }
    }

    pub fn heartbeat_uri(&self) -> Result<Uri, hyper::error::UriError> {
        let address = format!("{}://{}:{}/heartbeat", "http", self.ip_address, self.port);

        address.parse()
    }

    pub fn set_connection_status(&mut self, connected: bool) {
        self.connected = connected
    }
}

impl Display for Team {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} {}", self.name, self.ip_address)
    }
}
