use std::io::Read;
use std::fs::File;
use serde_json;
use predicate::Predicate;
use response::ResponseTemplate;
use can::CANMessage;


#[derive(Serialize, Deserialize, Debug)]
pub struct StubDefinition {
    predicates: Vec<Predicate>,
    responses: Vec<ResponseTemplate>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Stub {
    def: StubDefinition,
    response_idx: usize,
}

impl Stub {
    pub fn from_str(s: &str) -> Result<Stub, &'static str> {
        let def: StubDefinition = serde_json::from_str(s).expect("Failed to parse JSON");
        Ok(Stub { def, response_idx: 0 })
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
        self.response_idx = (self.response_idx + 1) % self.def.responses.len();
        template.generate_response(message)
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use can::CANMessage;

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

}
