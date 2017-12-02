use std::env;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

use futures::Future;
use hyper::{Client, Method, Request, Uri};
use tokio_core::reactor::Core;

use super::communication::Message;

pub struct Heartbeat {
    rx: Receiver<HeartbeatMessage>,
    tx: Sender<Message>,
}

impl Heartbeat {
    pub fn new(rx: Receiver<HeartbeatMessage>, tx: Sender<Message>) -> Heartbeat {
        Heartbeat { rx, tx }
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
            thread::sleep(sleep_duration);
            self.tx.send(Message::Heartbeat).unwrap();

            let message = self.rx.recv().unwrap();
            match message {
                HeartbeatMessage::Check(servers) => {
                    for (team_name, uri) in servers {
                        info!("heartbeat for {} at {}", team_name, uri);
                        let request = Request::new(Method::Head, uri);
                        let work = client
                            .request(request)
                            .map(|response|{
                                info!("{} {}", team_name, response.status());
                            })
                            .map_err(|_|{
                                error!("{} disconnected", team_name);
                            });

                        match core.run(work) {
                            _ => () /* Everything is fine */
                        }
                    }
                },
            }
        }
    }
}


pub enum HeartbeatMessage {
    Check(Vec<(String, Uri)>),
}
