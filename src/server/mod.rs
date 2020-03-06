mod handler;

use tokio::sync::mpsc::Sender;
use gotham::router::{builder::*, Router};

use super::simulation::communication::Message;

pub fn router(tx: &Sender<Message>) -> Router {
    build_simple_router(|route|{
        route.get("/").to_dir("static/");
        route.associate("/register", |assoc|{
            assoc.post().to_new_handler(handler::Register::new(tx.clone()));
            assoc.delete().to_new_handler(handler::Unregister::new());
        });
    })
}
