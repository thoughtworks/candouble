use std::thread;
use std::time::Duration;
use std::io::Read;
use std::fs::File;
use serde_json;
use predicate::Predicate;
use response::ResponseTemplate;
use can::CANMessage;
use response::Behavior;


#[derive(Serialize, Deserialize, Debug)]
pub struct StubDefinition {
    predicates: Vec<Predicate>,
    responses: Vec<ResponseTemplate>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Stub {
    def: StubDefinition,
    response_idx: usize,
    response_repeats: usize,
}

impl Stub {
    pub fn from_str(s: &str) -> Result<Stub, &'static str> {
        let def: StubDefinition = serde_json::from_str(s).expect("Failed to parse JSON");
        Ok(Stub { def, response_idx: 0, response_repeats: 0 })
    }

    pub fn from_file(filename: &str) -> Result<Stub, &'static str> {
        let mut file = File::open(filename).expect(&format!("Failed to open file {}", filename));
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect(&format!("Failed to read file {}", filename));
        Stub::from_str(&contents)
    }

    pub fn matches_message(&self, message: &CANMessage) -> bool {
        self.def.predicates.iter().find(|p| p.eval(message) == false).is_none()
    }

    pub fn generate_response(&mut self, message: &CANMessage) -> CANMessage {
        if self.def.responses.len() == 0 {
            panic!("cannot generate response; no response template defined on stub");
        }

        let template = &self.def.responses[self.response_idx];
        let mut done_with_response = true;

        if let Some(b) = &template.behaviors {
            match &b[0] {
                Behavior::Wait(arg) => {
                    let millis = Duration::from_millis(*arg);
                    thread::sleep(millis);
                }
                Behavior::Repeat(arg) => {
                    if self.response_repeats == 0 {
                        self.response_repeats = *arg as usize;
                    }
                    self.response_repeats -= 1;
                    if self.response_repeats > 0 {
                        done_with_response = false;
                    }
                }
            }
        }

        if done_with_response {
            self.response_idx = (self.response_idx + 1) % self.def.responses.len();
        }

        template.generate_response(message)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use can::CANMessage;
    use std::time::Instant;

    #[test]
    fn cycles_through_multiple_responses() {
        let mut stub = Stub::from_str(r#"{
                     "predicates": [],
                     "responses": [
                        { "id": "0x01", "data": [ "0x17" ] },
                        { "id": "0x02", "data": [ "0x17" ] }
                      ]
                   }"#).expect("");

        let response1 = stub.generate_response(&CANMessage::new());
        assert_eq!(0x01, response1.id);
        let response2 = stub.generate_response(&CANMessage::new());
        assert_eq!(0x02, response2.id);
        let response3 = stub.generate_response(&CANMessage::new());
        assert_eq!(0x01, response3.id);
    }

    #[test]
    fn wait_behavior_waits_before_responding() {
        let mut stub = Stub::from_str(r#"{
                     "predicates": [],
                     "responses": [
                        { "id": "0x01", "data": [ "0x17" ], "_behaviors": [ { "wait": 50 } ] }
                      ]
                   }"#).expect("");

        let start = Instant::now();
        stub.generate_response(&CANMessage::new());
        let end = Instant::now();
        assert!(end.duration_since(start).subsec_millis() >= 50);
    }

    #[test]
    fn repeat_behavior_repeats_a_message() {
        let mut stub = Stub::from_str(r#"{
                     "predicates": [],
                     "responses": [
                        { "id": "0x01", "data": [ "0x17" ], "_behaviors": [ { "repeat": 2 } ] },
                        { "id": "0x02", "data": [ "0x17" ] }
                      ]
                   }"#).expect("");

        let response1 = stub.generate_response(&CANMessage::new());
        assert_eq!(0x01, response1.id);
        let response2 = stub.generate_response(&CANMessage::new());
        assert_eq!(0x01, response2.id);
        let response3 = stub.generate_response(&CANMessage::new());
        assert_eq!(0x02, response3.id);
        let response4 = stub.generate_response(&CANMessage::new());
        assert_eq!(0x01, response4.id);
    }
}
