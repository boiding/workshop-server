use super::register::model::{Registration, Unregistration};

#[derive(Debug)]
pub enum Message {
    Register(Registration),
    Unregister(Unregistration),
    Heartbeat,
}
