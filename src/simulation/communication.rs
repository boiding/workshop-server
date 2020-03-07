use super::{Registration, Unregistration, Intent};

#[derive(Debug)]
pub enum Message {
    Register(Registration),
    Unregister(Unregistration),
    Heartbeat,
    HeartbeatStatus((String, bool)),
    Tick,
    SpawnAll(usize),
    Spawn((String, usize)),
    BrainUpdate(String, Intent),
}
