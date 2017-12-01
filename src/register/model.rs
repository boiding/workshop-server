use std::convert::Into;

use super::super::model::{Teams, Team};

pub trait TeamRepository {
    fn register(&mut self, registration: Registration) -> RegistrationAttempt;
    fn unregister(&mut self, unregistration: Unregistration) -> UnregistrationAttempt;
}

impl TeamRepository for Teams {
    fn register(&mut self, registration: Registration) -> RegistrationAttempt {
        if self.teams.contains_key(&registration.name) {
            return RegistrationAttempt::Failure(RegistrationFailureReason::NameTaken);
        }

        if !self.available(&registration.ip_address, registration.port) {
            return RegistrationAttempt::Failure(RegistrationFailureReason::IPAddressWithPortTaken);
        }

        self.teams.insert(
            registration.name.clone(),
            registration.into(),
        );
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
    Failure(RegistrationFailureReason),
}

#[derive(PartialEq, Debug)]
pub enum RegistrationFailureReason {
    NameTaken,
    IPAddressWithPortTaken,
}

#[derive(Deserialize, Debug)]
pub struct Registration {
    name: String,
    ip_address: String,
    port: u16,
}

impl Into<Team> for Registration {
    fn into(self) -> Team {
        Team::new(self.name, self.ip_address, self.port)
    }
}

#[derive(Serialize, Debug)]
pub struct RegistrationFailure {
    reason: String,
}

impl RegistrationFailure {
    pub fn new<S>(reason: S) -> RegistrationFailure
    where
        S: Into<String>,
    {
        RegistrationFailure { reason: reason.into() }
    }
}

#[derive(PartialEq, Debug)]
pub enum UnregistrationAttempt {
    Success,
    Failure(UnregistrationFailureReason),
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
    pub fn new<S>(reason: S) -> UnregistrationFailure
    where
        S: Into<String>,
    {
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
            name: "TEST".to_owned(),
            ip_address: "TEST ADDRESS".to_owned(),
            port: 2643,
        };

        let result = teams.register(registration);

        assert_eq!(result, RegistrationAttempt::Success);
    }

    #[test]
    fn team_with_same_name_should_not_be_registered() {
        let mut teams = Teams::new();
        let first = Registration {
            name: "TEST".to_owned(),
            ip_address: "TEST ADDRESS".to_owned(),
            port: 2643,
        };
        let _ = teams.register(first);

        let second = Registration {
            name: "TEST".to_owned(),
            ip_address: "OTHER TEST ADDRESS".to_owned(),
            port: 2643,
        };
        let result = teams.register(second);

        assert_eq!(result, RegistrationAttempt::Failure(RegistrationFailureReason::NameTaken));
    }

    #[test]
    fn team_with_same_ip_address_should_not_be_registered() {
        let mut teams = Teams::new();
        let first = Registration {
            name: "TEST".to_owned(),
            ip_address: "TEST ADDRESS".to_owned(),
            port: 2643,
        };
        let _ = teams.register(first);

        let second = Registration {
            name: "OTHER TEST".to_owned(),
            ip_address: "TEST ADDRESS".to_owned(),
            port: 2643,
        };
        let result = teams.register(second);

        assert_eq!(result, RegistrationAttempt::Failure(RegistrationFailureReason::IPAddressWithPortTaken));
    }
}
