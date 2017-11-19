use std::convert::{From, Into};
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, RwLock};

use iron::{Request, Response, status};
use router::Router;
use serde_json;
use serde_json::{Error};

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
struct RegistrationFailure {
    reason: String,
}

impl RegistrationFailure {
    fn new<S>(reason: S) -> RegistrationFailure where S: Into<String> {
        RegistrationFailure { reason: reason.into() }
    }
}

pub fn router(tr_ref: &Arc<RwLock<Teams>>) -> Router {
    let mut router = Router::new();

    let team_repository_ref = tr_ref.clone();
    router.post("/", move |request: &mut Request|{
        let mut body: String = String::new();
        if let Ok(_) = request.body.read_to_string(&mut body) {
            let registration_result: Result<Registration, Error> = serde_json::from_str(&body);
            if let Ok(registration) = registration_result {
                info!("received {:?}", registration);

                let mut team_repository = team_repository_ref.write().unwrap();
                let attempt = team_repository.register(registration);
                match attempt {
                    RegistrationAttempt::Success => {
                        info!("Successfully registered");
                        Ok(Response::with(status::NoContent))
                    },

                    RegistrationAttempt::Failure(reason) => {
                        match reason {
                            Reason::NameTaken => {
                                error!("name already registered");
                                let reason = RegistrationFailure::new(
                                    format!("name already registered")
                                );
                                let payload = serde_json::to_string(&reason).unwrap();

                                Ok(Response::with((status::InternalServerError, payload)))
                            },

                            Reason::IPAddressTaken => {
                                error!("ip address already registered");
                                let reason = RegistrationFailure::new(
                                    format!("ip address already registered")
                                );
                                let payload = serde_json::to_string(&reason).unwrap();

                                Ok(Response::with((status::InternalServerError, payload)))
                            }
                        }
                    }
                }
            } else {
                error!("unable to deserialize registation \"{}\"", body);
                let reason = RegistrationFailure::new(
                    format!("unable to deserialize registration \"{}\"", body)
                );
                let payload = serde_json::to_string(&reason).unwrap();

                Ok(Response::with((status::InternalServerError, payload)))
            }
        } else {
            error!("unable to read body");
            let reason = RegistrationFailure::new("unable to read body");
            let payload = serde_json::to_string(&reason).unwrap();

            Ok(Response::with((status::InternalServerError, payload)))
        }
    }, "register");

    router
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
