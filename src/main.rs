extern crate bws;
extern crate dotenv;
extern crate iron;
#[macro_use] extern crate log;
extern crate logger;
extern crate mount;
extern crate router;
extern crate simplelog;

use std::env;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

use dotenv::dotenv;
use iron::Iron;
use simplelog::{Config, LogLevelFilter, TermLogger, CombinedLogger};

use bws::heartbeat::Heartbeat;
use bws::model::Teams;
use bws::server;

fn main() {
    dotenv().ok();
    CombinedLogger::init(
        vec![
            TermLogger::new(LogLevelFilter::Info, Config::default()).unwrap(),
        ]
    ).unwrap();

    info!("Logger configured");

    let (tx, rx): (Sender<Message>, Receiver<Message>) = channel();
    let teams_thread = thread::spawn(||{
        info!("teams thread started");
        let team_repository = Teams::new();
    });

    let team_repository_ref = Arc::new(RwLock::new(Teams::new()));
    let iron_team_repository_ref = team_repository_ref.clone();
    let iron_thread = thread::spawn(move ||{
        let server_address = env::var("address").expect("\"address\" in environment variables");
        info!("starting server at {}", server_address);

        Iron::new(server::chain(&iron_team_repository_ref)).http(server_address).unwrap();
    });

    let heartbeat_team_repository_ref = team_repository_ref.clone();
    let heartbeat_thread = thread::spawn(move ||{
        let mut heartbeat = Heartbeat::new(heartbeat_team_repository_ref);
        info!("starting heartbeat monitor");

        heartbeat.monitor();
    });

    iron_thread.join().unwrap();
    heartbeat_thread.join().unwrap();
    teams_thread.join().unwrap();
}

enum Message {
    Ack
}
