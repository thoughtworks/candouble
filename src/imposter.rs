use std::borrow::BorrowMut;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};

use gotham_derive::*;
use serde_derive::*;

use crate::can::{CANAdaptor, CANMessage, create_adaptor};
use crate::stub::Stub;
use crate::utils;


#[derive(Clone, StateData)]
pub struct ImposterList {
    inner: Arc<Mutex<Vec<Imposter>>>,
}

impl ImposterList {

    pub fn new()-> Self {
        Self { inner: Arc::new(Mutex::new(Vec::new())) }
    }

    pub fn add(&self, imposter: Imposter) {
        let mut guard = self.inner.lock().unwrap();
        guard.borrow_mut().push(imposter);
    }

    pub fn get_by_id(&self, id: u32) -> Option<Imposter> {
        let mut guard = self.inner.lock().unwrap();
        for imposter in guard.borrow_mut().iter_mut() {
            if imposter.id == id {
                return Some(imposter.clone());
            }
        }
        None
    }

    pub fn get_all(&self) -> Vec<Imposter> {
        let mut guard = self.inner.lock().unwrap();
        guard.borrow_mut().clone()
    }

}


#[derive(Clone, Serialize, Deserialize)]
pub struct Imposter {
    pub id: u32,
    pub stubs: Vec<Stub>,
}


impl Imposter {

    pub fn from_json(json: &str) -> Imposter {
        utils::from_json(json)
    }

    pub fn from_file(filename: &str) -> Imposter {
        println!("Reading imposter from file: {}", filename);
        let mut file = File::open(filename).expect(&format!("Failed to open file {}", filename));
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect(&format!("Failed to read file {}", filename));
        Imposter::from_json(&contents)
    }

    pub fn run(&mut self) {
        println!("Running an imposter with id {}...", self.id);
        let mut adaptor = create_adaptor().expect("Failed to initialize CAN device.");
        loop {
            match adaptor.receive() {
                Ok(message) => self.handle_message(adaptor.as_mut(), &message),
                Err(errmsg) => { println!("Error: {}", errmsg); break; }
            }
        }
    }

    pub fn handle_message(&mut self, adaptor: &mut CANAdaptor, message: &CANMessage) {
        for response in self.responses_to_message(&message) {
            adaptor.send(&response).expect("Failed to send CAN message.");
        }
    }

    pub fn responses_to_message(&mut self, message: &CANMessage) -> Vec<CANMessage> {
        for i in 0..(self.stubs.len()) {
            let stub = &mut self.stubs[i];
            if stub.matches_message(message) {
                return stub.generate_responses(message);
            }
        }
        vec![]
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_stub_from_json_definition() {
        let imposter = Imposter::from_json(r#"{
            "id": 12,
            "stubs": [
                {
                  "predicates": [{ "eq": { "id": "0x200" } }],
                  "responses": [{ "id": "0x0201", "data": [ "0x01" ] }]
                }
            ]}"#);

        assert_eq!(12, imposter.id);
        assert_eq!(1, imposter.stubs.len());
    }

    #[test]
    fn returns_response_from_first_matching_stub() {
        let mut imposter = Imposter::from_json(r#"{
            "id": 1,
            "stubs": [
                {
                    "predicates": [{ "msg": { "id": "0x201", "data": [ "0x00" ] } }],
                    "responses": [{ "id": "0x0201", "data": [ "0x12" ] }]
                },
                {
                    "predicates": [{ "eq": { "id": "0x202" } }],
                    "responses": [{ "id": "0x0202", "data": [ "0x12" ] }]
                },
                {
                    "predicates": [{ "eq": { "id": "*" } }],
                    "responses": [{ "id": "0xFFFF", "data": [ "0x12" ] }]
                }
            ]}"#);

        let message = CANMessage::with_content(0x202, 0, &[ 0x00 ]);

        let responses = imposter.responses_to_message(&message);

        assert_eq!(1, responses.len());
        assert_eq!(0x202, responses[0].id);
    }

}

