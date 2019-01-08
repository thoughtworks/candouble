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

fn url(path: &str) -> String {
    format!("http://testhost{}", path)
}

fn as_json_obj(response: TestResponse) -> Map<String, Value> {
    let body = response.read_utf8_body().unwrap();
    let json_val: Value = utils::from_json(&body);
    json_val.as_object().expect("expected object").clone()
}


#[test]
fn it_can_ping_api() {
    let client = client(ImposterList::new());

    let response = client.get(&url("/ping")).perform().unwrap();

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

    let response = client.post(url("/imposters"), doc.to_string(), mime::APPLICATION_JSON).perform().unwrap();

    assert_eq!(201, response.status());
    assert_eq!(1, list.get_all().len());
}

#[test]
fn it_can_post_to_update_imposter() {
    let doc = r#"{ "id": 1, "stubs": [] }"#;
    let list = ImposterList::new();
    let client = client(list.clone());

    client.post(url("/imposters"), doc.to_string(), mime::APPLICATION_JSON).perform().unwrap();
    let response = client.post(url("/imposters"), doc.to_string(), mime::APPLICATION_JSON).perform().unwrap();

    assert_eq!(200, response.status());
//    assert_eq!(&format!("{}/1", url("/imposters")), response.headers().get("Location").unwrap());
    assert_eq!(1, list.get_all().len());
}

#[test]
fn it_returns_400_for_unparseable_json() {
    let doc = r#"{ "id": 1"#;
    let list = ImposterList::new();
    let client = client(list.clone());

    let response = client.post(url("/imposters"), doc.to_string(), mime::APPLICATION_JSON).perform().unwrap();

    assert_eq!(400, response.status());
    assert_eq!(0, list.get_all().len());
}


#[test]
fn it_can_get_all_imposters() {
    let list = ImposterList::new();
    list.upsert(Imposter::from_json(r#"{ "id": 1, "stubs": [ ] }"#));
    list.upsert(Imposter::from_json(r#"{ "id": 2, "stubs": [ ] }"#));
    let client = client(list.clone());

    let response = client.get(&url("/imposters")).perform().unwrap();

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

    let response = client.get(&url("/imposters/3")).perform().unwrap();

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

    let response = client.get(&url("/imposters/2")).perform().unwrap();

    assert_eq!(404, response.status());
}

#[test]
fn it_can_delete_imposter_by_id() {
    let list = ImposterList::new();
    list.upsert(Imposter::from_json(r#"{ "id": 1, "stubs": [ ] }"#));
    let client = client(list.clone());

    let response = client.delete(&url("/imposters/1")).perform().unwrap();

    assert_eq!(204, response.status());
    assert_eq!(0, list.get_all().len());
}
