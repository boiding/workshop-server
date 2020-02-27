use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use super::simulation::communication::Message as TeamsMessage;

pub struct Clock {
    tick_duration: Duration,
    tx: Sender<TeamsMessage>,
}

impl Clock {
    pub fn new(tick_duration: Duration, tx: Sender<TeamsMessage>) -> Self {
        Self { tick_duration, tx }
    }

    pub fn start(&mut self) {
        loop {
            thread::sleep(self.tick_duration);
            if let Err(error) = self.tx.send(TeamsMessage::Tick) {
                error!("Could not send tick message: {}", error);
            }
        }
    }
}
