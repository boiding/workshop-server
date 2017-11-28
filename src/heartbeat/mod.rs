use std::env;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use futures::Future;
use hyper::{Request, Method, Client};
use tokio_core::reactor::Core;

use super::model::Teams;

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
            let team_repository = self.team_repository_ref.read().unwrap();
            for (_, team) in team_repository.teams.iter() {
                match team.heartbeat_uri() {
                    Ok(uri) => {
                        info!("heartbeat at {}", uri);
                        let request = Request::new(Method::Head, uri);
                        let work = client
                            .request(request)
                            .map(|response|{
                                info!("{} {}", team, response.status());
                            })
                            .map_err(|_|{
                                error!("{} disconnected", team);
                            });

                        match core.run(work) {
                            _ => () /* Everything is fine */
                        }
                    },

                    Err(e) => {
                        error!("{}", e)
                    }
                }
            }

            thread::sleep(sleep_duration);
        }
    }
}
