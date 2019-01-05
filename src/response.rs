use serde_derive::*;
use crate::can::CANMessage;
use crate::utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTemplate {
    id: String,
    data: Vec<String>,
    #[serde(rename = "_behaviors")]
    pub behaviors: Option<Vec<Behavior>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Behavior {
    #[serde(rename = "wait")]    Wait(u64),
    #[serde(rename = "repeat")]  Repeat(usize),
    #[serde(rename = "drop")]    Drop(bool),
    #[serde(rename = "concat")]  Concat(bool),
}


impl ResponseTemplate {
    pub fn generate_response(&self, _message: &CANMessage) -> CANMessage {
        let mut response = CANMessage::new();
        response.id = utils::num_from_string_u64(&self.id);
        response.len = self.data.len() as u8;
        for i in 0..(self.data.len()) {
            response.data[i] = utils::num_from_string_u64(&self.data[i]) as u8;
        }
        response
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::from_json;
    use crate::can::CANMessage;

    #[test]
    fn creates_response_with_hex_id_and_data_from_template() {
        let t: ResponseTemplate = from_json(r#"{ "id": "0x0102", "data": ["0x017", "0x03"] }"#);
        let response = t.generate_response(&CANMessage::new());
        assert_eq!(0x0102, response.id);
        assert_eq!(2, response.len);
        assert_eq!(0x17, response.data[0]);
        assert_eq!(0x03, response.data[1]);
    }

    #[test]
    fn parses_behavior_from_template() {
        let t: ResponseTemplate = from_json(r#"{ "id": "0x0102", "data": ["0x017" ],
                                                 "_behaviors": [ { "wait": 500 } ] }"#);
        assert!(t.behaviors.is_some());
        if let Some(b) = t.behaviors {
            match &b[0] {
                Behavior::Wait(arg) => { assert_eq!(500, *arg); }
                _ => panic!("expected to find wait behavior")
            }
        }
    }
}
