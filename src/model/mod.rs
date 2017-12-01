use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};

use hyper::{self, Uri};

pub struct Teams {
    pub teams: HashMap<String, Team>,
}

impl Teams {
    pub fn new() -> Teams {
        Teams { teams: HashMap::new() }
    }

    pub fn available(&self, ip_address: &str, port: u16) -> bool {
        self.teams
            .iter()
            .filter(|&(_name ,team)| team.ip_address == ip_address && team.port == port)
            .count() == 0
    }
}

pub struct Team {
    name: String,
    ip_address: String,
    port: u16,
}

impl Team {
    pub fn new<S>(name: S, ip_address: S, port: u16) -> Team where S: Into<String> {
        Team { name: name.into(), ip_address: ip_address.into(), port }
    }

    pub fn heartbeat_uri(&self) -> Result<Uri, hyper::error::UriError> {
        let address = format!("{}://{}:{}/heartbeat", "http", self.ip_address, self.port);

        address.parse()
    }
}

impl Display for Team {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} {}", self.name, self.ip_address)
    }
}

