use std::collections::HashMap;
use can::CANMessage;
use utils;


#[derive(Serialize, Deserialize, Debug)]
pub enum Predicate {
    #[serde(rename = "eq")]
    Equals(HashMap<String, String>),
    #[serde(rename = "msg")]
    Message { id: String, data: Vec<String> },
}

impl Predicate {

    pub fn eval(&self, message: &CANMessage) -> bool {
        match self {
            Predicate::Equals(args) => {
                return Predicate::equals(message, args);
            }
            Predicate::Message { id, data } => {
                return Predicate::matches_template(message, id, data);
            }
        }
    }

    pub fn equals(message: &CANMessage, args: &HashMap<String, String>) -> bool {
        if let Some(id) = args.get("id") {
            return Predicate::matches_value(id, message.id);
        } else {
            panic!("invalid args for eq predicate; found {:?}", args);
        }
    }

    pub fn matches_template(message: &CANMessage, id: &String, data: &Vec<String>) -> bool {
        if Predicate::matches_value(id, message.id) == false {
            return false;
        }
        for i in 0..data.len() {
            if Predicate::matches_value(&data[i], message.data[i] as u64) == false {
                return false;
            }
        }
        true
    }

    fn matches_value(pattern: &str, value: u64) -> bool {
        if pattern == "*" {
            return true;
        } else {
            return value == utils::num_from_string_u64(pattern);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use utils;
    use can::CANMessage;

    // this methods is only here to make the return type explicit,
    // which in turn makes the tests a tiny bit more concise
    fn from_json(s: &str) -> Predicate {
        utils::from_json(s)
    }


    #[test]
    fn pattern_matches_value_when_decimal_number_is_equal() {
        assert_eq!(true, Predicate::matches_value("256", 0x0100));
    }

    #[test]
    fn pattern_does_not_match_value_when_decimal_number_is_not_equal() {
        assert_eq!(false, Predicate::matches_value("257", 0x0100));
    }

    #[test]
    fn pattern_matches_value_when_hexadecimal_number_is_equal() {
        assert_eq!(true, Predicate::matches_value("0x0101", 0x0101));
    }

    #[test]
    fn pattern_matches_value_when_pattern_is_asterisk() {
        assert_eq!(true, Predicate::matches_value("*", 0x0101));
    }


    #[test]
    fn matches_if_id_is_equal() {
        let p = from_json(r#"{ "eq": { "id": "0x0101" } }"#);
        let message = CANMessage::with_content(0x0101, 0, &[]);
        assert_eq!(true, p.eval(&message));
    }

    #[test]
    fn does_not_match_if_id_is_not_equal() {
        let p = from_json(r#"{ "eq": { "id": "0x0101" } }"#);
        let message = CANMessage::with_content(0x0102, 0, &[]);
        assert_eq!(false, p.eval(&message));
    }

    #[test]
    fn matches_when_id_and_literal_data_match() {
        let p = from_json(r#"{ "msg": { "id": "0x0101", "data": ["0x01"] } }"#);
        let message = CANMessage::with_content(0x0101, 0, &[0x01]);
        assert_eq!(true, p.eval(&message));
    }

    #[test]
    fn matches_when_id_and_data_with_asterisk_match() {
        let p = from_json(r#"{ "msg": { "id": "0x0101", "data": ["*", "0x02"] } }"#);
        let message = CANMessage::with_content(0x0101, 0, &[0x01, 0x02]);
        assert_eq!(true, p.eval(&message));
    }

    #[test]
    fn does_not_match_when_id_matches_but_data_does_not() {
        let p = from_json(r#"{ "msg": { "id": "0x0101", "data": ["0x02"] } }"#);
        let message = CANMessage::with_content(0x101, 0, &[0x01]);
        assert_eq!(false, p.eval(&message));
    }

    #[test]
    fn does_not_match_when_data_matches_but_id_does_not() {
        let p = from_json(r#"{ "msg": { "id": "0x0102", "data": ["*"] } }"#);
        let message = CANMessage::with_content(0x0101, 0, &[0x01]);
        assert_eq!(false, p.eval(&message));
    }

}
