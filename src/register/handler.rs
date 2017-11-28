use std::io::Read;
use std::sync::{Arc, RwLock};

use iron::{Request, Response, status};
use router::Router;
use serde_json;
use serde_json::Error;

use super::model::*;
use super::super::model::Teams;

pub fn router(tr_ref: &Arc<RwLock<Teams>>) -> Router {
    let mut router = Router::new();

    let registration_team_repository_ref = tr_ref.clone();
    router.post("/", move |request: &mut Request|{
        let mut body: String = String::new();
        if let Ok(_) = request.body.read_to_string(&mut body) {
            let registration_result: Result<Registration, Error> = serde_json::from_str(&body);
            if let Ok(registration) = registration_result {
                info!("received {:?}", registration);

                let mut team_repository = registration_team_repository_ref.write().unwrap();
                let attempt = team_repository.register(registration);
                match attempt {
                    RegistrationAttempt::Success => {
                        info!("Successfully registered");
                        Ok(Response::with(status::NoContent))
                    },

                    RegistrationAttempt::Failure(reason) => {
                        match reason {
                            RegistrationFailureReason::NameTaken => {
                                error!("name already registered");
                                let reason = RegistrationFailure::new(
                                    format!("name already registered")
                                );
                                let payload = serde_json::to_string(&reason).unwrap();

                                Ok(Response::with((status::InternalServerError, payload)))
                            },

                            RegistrationFailureReason::IPAddressTaken => {
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

    let unregister_team_repository_ref = tr_ref.clone();
    router.delete("/", move |request: &mut Request|{
        let mut body: String = String::new();
        if let Ok(_) = request.body.read_to_string(&mut body) {
            let unregistration_result: Result<Unregistration, Error> = serde_json::from_str(&body);
            if let Ok(unregistration) = unregistration_result {
                info!("received {:?}", unregistration);

                let mut team_repository = unregister_team_repository_ref.write().unwrap();
                let attempt = team_repository.unregister(unregistration);
                match attempt {
                    UnregistrationAttempt::Success => {
                        info!("Successfully unregistered");
                        Ok(Response::with(status::NoContent))
                    },

                    UnregistrationAttempt::Failure(reason) => {
                        match reason {
                            UnregistrationFailureReason::NameNotRegistered => {
                                error!("name not registered");
                                let reason = UnregistrationFailure::new(
                                    format!("name not registered")
                                );
                                let payload = serde_json::to_string(&reason).unwrap();

                                Ok(Response::with((status::InternalServerError, payload)))
                            },
                        }
                    }
                }
            } else {
                error!("unable to deserialize unregistation \"{}\"", body);
                let reason = UnregistrationFailure::new(
                    format!("unable to deserialize unregistration \"{}\"", body)
                );
                let payload = serde_json::to_string(&reason).unwrap();

                Ok(Response::with((status::InternalServerError, payload)))
            }
        } else {
            error!("unable to read body");
            let reason = UnregistrationFailure::new("unable to read body");
            let payload = serde_json::to_string(&reason).unwrap();

            Ok(Response::with((status::InternalServerError, payload)))
        }
    }, "unregister");

    router
}
