use std::env;
use std::io::{self, Write};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use futures::{Future, Stream};
use hyper::Client;
use tokio_core::reactor::Core;

use super::register::Teams;

pub struct Heartbeat {
    team_repository_ref: Arc<RwLock<Teams>>,
}

impl Heartbeat {
    pub fn new(team_repository_ref: Arc<RwLock<Teams>>) -> Heartbeat {
        Heartbeat { team_repository_ref }
    }

    pub fn monitor(&mut self) {
        let mut core = Core::new().unwrap(); // TODO handle error?
        let client = Client::new(&core.handle());

        let sleep_duration_value = u64::from_str_radix(
            &env::var("heartbeat_sleep_duration")
                .expect("\"heartbeat_sleep_duration\" in environment variables")
                , 10).expect("\"heartbeat_sleep_duration\" to be u64");
        let sleep_duration = Duration::from_secs(sleep_duration_value);

        loop {
            let uri = "http://httpbin.org/ip".parse().unwrap(); // TODO handle error?
            let work = client.get(uri).and_then(|res| {
                println!("Response: {}", res.status());

                res.body().for_each(|chunk| {
                    io::stdout()
                        .write_all(&chunk)
                        .map_err(From::from)
                })
            });
            core.run(work).unwrap(); // TODO handle error?
            thread::sleep(sleep_duration);
        }
    }
}
