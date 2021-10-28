use std::{sync::mpsc::Sender, thread, time::Duration};

use crate::simulation::communication::Message as SimulationMessage;

pub struct Clock {
    tick_duration: Duration,
    tx: Sender<SimulationMessage>,
}

impl Clock {
    pub fn new(tick_duration: Duration, tx: Sender<SimulationMessage>) -> Self {
        Self { tick_duration, tx }
    }

    pub fn start(&mut self) {
        loop {
            thread::sleep(self.tick_duration);
            if let Err(error) = self.tx.send(SimulationMessage::Tick) {
                error!("Could not send tick message: {}", error);
            }
        }
    }
}
