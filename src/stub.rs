use std::thread;
use std::time::Duration;

use serde_derive::*;
use serde_json;

use crate::can::CANMessage;
use crate::predicate::Predicate;
use crate::response::{Behavior, ResponseTemplate};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stub {
    predicates: Vec<Predicate>,
    responses: Vec<ResponseTemplate>,
    #[serde(skip)]
    response_idx: usize,
    #[serde(skip)]
    response_repeats: usize,
}

impl Stub {

    pub fn matches_message(&self, message: &CANMessage) -> bool {
        self.predicates.iter().find(|p| p.eval(message) == false).is_none()
    }

    pub fn generate_responses(&mut self, message: &CANMessage) -> Vec<CANMessage> {
        if self.responses.len() == 0 {
            panic!("cannot generate response; no response template defined on stub");
        }

        let prev_idx = self.response_idx;
        let mut responses = Vec::new();
        let mut generate_response = true;

        while generate_response {
            responses.push(self.get_template().generate_response(message));
            generate_response = false;
            if let Some(behaviors) = self.get_template().behaviors.clone() {
                for b in behaviors {
                    match b {
                        Behavior::Wait(arg) => { thread::sleep(Duration::from_millis(arg)); }
                        Behavior::Repeat(arg) => { self.update_response_repeats(arg); }
                        Behavior::Drop(arg) => { if arg { responses.pop(); } }
                        Behavior::Concat(arg) => { generate_response = arg }
                    }
                }
            }
            self.inc_response_idx();
        }

        if self.response_repeats > 0 {
            self.response_idx = prev_idx;
        }

        responses
    }

    fn get_template(&self) -> &ResponseTemplate {
        &self.responses[self.response_idx]
    }

    fn inc_response_idx(&mut self) {
        self.response_idx = (self.response_idx + 1) % self.responses.len();
    }

    fn update_response_repeats(&mut self, count: usize) {
        if self.response_repeats == 0 {
            self.response_repeats = count;
        }
        self.response_repeats -= 1;
    }
}


#[cfg(test)]
mod tests {
    use std::time::Instant;
    use crate::can::CANMessage;
    use super::*;
    use crate::utils;

    // this methods is only here to make the return type explicit,
    // which in turn makes the tests a tiny bit more concise
    fn from_json(s: &str) -> Stub {
        utils::from_json(s)
    }

    #[test]
    fn cycles_through_multiple_responses() {
        let mut stub = from_json(r#"{
                     "predicates": [],
                     "responses": [
                        { "id": "0x01", "data": [ "0x17" ] },
                        { "id": "0x02", "data": [ "0x17" ] }
                      ]
                   }"#);

        let response1 = stub.generate_responses(&CANMessage::new())[0];
        assert_eq!(0x01, response1.id);
        let response2 = stub.generate_responses(&CANMessage::new())[0];
        assert_eq!(0x02, response2.id);
        let response3 = stub.generate_responses(&CANMessage::new())[0];
        assert_eq!(0x01, response3.id);
    }

    #[test]
    fn wait_behavior_waits_before_responding() {
        let mut stub = from_json(r#"{
                     "predicates": [],
                     "responses": [
                        { "id": "0x01", "data": [ "0x17" ], "_behaviors": [ { "wait": 50 } ] }
                      ]
                   }"#);

        let start = Instant::now();
        stub.generate_responses(&CANMessage::new());
        let end = Instant::now();
        assert!(end.duration_since(start).subsec_millis() >= 50);
    }

    #[test]
    fn repeat_behavior_repeats_a_message() {
        let mut stub = from_json(r#"{
                     "predicates": [],
                     "responses": [
                        { "id": "0x01", "data": [ "0x17" ], "_behaviors": [ { "repeat": 2 } ] },
                        { "id": "0x02", "data": [ "0x17" ] }
                      ]
                   }"#);

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
        let mut stub = from_json(r#"{
                     "predicates": [],
                     "responses": [
                        { "id": "0x01", "data": [], "_behaviors": [ { "drop": true } ] },
                        { "id": "0x02", "data": [] }
                      ]
                   }"#);

        let responses = stub.generate_responses(&CANMessage::new());
        assert_eq!(0, responses.len());
        let response2 = stub.generate_responses(&CANMessage::new())[0];
        assert_eq!(0x02, response2.id);
    }

    #[test]
    fn concat_behavior_concatenates_messages() {
        let mut stub = from_json(r#"{
                     "predicates": [],
                     "responses": [
                        { "id": "0x01", "data": [], "_behaviors": [ { "concat": true } ] },
                        { "id": "0x02", "data": [], "_behaviors": [ { "concat": true } ] },
                        { "id": "0x03", "data": [] },
                        { "id": "0x04", "data": [] }
                      ]
                   }"#);

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
        let mut stub = from_json(r#"{
                     "predicates": [],
                     "responses": [
                        { "id": "0x01", "data": [], "_behaviors": [
                                                        { "concat": true },
                                                        { "repeat": 2 }
                                                     ] },
                        { "id": "0x02", "data": [] },
                        { "id": "0x03", "data": [] }
                      ]
                   }"#);

        let responses1 = stub.generate_responses(&CANMessage::new());
        assert_eq!(2, responses1.len());
        let responses2 = stub.generate_responses(&CANMessage::new());
        assert_eq!(2, responses2.len());
        let responses3 = stub.generate_responses(&CANMessage::new());
        assert_eq!(1, responses3.len());
    }
}
