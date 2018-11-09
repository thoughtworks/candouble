
pub fn num_from_string_u64(string: &str) -> Option<u64> {
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
