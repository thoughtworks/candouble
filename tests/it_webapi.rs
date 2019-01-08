extern crate candouble;
extern crate gotham;
extern crate hyper;
extern crate serde_json;

use std::str;

use gotham::test::{TestClient, TestResponse, TestServer};
use serde_json::{Map, Value};

use candouble::imposter::Imposter;
use candouble::controller::ImposterList;
use candouble::utils;
use candouble::webapi;


fn client(imposter_list: ImposterList) -> TestClient {
    TestServer::new(webapi::router(imposter_list)).unwrap().client()
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
    let client = client(ImposterList::new());

    let response = get(&client, "/ping");

    assert_eq!(response.status(), 200);
    let obj = as_json_obj(response);
    assert_eq!("ok", obj.get("status").unwrap());
}

#[test]
fn it_can_post_new_imposter() {
    let doc = r#"{
                    "id": 1,
                    "stubs": [
                        { "predicates": [{ "eq": { "id": "0x01" } }],
                          "responses": [{ "id": "0x02", "data": [ "0x17" ] }] }
                    ]
                 }"#;
    let list = ImposterList::new();
    let client = client(list.clone());

    let response = post(&client, "/imposters", doc.to_string());

    assert_eq!(201, response.status());
    assert_eq!(1, list.get_all().len());
}

#[test]
fn it_can_get_all_imposters() {
    let list = ImposterList::new();
    list.upsert(Imposter::from_json(r#"{ "id": 1, "stubs": [ ] }"#));
    list.upsert(Imposter::from_json(r#"{ "id": 2, "stubs": [ ] }"#));
    let client = client(list.clone());

    let response = get(&client, "/imposters");

    assert_eq!(200, response.status());
    let obj = as_json_obj(response);
    assert_eq!(2, obj.get("imposters").unwrap().as_array().unwrap().len());
}

#[test]
fn it_can_get_imposter_by_id() {
    let list = ImposterList::new();
    list.upsert(Imposter::from_json(r#"{ "id": 1, "stubs": [ ] }"#));
    list.upsert(Imposter::from_json(r#"{ "id": 3, "stubs": [ ] }"#));
    let client = client(list.clone());

    let response = get(&client, "/imposters/3");

    assert_eq!(200, response.status());
    let obj = as_json_obj(response);
    assert_eq!(3, obj.get("id").unwrap().as_i64().unwrap());
}

#[test]
fn it_returns_404_for_non_existing_imposter() {
    let list = ImposterList::new();
    list.upsert(Imposter::from_json(r#"{ "id": 1, "stubs": [ ] }"#));
    list.upsert(Imposter::from_json(r#"{ "id": 3, "stubs": [ ] }"#));
    let client = client(list.clone());

    let response = get(&client, "/imposters/2");

    assert_eq!(404, response.status());
}
