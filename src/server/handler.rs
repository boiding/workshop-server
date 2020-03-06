use futures::future;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::Sender;
use gotham::error::Result;
use gotham::handler::{Handler, HandlerFuture, NewHandler};
use gotham::helpers::http::response::create_empty_response;
use gotham::state::State;
use hyper::{StatusCode};

use crate::simulation::communication::Message;

#[derive(Clone)]
pub struct Register {
    tx_mutex : Arc<Mutex<Sender<Message>>>,
}

impl Register {
    pub fn new(tx: Sender<Message>) -> Self {
        let tx_mutex = Arc::new(Mutex::new(tx));
        Self { tx_mutex }
    }
}

impl Handler for Register {
    fn handle(self, state: State) -> Box<HandlerFuture> {
        let response = create_empty_response(&state, StatusCode::NO_CONTENT);
        
        Box::new(future::ok((state, response)))
    }
}

impl NewHandler for Register {
    type Instance = Self;

    fn new_handler(&self) -> Result<Self::Instance> {
        Ok(self.clone())
    }
}

#[derive(Clone)]
pub struct Unregister {

}

impl Unregister {
    pub fn new() -> Self {
        Self {}
    }
}

impl Handler for Unregister {
    fn handle(self, state: State) -> Box<HandlerFuture> {
        let response = create_empty_response(&state, StatusCode::NO_CONTENT);
               
        Box::new(future::ok((state, response)))
    }
}

impl NewHandler for Unregister {
    type Instance = Self;

    fn new_handler(&self) -> Result<Self::Instance> {
        Ok(self.clone())
    }
}