use std::io::Read;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;

use iron::{Request, Response, status};
use router::Router;
use serde_json;
use serde_json::Error;

use super::model::*;
use super::super::model::communication::Message;

pub fn router(tx: &Sender<Message>) -> Router {
    let mut router = Router::new();

    let registration_tx = tx.clone();
    let registration_tx_mutex = Arc::new(Mutex::new(registration_tx));
    router.post("/", move |request: &mut Request|{
        let mut body: String = String::new();
        if let Ok(_) = request.body.read_to_string(&mut body) {
            let registration_result: Result<Registration, Error> = serde_json::from_str(&body);
            if let Ok(registration) = registration_result {
                info!("received {:?}", registration);

                registration_tx_mutex
                    .lock().unwrap()
                    .send(Message::Register(registration)).unwrap();

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
    }, "register");

    let unregister_tx = tx.clone();
    let unregister_tx_mutex = Arc::new(Mutex::new(unregister_tx));
    router.delete("/", move |request: &mut Request|{
        let mut body: String = String::new();
        if let Ok(_) = request.body.read_to_string(&mut body) {
            let unregistration_result: Result<Unregistration, Error> = serde_json::from_str(&body);
            if let Ok(unregistration) = unregistration_result {
                info!("received {:?}", unregistration);

                unregister_tx_mutex
                    .lock().unwrap()
                    .send(Message::Unregister(unregistration)).unwrap();

                Ok(Response::with(status::NoContent))
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
