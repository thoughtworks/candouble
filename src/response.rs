use can::CANMessage;
use utils;

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseTemplate {
    id: String,
    data: Vec<String>,
}

impl ResponseTemplate {

    pub fn generate_response(&self, _message: &CANMessage) -> CANMessage {
        let mut response = CANMessage::new();
        if let Some(id) = utils::num_from_string_u64(&self.id) {
            response.id = id;
        }
        response.len = self.data.len() as u8;
        for i in 0..(self.data.len()) {
            if let Some(val) = utils::num_from_string_u64(&self.data[i]) {
                response.data[i] = val as u8;
            }
        }
        response
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use utils::from_json;
    use can::CANMessage;

    #[test]
    fn creates_response_with_hex_id_and_data_from_template() {
        let t: ResponseTemplate = from_json(r#"{ "id": "0x0102", "data": ["0x017", "SDF", "0x03"] }"#);
        let response = t.generate_response(&CANMessage::new());
        assert_eq!(0x0102, response.id);
        assert_eq!(3, response.len);
        assert_eq!(0x17, response.data[0]);
        assert_eq!(0, response.data[1]); // TODO: skips unparsable numbers
        assert_eq!(0x03, response.data[2]);
    }

}
