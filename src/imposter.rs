use std::fs::File;
use std::io::Read;

use serde_derive::*;

use crate::can::{CANMessage, create_adaptor};
use crate::can::CANAdaptor;
use crate::controller::ImposterList;
use crate::stub::Stub;
use crate::utils;

// TODO: remove Debug
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Imposter {
    pub id: u32,
    pub stubs: Vec<Stub>,
    #[serde(skip_deserializing)]
    pub messages: Vec<CANMessage>
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

    pub fn received_messages(&mut self) -> &Vec<CANMessage> {
        &self.messages
    }

    pub fn responses_to_message(&mut self, message: &CANMessage) -> Vec<CANMessage> {
        self.messages.push(message.clone());
        for i in 0..(self.stubs.len()) {
            let stub = &mut self.stubs[i];
            if stub.matches_message(message) {
                return stub.generate_responses(message);
            }
        }
        Vec::new()
    }
}


pub fn run(id: u32, list: ImposterList) {
    let mut adaptor = create_adaptor().expect("Failed to initialize CAN device.");
    run_with_adaptor(id, list, adaptor.as_mut());
}

// mostly extracted from above to allow for testing with mock from integration test
pub fn run_with_adaptor(id: u32, mut list: ImposterList, adaptor: &mut CANAdaptor) {
    loop {
        match adaptor.receive() {
            Ok(message) => {
                list.do_with_imposter_by_id(id, |imposter| {
                    for response in imposter.responses_to_message(&message) {
                        adaptor.send(&response).expect("Failed to send CAN message.");
                    }
                });
            }
            Err(errmsg) => {
                println!("Failed to receive CAN message: {}", errmsg);
                break;
            }
        };
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

    #[test]
    fn records_received_messages() {
        let mut imposter = Imposter::from_json(r#"{ "id": 0, "stubs": [] }"#);
        let message = CANMessage::with_content(0x202, 0, &[ 0x00 ]);

        imposter.responses_to_message(&message);

        let received = imposter.received_messages();

        assert_eq!(1, received.len());
        assert_eq!(0x202, received[0].id);
    }

}

