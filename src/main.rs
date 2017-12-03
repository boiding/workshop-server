extern crate bws;
extern crate dotenv;
extern crate iron;
#[macro_use] extern crate log;
extern crate logger;
extern crate mount;
extern crate router;
extern crate simplelog;

use std::env;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

use dotenv::dotenv;
use iron::Iron;
use simplelog::{Config, LogLevelFilter, TermLogger, CombinedLogger};

use bws::heartbeat::{Heartbeat, HeartbeatMessage};
use bws::model::Teams;
use bws::model::communication::Message;
use bws::register::model::{TeamRepository, RegistrationAttempt, UnregistrationAttempt};
use bws::server;

fn main() {
    dotenv().ok();
    CombinedLogger::init(
        vec![
            TermLogger::new(LogLevelFilter::Info, Config::default()).unwrap(),
        ]
    ).unwrap();

    info!("Logger configured");

    let (team_tx, team_rx): (Sender<Message>, Receiver<Message>) = channel();
    let (heartbeat_tx, heartbeat_rx): (Sender<HeartbeatMessage>, Receiver<HeartbeatMessage>) = channel();
    let team_heartbeat_tx = heartbeat_tx.clone();
    let teams_thread = thread::spawn(move ||{
        info!("teams thread started");
        let mut team_repository = Teams::new();
        loop {
            let message = team_rx.recv().unwrap();
            match message {
                Message::Register(registration) => {
                    let attempt = team_repository.register(registration);
                    match attempt {
                        RegistrationAttempt::Success => info!("successfully registered a server"),
                        RegistrationAttempt::Failure(reason) => error!("problem registering a server: \"{:?}\"", reason),
                    }
                },
                Message::Unregister(unregistration) => {
                    let attempt = team_repository.unregister(unregistration);
                    match attempt {
                        UnregistrationAttempt::Success => info!("successfully unregistered a server"),
                        UnregistrationAttempt::Failure(reason) => error!("problem unregistering a server: \"{:?}\"", reason),
                    }
                },
                Message::Heartbeat => {
                    let servers = team_repository.teams.iter()
                        .map(|(name, team)|{ (name.clone(), team.heartbeat_uri().unwrap()) })
                        .collect();

                    team_heartbeat_tx.send(HeartbeatMessage::Check(servers)).unwrap();
                },
                Message::HeartbeatStatus((name, connected)) => {
                    info!("received heartbeat status for {}: connected {}", name, connected);
                },
            }
        }
    });

    let iron_team_tx = team_tx.clone();
    let iron_thread = thread::spawn(move ||{
        let server_address = env::var("address").expect("\"address\" in environment variables");
        info!("starting server at {}", server_address);

        Iron::new(server::chain(&iron_team_tx)).http(server_address).unwrap();
    });

    let heartbeat_team_tx = team_tx.clone();
    let heartbeat_thread = thread::spawn(move ||{
        let mut heartbeat = Heartbeat::new(heartbeat_rx, heartbeat_team_tx);
        info!("starting heartbeat monitor");

        heartbeat.monitor();
    });

    iron_thread.join().unwrap();
    heartbeat_thread.join().unwrap();
    teams_thread.join().unwrap();
}
