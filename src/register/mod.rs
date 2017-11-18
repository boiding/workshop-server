use std::io::Read;
use std::convert::Into;

use iron::{Request, Response, IronResult, status};
use router::Router;
use serde_json;
use serde_json::{Error};

#[derive(Deserialize, Debug)]
struct Registration {
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

pub fn router() -> Router {
    let mut router = Router::new();
    router.post("/", handler, "register");

    router
}

fn handler(request: &mut Request) -> IronResult<Response> {
    let mut body: String = String::new();
    if let Ok(_) = request.body.read_to_string(&mut body) {
        let registration_result: Result<Registration, Error> = serde_json::from_str(&body);
        if let Ok(registration) = registration_result {
            info!("received {:?}", registration);

            Ok(Response::with(status::NoContent))
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
}
