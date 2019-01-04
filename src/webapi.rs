use std::thread;

use futures::{future, Future, Stream};
use gotham::handler::HandlerFuture;
use gotham::helpers::http::response::{create_empty_response, create_response};
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{FromState, State};
use hyper::{Body, Response, StatusCode};
use serde_derive::*;
use serde_json::Value;

use crate::imposter::Imposter;
use crate::imposter::ImposterList;
use crate::utils;

#[derive(Serialize, Clone)]
struct ImposterWrapper {
    imposters: Vec<Imposter>
}

pub fn start_listener(host: &str, port: u16, imposters: ImposterList) {
    let addr = format!("{}:{}", host, port);
    thread::spawn(move || {
        println!("Listening for requests at http://{}", addr);
        gotham::start(addr, router(imposters));
    });
}

pub fn router(imposters: ImposterList) -> Router {
    let middleware = StateMiddleware::new(imposters);
    let pipeline = pipeline::single_middleware(middleware);
    let (chain, pipelines) = pipeline::single::single_pipeline(pipeline);
    build_router(chain, pipelines, |route| {
        route.get("/ping").to(get_ping);
        route.get("/imposters").to(get_imposters);
        route.post("/imposters").to(post_imposter);
    })
}

fn get_ping(state: State) -> (State, Response<Body>) {
    let body = "{ \"status\": \"ok\" }\n";
    let response = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, body);
    (state, response)
}

fn get_imposters(state: State) -> (State, Response<Body>) {
    let imposters = ImposterList::borrow_from(&state).get_all();
    let wrapper = ImposterWrapper { imposters };
    let mut response_body = serde_json::to_string_pretty(&wrapper).unwrap();
    response_body.push('\n');
    let response = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, response_body);
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
