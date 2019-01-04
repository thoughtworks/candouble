extern crate candouble;
extern crate serde_json;
extern crate hyper;
extern crate gotham;

use std::str;
use serde_json::{Value, Map};
use gotham::test::{TestServer, TestClient, TestResponse};
use candouble::webapi;
use candouble::utils;


fn client() -> TestClient {
    TestServer::new(webapi::router()).unwrap().client()
}

fn get(client: &TestClient, path: &str) -> TestResponse {
    client.get(&format!("http://testhost{}", path)).perform().unwrap()
}

fn post(client: &TestClient, path: &str, body: String) -> TestResponse {
    client.post(format!("http://testhost{}", path), body, mime::APPLICATION_JSON).perform().unwrap()
}

fn as_json_obj(response: TestResponse) -> Map<String, Value> {
    let body = response.read_utf8_body().unwrap();
    let json_val: Value = utils::from_json(&body);
    json_val.as_object().expect("expected object").clone()
}


#[test]
fn it_can_ping_api() {
    let client = client();
    let response = get(&client, "/ping");

    assert_eq!(response.status(), 200);
    let obj = as_json_obj(response);
    assert_eq!("ok", obj.get("status").unwrap());
}

#[test]
fn it_can_post_new_imposter() {
    let imposter = r#"{
                    "id": 1x,
                    "stubs": [
                        { "predicates": [{ "eq": { "id": "0x01" } }],
                          "responses": [{ "id": "0x02", "data": [ "0x17" ] }] }
                    ]}"#;

    let client = client();

    let response = post(&client, "/imposters", imposter.to_string());
    assert_eq!(response.status(), 201);
}

