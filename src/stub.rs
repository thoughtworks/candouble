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

    pub fn generate_responses(&mut self, message: &CANMessage) -> Vec<CANMessage> {
        if self.def.responses.len() == 0 {
            panic!("cannot generate response; no response template defined on stub");
        }

        let mut responses = Vec::new();
        let mut idx = self.response_idx;
        let mut move_to_next_template = true;

        loop {
            let mut drop_response = false;
            let mut generate_another_response = false;

            let template = &self.def.responses[idx];
            if let Some(behaviors) = &template.behaviors {
                for b in behaviors {
                    match b {
                        Behavior::Wait(arg) => {
                            thread::sleep(Duration::from_millis(*arg));
                        }
                        Behavior::Repeat(arg) => {
                            if self.response_repeats == 0 {
                                self.response_repeats = *arg as usize;
                            }
                            self.response_repeats -= 1;
                            if self.response_repeats > 0 {
                                move_to_next_template = false;
                            }
                        }
                        Behavior::Drop(arg) => {
                            drop_response = *arg;
                        }
                        Behavior::Concat(arg) => {
                            generate_another_response = *arg;
                        }
                    }
                }
            }
            if !drop_response {
                responses.push(template.generate_response(message))
            }
            idx = (idx + 1) % self.def.responses.len();
            if !generate_another_response {
                break;
            }
        }

        if move_to_next_template {
            self.response_idx = idx;
        }

        responses
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

        let response1 = stub.generate_responses(&CANMessage::new())[0];
        assert_eq!(0x01, response1.id);
        let response2 = stub.generate_responses(&CANMessage::new())[0];
        assert_eq!(0x02, response2.id);
        let response3 = stub.generate_responses(&CANMessage::new())[0];
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
        stub.generate_responses(&CANMessage::new());
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

        let response1 = stub.generate_responses(&CANMessage::new())[0];
        assert_eq!(0x01, response1.id);
        let response2 = stub.generate_responses(&CANMessage::new())[0];
        assert_eq!(0x01, response2.id);
        let response3 = stub.generate_responses(&CANMessage::new())[0];
        assert_eq!(0x02, response3.id);
        let response4 = stub.generate_responses(&CANMessage::new())[0];
        assert_eq!(0x01, response4.id);
    }

    #[test]
    fn drop_behavior_drops_a_message() {
        let mut stub = Stub::from_str(r#"{
                     "predicates": [],
                     "responses": [
                        { "id": "0x01", "data": [ ], "_behaviors": [ { "drop": true } ] },
                        { "id": "0x02", "data": [ ] }
                      ]
                   }"#).expect("");

        let responses = stub.generate_responses(&CANMessage::new());
        assert_eq!(0, responses.len());
        let response2 = stub.generate_responses(&CANMessage::new())[0];
        assert_eq!(0x02, response2.id);
    }

    #[test]
    fn concat_behavior_concatenates_message() {
        let mut stub = Stub::from_str(r#"{
                     "predicates": [],
                     "responses": [
                        { "id": "0x01", "data": [ ], "_behaviors": [ { "concat": true } ] },
                        { "id": "0x02", "data": [ ], "_behaviors": [ { "concat": true } ] },
                        { "id": "0x03", "data": [ ] },
                        { "id": "0x04", "data": [ ] }
                      ]
                   }"#).expect("");

        let responses = stub.generate_responses(&CANMessage::new());
        assert_eq!(3, responses.len());
        assert_eq!(0x01, responses[0].id);
        assert_eq!(0x02, responses[1].id);
        assert_eq!(0x03, responses[2].id);
        let responses = stub.generate_responses(&CANMessage::new());
        assert_eq!(1, responses.len());
        assert_eq!(0x04, responses[0].id);
    }

    #[test]
    fn repeat_and_concat_behaviors_can_be_combined() {
        let mut stub = Stub::from_str(r#"{
                     "predicates": [],
                     "responses": [
                        { "id": "0x01", "data": [ ], "_behaviors": [
                                                        { "concat": true },
                                                        { "repeat": 2 }
                                                     ] },
                        { "id": "0x02", "data": [ ] },
                        { "id": "0x03", "data": [ ] }
                      ]
                   }"#).expect("");

        let responses1 = stub.generate_responses(&CANMessage::new());
        assert_eq!(2, responses1.len());
        let responses2 = stub.generate_responses(&CANMessage::new());
        assert_eq!(2, responses2.len());
        let responses3 = stub.generate_responses(&CANMessage::new());
        assert_eq!(1, responses3.len());
    }
}
