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
        if let Some(num) = utils::num_from_string_u64(pattern) {
            return value == num;
        } else if pattern == "*" {
            return true;
        } else {
            return false;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

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

    // TODO: consider moving some of the tests from Stub over here
}
