use std::io::Read;
use std::sync::{Arc, RwLock};

use iron::{Request, Response, status};
use router::Router;
use serde_json;
use serde_json::Error;

use super::model::{Registration, RegistrationAttempt, RegistrationFailure, Reason, TeamRepository, Teams};

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
