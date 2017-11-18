extern crate bws;
extern crate dotenv;
extern crate iron;
#[macro_use] extern crate log;
extern crate logger;
extern crate mount;
extern crate router;
extern crate simplelog;

use std::env;

use dotenv::dotenv;
use iron::{Iron, Chain};
use logger::Logger;
use mount::Mount;
use simplelog::{Config, LogLevelFilter, TermLogger, CombinedLogger};

use bws::register;

fn main() {
    dotenv().ok();
    CombinedLogger::init(
        vec![
            TermLogger::new(LogLevelFilter::Info, Config::default()).unwrap(),
        ]
    ).unwrap();

    info!("Logger configured");

    let server_address = env::var("address").expect("\"address\" in environment variables");
    info!("starting server at {}", server_address);

    Iron::new(chain()).http(server_address).unwrap();
}

fn chain() -> Chain {
    let mut chain = Chain::new(mount());
    let (logger_before, logger_after) = Logger::new(None);
    chain.link_before(logger_before);
    chain.link_after(logger_after);

    chain
}

fn mount() -> Mount {
    let mut mount = Mount::new();

    mount.mount("/register", register::router());

    mount
}
