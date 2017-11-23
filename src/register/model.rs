use std::convert::{From, Into};
use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};

use hyper::{self, Uri};

pub trait TeamRepository {
    fn register(&mut self, registration: Registration) -> RegistrationAttempt;
    fn unregister(&mut self, unregistration: Unregistration) -> UnregistrationAttempt;
}

pub struct Teams {
    pub teams: HashMap<String, Team>,
}

impl Teams {
    pub fn new() -> Teams {
        Teams { teams: HashMap::new() }
    }

    fn ip_addresses(&self) -> Vec<&str> {
        self.teams.iter().map(|team| &team.1.ip_address[..]).collect()
    }
}

impl TeamRepository for Teams {
    fn register(&mut self, registration: Registration) -> RegistrationAttempt {
        if self.teams.contains_key(&registration.name) {
            return RegistrationAttempt::Failure(RegistrationFailureReason::NameTaken);
        }

        for ip_address in self.ip_addresses() {
            if ip_address == registration.ip_address {
                return RegistrationAttempt::Failure(RegistrationFailureReason::IPAddressTaken);
            }
        }

        self.teams.insert(registration.name.clone(), Team::from(registration));
        RegistrationAttempt::Success
    }

    fn unregister(&mut self, unregistration: Unregistration) -> UnregistrationAttempt {
        if !self.teams.contains_key(&unregistration.name) {
            return UnregistrationAttempt::Failure(UnregistrationFailureReason::NameNotRegistered);
        }

        self.teams.remove(&unregistration.name);
        UnregistrationAttempt::Success
    }
}

#[derive(PartialEq, Debug)]
pub enum RegistrationAttempt {
    Success,
    Failure(RegistrationFailureReason)
}

#[derive(PartialEq, Debug)]
pub enum RegistrationFailureReason {
    NameTaken,
    IPAddressTaken,
}

pub struct Team {
    name: String,
    ip_address: String,
    port: u16,
}

impl Team {
    pub fn heartbeat_uri(&self) -> Result<Uri, hyper::error::UriError> {
        let address = format!("{}://{}:{}/heartbeat", "http", self.ip_address, self.port);

        address.parse()
    }
}

impl From<Registration> for Team {
    fn from(registration: Registration) -> Self {
        Team { name: registration.name, ip_address: registration.ip_address, port: registration.port }
    }
}

impl Display for Team {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} {}", self.name, self.ip_address)
    }
}

#[derive(Deserialize, Debug)]
pub struct Registration {
    name: String,
    ip_address: String,
    port: u16,
}

#[derive(Serialize, Debug)]
pub struct RegistrationFailure {
    reason: String,
}

impl RegistrationFailure {
    pub fn new<S>(reason: S) -> RegistrationFailure where S: Into<String> {
        RegistrationFailure { reason: reason.into() }
    }
}

#[derive(PartialEq, Debug)]
pub enum UnregistrationAttempt {
    Success,
    Failure(UnregistrationFailureReason)
}

#[derive(PartialEq, Debug)]
pub enum UnregistrationFailureReason {
    NameNotRegistered,
}

#[derive(Deserialize, Debug)]
pub struct Unregistration {
    name: String,
}

#[derive(Serialize, Debug)]
pub struct UnregistrationFailure {
    reason: String,
}

impl UnregistrationFailure {
    pub fn new<S>(reason: S) -> UnregistrationFailure where S: Into<String> {
        UnregistrationFailure { reason: reason.into() }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_should_be_registered_with_a_registration() {
        let mut teams = Teams::new();
        let registration = Registration {
            name : "TEST".to_owned(),
            ip_address : "TEST ADDRESS".to_owned(),
        };

        let result = teams.register(registration);

        assert_eq!(result, RegistrationAttempt::Success);
    }

    #[test]
    fn team_with_same_name_should_not_be_registered() {
        let mut teams = Teams::new();
        let first = Registration {
            name : "TEST".to_owned(),
            ip_address : "TEST ADDRESS".to_owned(),
        };
        let _ = teams.register(first);

        let second = Registration {
            name : "TEST".to_owned(),
            ip_address : "OTHER TEST ADDRESS".to_owned(),
        };
        let result = teams.register(second);

        assert_eq!(result, RegistrationAttempt::Failure(Reason::NameTaken));
    }

    #[test]
    fn team_with_same_ip_address_should_not_be_registered() {
        let mut teams = Teams::new();
        let first = Registration {
            name : "TEST".to_owned(),
            ip_address : "TEST ADDRESS".to_owned(),
        };
        let _ = teams.register(first);

        let second = Registration {
            name : "OTHER TEST".to_owned(),
            ip_address : "TEST ADDRESS".to_owned(),
        };
        let result = teams.register(second);

        assert_eq!(result, RegistrationAttempt::Failure(Reason::IPAddressTaken));
    }
}
