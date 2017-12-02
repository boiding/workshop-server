use std::sync::{Arc, RwLock};

use iron::Chain;
use logger::Logger;
use mount::Mount;

use super::model::Teams;
use super::register;

pub fn chain(team_repository_ref: &Arc<RwLock<Teams>>) -> Chain {
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
