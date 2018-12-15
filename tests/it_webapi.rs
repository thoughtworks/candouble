extern crate candouble;
extern crate serde_json;
extern crate hyper;
extern crate gotham;

use std::str;
use serde_json::{Value, Map};
use gotham::test::{TestServer, TestResponse};
use candouble::webapi;
use candouble::utils;


fn get(path: &str) -> TestResponse {
    TestServer::new(webapi::router())
        .unwrap()
        .client()
        .get(&format!("http://testhost{}", path))
        .perform()
        .unwrap()
}

fn as_json_obj(response: TestResponse) -> Map<String, Value> {
    let body = response.read_utf8_body().unwrap();
    let json_val: Value = utils::from_json(&body);
    json_val.as_object().expect("expected object").clone()
}


#[test]
fn it_can_ping_api() {

    let response = get("/ping");

    assert_eq!(response.status(), 200);
    let obj = as_json_obj(response);
    assert_eq!("ok", obj.get("status").unwrap());
}

