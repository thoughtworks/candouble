use std::thread;
use serde_json::Value;
use futures::{future, Future, Stream};
use hyper::{Body, Response, StatusCode};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::helpers::http::response::{create_response, create_empty_response};
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::single_middleware;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::state::{State, FromState};
use crate::utils;
use crate::imposter::ImposterList;
use crate::imposter::Imposter;


pub struct WebApi {}

impl WebApi {
    pub fn new() -> WebApi {
        WebApi {}
    }

    pub fn run(&mut self, host: &str, port: u16, imposters: ImposterList) {
        let addr = format!("{}:{}", host, port);
        thread::spawn(move || {
            println!("Listening for requests at http://{}", addr);
            gotham::start(addr, router(imposters));
        });
    }

    fn get_ping(state: State) -> (State, Response<Body>) {
        let body = r#"{ "status": "ok" }"#;
        let response = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, body);
        (state, response)
    }

    fn post_imposter(mut state: State) -> Box<HandlerFuture> {
        let f = Body::take_from(&mut state).concat2().then(|full_body| {
            // TODO: consider adding explicit error handling
            let body_content = String::from_utf8(full_body.unwrap().to_vec()).unwrap();
            println!("imposters <= {}", utils::from_json::<Value>(&body_content));
            let imposter = Imposter::from_json(&body_content);
            ImposterList::borrow_from(&state).add(imposter);
            let response = create_empty_response(&state, StatusCode::CREATED);
            future::ok((state, response))
        });
        Box::new(f)
    }
}

pub fn router(imposters: ImposterList) -> Router {
    let middleware = StateMiddleware::new(imposters);
    let pipeline = single_middleware(middleware);
    let (chain, pipelines) = single_pipeline(pipeline);
    build_router(chain, pipelines, |route| {
        route.get("/ping").to(WebApi::get_ping);
        route.post("/imposters").to(WebApi::post_imposter);
    })
}
