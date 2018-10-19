use std::io::Read;
use serde_json;
use std::fs::File;
use pcan::TPCANMessage;


#[derive(Serialize, Deserialize, Debug)]
pub struct Template {
    id: Option<String>,
    data: Option<Vec<String>>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Stub {
    #[serde(rename = "match")]
    match_template: Template,
    #[serde(rename = "response")]
    response_template: Template,
}

impl Stub {
    pub fn from_str(s: &str) -> Result<Stub, &'static str> {
        let stub: Stub = serde_json::from_str(s).expect("Failed to parse JSON");
        Ok(stub)
    }

    pub fn from_file(filename: &str) -> Result<Stub, &'static str> {
        let mut file = File::open(filename).expect(&format!("Failed to open file {}", filename));
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect(&format!("Failed to read file {}", filename));
        Stub::from_str(&contents)
    }

    pub fn matches_message(&self, message: &TPCANMessage) -> bool {
        if let Some(ref match_id) = self.match_template.id {
            if Stub::matches_value(match_id, message.id) == false {
                return false;
            }
        }
        if let Some(ref match_data) = self.match_template.data {
            for i in 0..match_data.len() {
                if Stub::matches_value(&match_data[i], message.data[i] as u64) == false {
                    return false;
                }
            }
        }
        true
    }

    pub fn generate_response(&self, _message: &TPCANMessage) -> TPCANMessage {
        let mut response = TPCANMessage::new();
        if let Some(ref response_id) = self.response_template.id {
            if let Some(id) = Stub::num_from_string_u64(response_id) {
                response.id = id;
            }
        }
        if let Some(ref response_data) = self.response_template.data {
            response.len = response_data.len() as u8;
            for i in 0..(response_data.len()) {
                if let Some(val) = Stub::num_from_string_u64(&response_data[i]) {
                    response.data[i] = val as u8;
                }
            }
        }
        response
    }

    fn matches_value(pattern: &str, value: u64) -> bool {
        if let Some(num) = Stub::num_from_string_u64(pattern) {
            return value == num;
        } else if pattern == "*" {
            return true;
        } else {
            return false;
        }
    }

    fn num_from_string_u64(string: &str) -> Option<u64> {
        if string.starts_with("0x") {
            if let Ok(n) = u64::from_str_radix(&string[2..], 16) {
                return Some(n);
            }
        } else {
            if let Ok(n) = string.parse::<u64>() {
                return Some(n);
            }
        }
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use pcan::TPCANMessage;

    #[test]
    fn loads_stub_from_json_file() {
        let stub = Stub::from_file("tests/stubs/no_match_data.json").expect("Failed to parse JSON");
        assert_eq!(true, stub.match_template.id.is_some());
        assert_eq!("0101", stub.match_template.id.unwrap());
        assert_eq!(None, stub.match_template.data);
        assert_eq!(true, stub.response_template.id.is_some());
        assert_eq!("0102", stub.response_template.id.unwrap());
        assert_eq!("1", stub.response_template.data.unwrap()[0]);
    }

    #[test]
    fn matches_if_literal_id_is_same() {
        let stub = Stub::from_str(r#"{
                     "match": { "id": "0x101" },
                     "response": { }
                   }"#).expect("");

        let mut message = TPCANMessage::new();
        message.id = 0x101;

        assert_eq!(true, stub.matches_message(&message));
    }

    #[test]
    fn matches_if_literal_hex_id_is_same() {
        let stub = Stub::from_str(r#"{
                     "match": { "id": "0x101" },
                     "response": { }
                   }"#).expect("");

        let mut message = TPCANMessage::new();
        message.id = 0x101;

        assert_eq!(true, stub.matches_message(&message));
    }

    #[test]
    fn does_not_match_if_literal_id_is_not_same() {
        let stub = Stub::from_str(r#"{
                     "match": { "id": "0x101" },
                     "response": { }
                   }"#).expect("");

        let mut message = TPCANMessage::new();
        message.id = 0x102;

        assert_eq!(false, stub.matches_message(&message));
    }

    #[test]
    fn matches_if_id_is_asterisk() {
        let stub = Stub::from_str(r#"{
                     "match": { "id": "*" },
                     "response": { }
                   }"#).expect("");

        let mut message = TPCANMessage::new();
        message.id = 0x102;

        assert_eq!(true, stub.matches_message(&message));
    }

    #[test]
    fn matches_if_literal_data_is_the_same() {
        let stub = Stub::from_str(r#"{
                     "match": { "data": ["0x01", "0x02"] },
                     "response": { }
                   }"#).expect("");

        let message = TPCANMessage::with_content(0x101, 0, &[0x01, 0x02, 0x03]);

        assert_eq!(true, stub.matches_message(&message));
    }

    #[test]
    fn does_not_match_if_literal_data_is_not_the_same() {
        let stub = Stub::from_str(r#"{
                     "match": { "data": ["0x01", "0x02"] },
                     "response": { }
                   }"#).expect("");

        let message = TPCANMessage::with_content(0x101, 0, &[0x01, 0x03]);

        assert_eq!(false, stub.matches_message(&message));
    }

    #[test]
    fn matches_if_asterisk_or_literal_data_is_the_same() {
        let stub = Stub::from_str(r#"{
                     "match": { "data": ["*", "0x02"] },
                     "response": { }
                   }"#).expect("");

        let message = TPCANMessage::with_content(0x101, 0, &[0x01, 0x02, 0x03]);

        assert_eq!(true, stub.matches_message(&message));
    }

    #[test]
    fn matches_when_id_and_data_match() {
        let stub = Stub::from_str(r#"{
                     "match": { "id": "0x101", "data": ["*"] },
                     "response": { }
                   }"#).expect("");

        let message = TPCANMessage::with_content(0x101, 0, &[0x01]);

        assert_eq!(true, stub.matches_message(&message));
    }

    #[test]
    fn does_not_match_when_id_matches_but_data_does_not() {
        let stub = Stub::from_str(r#"{
                     "match": { "id": "0x101", "data": ["0x02"] },
                     "response": { }
                   }"#).expect("");

        let message = TPCANMessage::with_content(0x101, 0, &[0x01]);

        assert_eq!(false, stub.matches_message(&message));
    }

    #[test]
    fn does_not_match_when_data_matches_but_id_does_not() {
        let stub = Stub::from_str(r#"{
                     "match": { "id": "0x102", "data": ["*"] },
                     "response": { }
                   }"#).expect("");

        let message = TPCANMessage::with_content(0x101, 0, &[0x01]);

        assert_eq!(false, stub.matches_message(&message));
    }

    #[test]
    fn creates_response_with_hex_id_and_data_from_template() {
        let stub = Stub::from_str(r#"{
                     "match": { "id": "*" },
                     "response": { "id": "0x0102", "data": [ "0x17", "SDF", "0x03" ] }
                   }"#).expect("");

        let response = stub.generate_response(&TPCANMessage::new());

        assert_eq!(0x0102, response.id);
        assert_eq!(3, response.len);
        assert_eq!(0x17, response.data[0]);
        assert_eq!(0, response.data[1]); // TODO: skips unparsable numbers
        assert_eq!(0x03, response.data[2]);
    }
}
