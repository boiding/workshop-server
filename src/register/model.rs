use std::convert::{From, Into};
use std::collections::HashMap;

pub trait TeamRepository {
    fn register(&mut self, registration: Registration) -> RegistrationAttempt;
}

pub struct Teams {
    teams: HashMap<String, Team>,
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
            return RegistrationAttempt::Failure(Reason::NameTaken);
        }

        for ip_address in self.ip_addresses() {
            if ip_address == registration.ip_address {
                return RegistrationAttempt::Failure(Reason::IPAddressTaken);
            }
        }

        self.teams.insert(registration.name.clone(), Team::from(registration));
        RegistrationAttempt::Success
    }
}

#[derive(PartialEq, Debug)]
pub enum RegistrationAttempt {
    Success,
    Failure(Reason)
}

#[derive(PartialEq, Debug)]
pub enum Reason {
    NameTaken,
    IPAddressTaken,
}

struct Team {
    name: String,
    ip_address: String,
}

impl From<Registration> for Team {
    fn from(registration: Registration) -> Self {
        Team { name: registration.name, ip_address: registration.ip_address }
    }
}

#[derive(Deserialize, Debug)]
pub struct Registration {
    name: String,
    ip_address: String,
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
