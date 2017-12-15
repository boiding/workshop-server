use std::sync::mpsc::Sender;
use std::path::Path;

use iron::Chain;
use logger::Logger;
use mount::Mount;
use staticfile::Static;

use super::simulation::communication::Message;
use super::register;

pub fn chain(tx: &Sender<Message>) -> Chain {
    let mut chain = Chain::new(mount(&tx));
    let (logger_before, logger_after) = Logger::new(None);
    chain.link_before(logger_before);
    chain.link_after(logger_after);

    chain
}

fn mount(tx: &Sender<Message>) -> Mount {
    let mut mount = Mount::new();

    mount.mount("/", Static::new(Path::new("static/")));
    mount.mount("/register", register::router(&tx));

    mount
}
