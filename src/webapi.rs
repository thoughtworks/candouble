use std::thread;
use hyper::{Response, Body, StatusCode};
use gotham::state::State;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::helpers::http::response::create_response;


pub struct WebApi {
}

impl WebApi {

    pub fn new() -> WebApi {
        WebApi { }
    }

    pub fn run(&mut self, host: &str, port: u16) {
        let addr = format!("{}:{}", host, port);
        thread::spawn(move || {
            println!("Listening for requests at http://{}", addr);
            gotham::start(addr, router());
        });
    }

    fn ping(state: State) -> (State, Response<Body>) {
        let body = r#"{ "status": "ok" }"#;
        let response = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, body);
        (state, response)
    }

}

pub fn router() -> Router {
    build_simple_router(|route| {
        route.get("/ping").to(WebApi::ping);
    })
}
