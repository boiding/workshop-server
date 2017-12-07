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

use bws::heartbeat::Heartbeat;
use bws::heartbeat::communication::Message as HeartbeatMessage;
use bws::model::Simulation;
use bws::model::communication::Message as TeamsMessage;
use bws::server;

fn main() {
    dotenv().ok();
    CombinedLogger::init(
        vec![
            TermLogger::new(LogLevelFilter::Info, Config::default()).unwrap(),
        ]
    ).unwrap();

    info!("Logger configured");

    let (team_tx, team_rx): (Sender<TeamsMessage>, Receiver<TeamsMessage>) = channel();
    let (heartbeat_tx, heartbeat_rx): (Sender<HeartbeatMessage>, Receiver<HeartbeatMessage>) = channel();
    let team_heartbeat_tx = heartbeat_tx.clone();
    let teams_thread = thread::spawn(move ||{
        info!("simulation thread started");
        let mut simulation = Simulation::new();

        simulation.start(team_rx, team_heartbeat_tx);
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
