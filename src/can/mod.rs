use std::{fmt, mem};
use serde_derive::*;

#[cfg(not(feature = "pcan"))]
pub mod dummy;
#[cfg(feature = "pcan")]
pub mod peak;
#[cfg(feature = "pcan")]
pub mod pcbusb;


#[repr(C)]  // TODO: this is here because of the Peak library; let's see what happens on Linux...
#[derive(Debug, Copy, Clone, Serialize)]
pub struct CANMessage {
    pub id: u64,  // TODO: this is u64 because of the Peak library...
    #[serde(rename = "type")]
    pub message_type: u8,
    pub length: u8,
    pub data: [u8; 8],
}

impl CANMessage {
    pub fn new() -> CANMessage {
        unsafe { mem::zeroed() }
    }

    pub fn with_content(id: u64, message_type: u8, data: &[u8]) -> CANMessage {
        let mut m = CANMessage::new();
        m.id = id;
        m.message_type = message_type;
        m.length = data.len() as u8;
        for i in 0..data.len() {
            m.data[i] = data[i];
        }
        m
    }
}

impl fmt::Display for CANMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut data_str = String::new();
        for i in 0..(self.length as usize) {
            data_str.push_str(&format!("{:02X} ", self.data[i]));
        }
        write!(f, "ID:{:04X} LEN:{:1X} DATA: {}", self.id, self.length, data_str)
    }
}


pub trait CANAdaptor {
    fn receive(&mut self) -> Result<CANMessage, &'static str>;
    fn send(&mut self, message: &CANMessage) -> Result<(), &'static str>;
}


#[cfg(feature = "pcan")]
pub fn create_adaptor() -> Result<Box<CANAdaptor>, &'static str> {
    self::peak::PeakAdaptor::new()
}

#[cfg(not(feature = "pcan"))]
pub fn create_adaptor() -> Result<Box<CANAdaptor>, &'static str> {
    self::dummy::DummyAdaptor::new()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_message_with_content() {
        let m = CANMessage::with_content(0x101, 7, &[0x20, 0x30]);

        assert_eq!(0x101, m.id);
        assert_eq!(7, m.message_type);
        assert_eq!(2, m.length);
        assert_eq!([0x20, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], m.data);
    }
}
