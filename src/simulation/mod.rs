pub mod communication;

use std::collections::HashMap;
use std::f64::consts::PI;
use std::fmt::{Display, Error, Formatter};
use std::convert::Into;

use tokio::sync::mpsc::{Receiver, Sender};
use hyper::{self, http::uri::InvalidUri, Uri};
use random::{self, Source, Value};
use serde_json;

use self::communication::Message;
use super::brain::communication::Message as BrainMessage;
use super::heartbeat::communication::Message as HeartbeatMessage;
use super::websocket::communication::Message as WsMessage;

#[derive(Default)]
pub struct Simulation {
    team_repository: Teams,
}

pub trait Simulate {
    fn step(&mut self, dt: f64);
}

impl Simulation {
    pub fn new() -> Self {
        Self {
            team_repository: Teams::new(),
        }
    }

    pub async fn start(
        &mut self,
        rx: Receiver<Message>,
        brain_tx: Sender<BrainMessage>,
        heartbeat_tx: Sender<HeartbeatMessage>,
        ws_tx: Sender<WsMessage>,
    ) {
        loop {
            match rx.recv().await {
                Some(message) => match message {
                    Message::Register(registration) => {
                        let attempt = self.team_repository.register(registration);
                        match attempt {
                            Ok(_) => {
                                info!("successfully registered a server")
                            }
                            Err(reason) => {
                                error!("problem registering a server: \"{:?}\"", reason)
                            }
                        }
                    }
                    Message::Unregister(unregistration) => {
                        let attempt = self.team_repository.unregister(unregistration);
                        match attempt {
                            Ok(_) => {
                                info!("successfully unregistered a server")
                            }
                            Err(reason) => {
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

                        if let Err(error) =
                            heartbeat_tx.send(HeartbeatMessage::Check(servers)).await
                        {
                            error!("could not send heartbeat check message: {}", error);
                        }
                    }
                    Message::HeartbeatStatus((name, connected)) => {
                        match self.team_repository.teams.get_mut(&name) {
                            Some(team) => team.set_connection_status(connected),
                            None => {
                                info!("received heartbeat status for {} while unregistered", name)
                            }
                        }
                    }
                    Message::Tick => {
                        self.step(1f64);
                        self.control(brain_tx.clone()).await;
                    }
                    Message::SpawnAll(n) => {
                        info!("spawning {} boids in all connected teams", n);
                        self.team_repository.spawn(n);
                    }
                    Message::Spawn((team_name, n)) => {
                        info!("spawning {} boids in team {}", n, team_name);
                        self.team_repository.spawn_in_team(team_name, n);
                    }
                    Message::BrainUpdate(team_name) => {
                        info!("processing brain update for {}", team_name);
                    }
                },

                None => {
                    error!("could not receive message");
                }
            }

            if let Ok(json) = serde_json::to_string(&self.team_repository) {
                if let Err(error) = ws_tx.send(WsMessage::Update(json)).await {
                    error!("could not send update message: {}", error);
                }
            } else {
                error!("could not serialize team_repository");
            }
        }
    }

    async fn control(&self, tx: Sender<BrainMessage>) {
        let servers: Vec<(String, Uri, String)> = self
            .team_repository
            .teams
            .iter()
            .filter(|(_, team)| team.connected)
            .filter(|(_, team)| !team.flock.is_empty())
            .map(|(name, team)| (name, team.brain_uri(), team.brain_payload()))
            .filter(|(_, uri, _)| uri.is_ok())
            .filter(|(_, _, payload)| payload.is_ok())
            .map(|(name, uri, payload)| {
                (
                    name.clone(),
                    uri.unwrap(/* safe because is_ok check*/),
                    payload.unwrap(/* safe because is_ok check */),
                )
            })
            .collect();
        if tx.send(BrainMessage::Pick(servers)).await.is_err() {
            error!("could not pick brain");
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

pub trait TeamRepository {
    fn register(&mut self, registration: Registration) -> Result<(), RegistrationFailureReason>;
    fn unregister(&mut self, unregistration: Unregistration) -> Result<(), UnregistrationFailureReason>;
}

impl TeamRepository for Teams {
    fn register(&mut self, registration: Registration) -> Result<(), RegistrationFailureReason> {
        if self.teams.contains_key(&registration.name) {
            return Err(RegistrationFailureReason::NameTaken);
        }

        if !self.available(&registration.ip_address, registration.port) {
            return Err(RegistrationFailureReason::IPAddressWithPortTaken);
        }

        self.teams
            .insert(registration.name.clone(), registration.into());
        Ok(())
    }

    fn unregister(&mut self, unregistration: Unregistration) -> Result<(), UnregistrationFailureReason> {
        if !self.teams.contains_key(&unregistration.name) {
            return Err(UnregistrationFailureReason::NameNotRegistered);
        }

        self.teams.remove(&unregistration.name);
        Ok(())
    }
}

#[derive(PartialEq, Debug)]
pub enum RegistrationFailureReason {
    NameTaken,
    IPAddressWithPortTaken,
}

impl Into<String> for RegistrationFailureReason {
    fn into(self) -> String {
        (match self {
            RegistrationFailureReason::NameTaken => "name already taken",

            RegistrationFailureReason::IPAddressWithPortTaken => {
                "ip address with port already taken"
            }
        })
        .to_string()
    }
}

#[derive(Deserialize, Debug)]
pub struct Registration {
    name: String,
    ip_address: String,
    port: u16,
}

impl Into<Team> for Registration {
    fn into(self) -> Team {
        Team::new(self.name, self.ip_address, self.port)
    }
}

#[derive(Serialize, Debug)]
pub struct RegistrationFailure {
    reason: String,
}

impl RegistrationFailure {
    pub fn new<S>(reason: S) -> RegistrationFailure
    where
        S: Into<String>,
    {
        RegistrationFailure {
            reason: reason.into(),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum UnregistrationFailureReason {
    NameNotRegistered,
}

#[derive(Deserialize, Debug)]
pub struct Unregistration {
    name: String,
}

#[derive(Serialize, Debug)]
pub struct UnregistrationFailure {
    reason: String,
}

impl UnregistrationFailure {
    pub fn new<S>(reason: S) -> UnregistrationFailure
    where
        S: Into<String>,
    {
        UnregistrationFailure {
            reason: reason.into(),
        }
    }
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

    pub fn spawn_in_team(&mut self, name: String, n: usize) {
        self.teams
            .get_mut(&name)
            .iter_mut()
            .for_each(|team| team.spawn(n))
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

    pub fn heartbeat_uri(&self) -> Result<Uri, InvalidUri> {
        let address = format!("{}://{}:{}/heartbeat", "http", self.ip_address, self.port);

        address.parse()
    }

    pub fn brain_uri(&self) -> Result<Uri, InvalidUri> {
        let address = format!("{}://{}:{}/brain", "http", self.ip_address, self.port);

        address.parse()
    }

    pub fn brain_payload(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.flock)
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

    pub fn is_empty(&self) -> bool {
        self.boids.is_empty()
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

#[derive(Deserialize, Serialize, Hash, PartialEq, Eq)]
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

        while self.x < 0f64 {
            self.x += 1f64
        }
        while self.x > 1f64 {
            self.x -= 1f64
        }
        while self.y < 0f64 {
            self.y += 1f64
        }
        while self.y > 1f64 {
            self.y -= 1f64
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_should_be_registered_with_a_registration() {
        let mut teams = Teams::new();
        let registration = Registration {
            name: "TEST".to_owned(),
            ip_address: "TEST ADDRESS".to_owned(),
            port: 2643,
        };

        let result = teams.register(registration);

        assert!(result.is_ok());
    }

    #[test]
    fn team_with_same_name_should_not_be_registered() {
        let mut teams = Teams::new();
        let first = Registration {
            name: "TEST".to_owned(),
            ip_address: "TEST ADDRESS".to_owned(),
            port: 2643,
        };
        let _ = teams.register(first);

        let second = Registration {
            name: "TEST".to_owned(),
            ip_address: "OTHER TEST ADDRESS".to_owned(),
            port: 2643,
        };
        let result = teams.register(second);

        assert_eq!(
            result,
            Err(RegistrationFailureReason::NameTaken)
        );
    }

    #[test]
    fn team_with_same_ip_address_should_not_be_registered() {
        let mut teams = Teams::new();
        let first = Registration {
            name: "TEST".to_owned(),
            ip_address: "TEST ADDRESS".to_owned(),
            port: 2643,
        };
        let _ = teams.register(first);

        let second = Registration {
            name: "OTHER TEST".to_owned(),
            ip_address: "TEST ADDRESS".to_owned(),
            port: 2643,
        };
        let result = teams.register(second);

        assert_eq!(
            result,
            Err(RegistrationFailureReason::IPAddressWithPortTaken)
        );
    }
}
