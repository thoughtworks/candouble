use std::io::Read;
use std::fs::File;
use serde_json;
use predicate::Predicate;
use can::CANMessage;
use utils;


#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseTemplate {
    id: Option<String>,
    data: Option<Vec<String>>,
}

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
        let response = &self.def.responses[self.response_idx];
        self.response_idx = (self.response_idx + 1) % self.def.responses.len();
        self.generate_response_from_template(response, message)
    }

    fn generate_response_from_template(&self, template: &ResponseTemplate, _message: &CANMessage) -> CANMessage {
        let mut response = CANMessage::new();
        if let Some(ref response_id) = template.id {
            if let Some(id) = utils::num_from_string_u64(response_id) {
                response.id = id;
            }
        }
        if let Some(ref response_data) = template.data {
            response.len = response_data.len() as u8;
            for i in 0..(response_data.len()) {
                if let Some(val) = utils::num_from_string_u64(&response_data[i]) {
                    response.data[i] = val as u8;
                }
            }
        }
        response
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use can::CANMessage;

    #[test]
    fn matches_if_id_is_equal() {
        let stub = Stub::from_str(r#"{
                     "predicates": [{ "eq": { "id": "0x0101" } }],
                     "responses": []
                   }"#).expect("");
        let message = CANMessage::with_content(0x0101, 0, &[]);
        assert_eq!(true, stub.matches_message(&message));
    }

    #[test]
    fn does_not_match_if_id_is_not_equal() {
        let stub = Stub::from_str(r#"{
                     "predicates": [{ "eq": { "id": "0x0101" } }],
                     "responses": []
                   }"#).expect("");
        let message = CANMessage::with_content(0x0102, 0, &[]);
        assert_eq!(false, stub.matches_message(&message));
    }

    #[test]
    fn matches_when_id_and_literal_data_match() {
        let stub = Stub::from_str(r#"{
                     "predicates": [{ "msg": { "id": "0x0101", "data": ["0x01"] } }],
                     "responses": []
                   }"#).expect("");
        let message = CANMessage::with_content(0x0101, 0, &[0x01]);
        assert_eq!(true, stub.matches_message(&message));
    }

    #[test]
    fn matches_when_id_and_data_with_asterisk_match() {
        let stub = Stub::from_str(r#"{
                     "predicates": [{ "msg": { "id": "0x0101", "data": ["*", "0x02"] } }],
                     "responses": []
                   }"#).expect("");
        let message = CANMessage::with_content(0x0101, 0, &[0x01, 0x02]);
        assert_eq!(true, stub.matches_message(&message));
    }

    #[test]
    fn does_not_match_when_id_matches_but_data_does_not() {
        let stub = Stub::from_str(r#"{
                     "predicates": [{ "msg": { "id": "0x0101", "data": ["0x02"] } }],
                     "responses": []
                   }"#).expect("");
        let message = CANMessage::with_content(0x101, 0, &[0x01]);
        assert_eq!(false, stub.matches_message(&message));
    }

    #[test]
    fn does_not_match_when_data_matches_but_id_does_not() {
        let stub = Stub::from_str(r#"{
                     "predicates": [{ "msg": { "id": "0x0102", "data": ["*"] } }],
                     "responses": []
                   }"#).expect("");
        let message = CANMessage::with_content(0x0101, 0, &[0x01]);
        assert_eq!(false, stub.matches_message(&message));
    }

    #[test]
    fn creates_response_with_hex_id_and_data_from_template() {
        let mut stub = Stub::from_str(r#"{
                     "predicates": [],
                     "responses": [{ "id": "0x0102", "data": [ "0x17", "SDF", "0x03" ] }]
                   }"#).expect("");

        let response = stub.generate_response(&CANMessage::new());

        assert_eq!(0x0102, response.id);
        assert_eq!(3, response.len);
        assert_eq!(0x17, response.data[0]);
        assert_eq!(0, response.data[1]); // TODO: skips unparsable numbers
        assert_eq!(0x03, response.data[2]);
    }

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
