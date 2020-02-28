pub mod communication;

use std::collections::HashMap;
use std::f64::consts::PI;
use std::fmt::{Display, Error, Formatter};
use std::sync::mpsc::{Receiver, Sender};

use hyper::{self, Uri};
use random::{self, Source, Value};
use serde_json;

use self::communication::Message;
use super::heartbeat::communication::Message as HeartbeatMessage;
use super::register::model::{RegistrationAttempt, TeamRepository, UnregistrationAttempt};
use super::websocket::communication::Message as WsMessage;

#[derive(Default)]
pub struct Simulation {
    team_repository: Teams,
}

pub trait Simulate {
    fn step(&mut self, dt: f64);
}

impl Simulation {
    pub fn new() -> Simulation {
        Simulation {
            team_repository: Teams::new(),
        }
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
                            let servers = self
                                .team_repository
                                .teams
                                .iter()
                                .map(|(name, team)| (name.clone(), team.heartbeat_uri().unwrap()))
                                .collect();

                            if let Err(error) = heartbeat_tx.send(HeartbeatMessage::Check(servers))
                            {
                                error!("could not send heartbeat check message: {}", error);
                            }
                        }
                        Message::HeartbeatStatus((name, connected)) => {
                            match self.team_repository.teams.get_mut(&name) {
                                Some(team) => team.set_connection_status(connected),
                                None => info!(
                                    "received heartbeat status for {} while unregistered",
                                    name
                                ),
                            }
                        }
                        Message::Tick => { /* Process tick */ }
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

impl Simulate for Simulation {
    fn step(&mut self, dt: f64) {
        self.team_repository.step(dt);
    }
}

pub trait Spawn {
    fn spawn(&mut self, n: usize);
}

#[derive(Serialize, Default)]
pub struct Teams {
    pub teams: HashMap<String, Team>,
}

impl Teams {
    pub fn new() -> Teams {
        Teams {
            teams: HashMap::new(),
        }
    }

    pub fn available(&self, ip_address: &str, port: u16) -> bool {
        self.teams
            .iter()
            .filter(|&(_name, team)| team.ip_address == ip_address && team.port == port)
            .count()
            == 0
    }
}

impl Simulate for Teams {
    fn step(&mut self, dt: f64) {
        self.teams.iter_mut().for_each(|(_, team)| team.step(dt))
    }
}

impl Spawn for Teams {
    fn spawn(&mut self, n: usize) {
        self.teams.iter_mut().for_each(|(_, team)| team.spawn(n))
    }
}

#[derive(Serialize)]
pub struct Team {
    name: String,
    ip_address: String,
    port: u16,
    connected: bool,
    flock: Flock,
}

impl Team {
    pub fn new<S>(name: S, ip_address: S, port: u16) -> Team
    where
        S: Into<String>,
    {
        let flock = Flock::new();
        Team {
            name: name.into(),
            ip_address: ip_address.into(),
            port,
            connected: false,
            flock,
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

impl Simulate for Team {
    fn step(&mut self, dt: f64) {
        self.flock.step(dt);
    }
}

impl Spawn for Team {
    fn spawn(&mut self, n: usize) {
        self.flock.spawn(n)
    }
}

impl Display for Team {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} {}", self.name, self.ip_address)
    }
}

#[derive(Serialize, Default)]
pub struct Flock {
    pub boids: HashMap<FlockId, Boid>,
}

impl Flock {
    pub fn new() -> Flock {
        let boids = HashMap::new();
        Flock { boids }
    }
}

impl Simulate for Flock {
    fn step(&mut self, dt: f64) {
        self.boids.iter_mut().for_each(|(_, boid)| boid.step(dt))
    }
}

impl Spawn for Flock {
    fn spawn(&mut self, n: usize) {
        let mut source = random::default();
        let old_size = self.boids.len();
        while (self.boids.len() - old_size) < n {
            let identifier = source.read::<FlockId>();
            let boid = source.read::<Boid>();
            self.boids.insert(identifier, boid);
        }
    }
}

#[derive(Serialize, Hash, PartialEq, Eq)]
pub struct FlockId(u64);

impl From<u64> for FlockId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl Value for FlockId {
    fn read<S>(source: &mut S) -> Self
    where
        S: Source,
    {
        let id = source.read_u64();

        Self::from(id)
    }
}

#[derive(Serialize)]
pub struct Boid {
    x: f64,
    y: f64,
    heading: f64,
    speed: f64,
}

impl Boid {
    fn new(x: f64, y: f64, heading: f64, speed: f64) -> Self {
        Self {
            x,
            y,
            heading,
            speed,
        }
    }
}

impl Simulate for Boid {
    fn step(&mut self, dt: f64) {
        let d = self.speed * dt;
        let dx = d * self.heading.cos();
        let dy = d * self.heading.sin();

        self.x += dx;
        self.y += dy;
    }
}

impl Value for Boid {
    fn read<S>(source: &mut S) -> Self
    where
        S: Source,
    {
        let x = source.read_f64();
        let y = source.read_f64();
        let heading = 2f64 * PI * (source.read_f64() - 0.5);
        let speed = 0.01 * source.read_f64(); // TOOD determine maximum speed
        Self::new(x, y, heading, speed)
    }
}
