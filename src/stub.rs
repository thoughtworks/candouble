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

    pub fn matches(&self, message: &TPCANMessage) -> bool {
        if let Some(ref match_id) = self.match_template.id {
            // TODO: the next few lines will not allow also checking data later
            if let Ok(num) = match_id.parse::<u64>() {
                return message.id == num;
            } else if match_id == "*" {
                return true;
            }
        }
        false
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
                     "match": { "id": "0101" },
                     "response": { }
                   }"#).expect("");

        let mut message = TPCANMessage::new();
        message.id = 0101;

        assert_eq!(true, stub.matches(&message));
    }

    #[test]
    fn does_not_match_if_literal_id_is_not_same() {
        let stub = Stub::from_str(r#"{
                     "match": { "id": "0101" },
                     "response": { }
                   }"#).expect("");

        let mut message = TPCANMessage::new();
        message.id = 0102;

        assert_eq!(false, stub.matches(&message));
    }

    #[test]
    fn matches_if_id_is_asterisk() {
        let stub = Stub::from_str(r#"{
                     "match": { "id": "*" },
                     "response": { }
                   }"#).expect("");

        let mut message = TPCANMessage::new();
        message.id = 0102;

        assert_eq!(true, stub.matches(&message));
    }


}
