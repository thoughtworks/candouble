use std::thread;
use serde_json::Value;
use gotham::state::{State, FromState};
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::helpers::http::response::{create_response, create_empty_response};
use futures::{future, Future, Stream};
use hyper::{Body, Response, StatusCode};
use crate::utils;


pub struct WebApi {}

impl WebApi {
    pub fn new() -> WebApi {
        WebApi {}
    }

    pub fn run(&mut self, host: &str, port: u16) {
        let addr = format!("{}:{}", host, port);
        thread::spawn(move || {
            println!("Listening for requests at http://{}", addr);
            gotham::start(addr, router());
        });
    }

    fn get_ping(state: State) -> (State, Response<Body>) {
        let body = r#"{ "status": "ok" }"#;
        let response = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, body);
        (state, response)
    }

    fn post_imposter(mut state: State) -> Box<HandlerFuture> {
        let f = Body::take_from(&mut state).concat2().then(|full_body|
            match full_body {
                Ok(valid_body) => {
                    let body_content = String::from_utf8(valid_body.to_vec()).unwrap();
                    let json_val: Value = utils::from_json(&body_content);
                    println!("imposters <= {}", json_val);
                    let response = create_empty_response(&state, StatusCode::CREATED);
                    future::ok((state, response))
                }
                Err(e) => {
                    future::err((state, e.into_handler_error()))
                }
            });
        Box::new(f)
    }
}

pub fn router() -> Router {
    build_simple_router(|route| {
        route.get("/ping").to(WebApi::get_ping);
        route.post("/imposters").to(WebApi::post_imposter);
    })
}
