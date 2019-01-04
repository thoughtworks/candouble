use std::io::Error;
use std::io::Read;
use std::fs::File;
use serde_derive::*;
use crate::can::{create_adaptor, CANMessage, CANAdaptor};
use crate::stub::{Stub, StubDefinition};
use crate::utils;


#[derive(Serialize, Deserialize, Debug)]
pub struct ImposterDefinition {
    id: u32,
    stubs: Vec<StubDefinition>,
}

pub struct Imposter {
    pub stubs: Vec<Stub>,
}


impl Imposter {
    pub fn new() -> Imposter {
        Imposter { stubs: Vec::new() }
    }

    pub fn from_json(json: &str) -> Imposter {
        let def: ImposterDefinition = utils::from_json(json);
        let mut imposter = Imposter::new();
        for stub in def.stubs.into_iter() {
            imposter.add_stub(Stub::new(stub))
        }
        imposter
    }

    pub fn from_file(filename: &str) -> Imposter {
        println!("Reading stub from file: {}", filename);
        let mut file = File::open(filename).expect(&format!("Failed to open file {}", filename));
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect(&format!("Failed to read file {}", filename));
        Imposter::from_json(&contents)
    }

    pub fn load_stub(&mut self, filename: &str) -> Result<(), Error> {
        println!("Reading stub from file: {}", filename);
        let mut file = File::open(filename).expect(&format!("Failed to open file {}", filename));
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect(&format!("Failed to read file {}", filename));
        let stub = Stub::new(utils::from_json(&contents));
        self.add_stub(stub);
        Ok(())
    }

    fn add_stub(&mut self, stub: Stub) {
        println!("Adding stub: {:?}", stub);
        self.stubs.push(stub);
    }

    pub fn run(&mut self) {
        println!("Running an imposter...");

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
    fn adds_stubs_from_json_definition() {
        let imposter = Imposter::from_json(r#"{
            "id": 1,
            "stubs": [
                {
                  "predicates": [{ "eq": { "id": "0x200" } }],
                  "responses": [{ "id": "0x0201", "data": [ "0x01" ] }]
                }
            ]}"#);

        assert_eq!(1, imposter.stubs.len());
    }

    #[test]
    fn returns_reponse_from_first_matching_stub() {
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

        let mut message = CANMessage::new();
        message.id = 0x0202;

        let responses = imposter.responses_to_message(&message);

        assert_eq!(1, responses.len());
        assert_eq!(0x0202, responses[0].id);
    }

}

