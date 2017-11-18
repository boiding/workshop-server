use iron::{Request, Response, IronResult, status};
use router::Router;

pub fn router() -> Router {
    let mut router = Router::new();
    router.post("/", handler, "register");

    router
}

fn handler(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with(status::NoContent))
}
