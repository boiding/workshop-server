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
use std::thread;

use dotenv::dotenv;
use iron::{Iron, Chain};
use logger::Logger;
use mount::Mount;
use simplelog::{Config, LogLevelFilter, TermLogger, CombinedLogger};

use bws::register;
use bws::register::Teams;
use bws::heartbeat::Heartbeat;

fn main() {
    dotenv().ok();
    CombinedLogger::init(
        vec![
            TermLogger::new(LogLevelFilter::Debug, Config::default()).unwrap(),
        ]
    ).unwrap();

    info!("Logger configured");

    let team_repository_ref = Arc::new(RwLock::new(Teams::new()));
    let iron_team_repository_ref = team_repository_ref.clone();
    let iron_thread = thread::spawn(move ||{
        let server_address = env::var("address").expect("\"address\" in environment variables");
        info!("starting server at {}", server_address);

        Iron::new(chain(&iron_team_repository_ref)).http(server_address).unwrap();
    });

    let heartbeat_team_repository_ref = team_repository_ref.clone();
    let heartbeat_thread = thread::spawn(move ||{
        let mut heartbeat = Heartbeat::new(heartbeat_team_repository_ref);

        heartbeat.monitor();
    });

    iron_thread.join().unwrap();
    heartbeat_thread.join().unwrap();
}

fn chain(team_repository_ref: &Arc<RwLock<Teams>>) -> Chain {
    let mut chain = Chain::new(mount(&team_repository_ref));
    let (logger_before, logger_after) = Logger::new(None);
    chain.link_before(logger_before);
    chain.link_after(logger_after);

    chain
}

fn mount(team_repository_ref: &Arc<RwLock<Teams>>) -> Mount {
    let mut mount = Mount::new();

    mount.mount("/register", register::router(&team_repository_ref));

    mount
}
