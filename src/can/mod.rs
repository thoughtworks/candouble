use std::fmt;
use std::mem;

#[cfg(not(target_os = "macos"))]
pub mod dummy;
#[cfg(target_os = "macos")]
pub mod peak;
#[cfg(target_os = "macos")]
pub mod pcbusb;


#[repr(C)]  // TODO: this is here because of the Peak library; let's see what happens on Linux...
#[derive(Copy, Clone, Debug)]
pub struct CANMessage {
    pub id: u64,  // TODO: this is u64 because of the Peak library...
    pub message_type: u8,
    pub len: u8,
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
        m.len = data.len() as u8;
        for i in 0..data.len() {
            m.data[i] = data[i];
        }
        m
    }
}

impl fmt::Display for CANMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut data_str = String::new();
        for i in 0..(self.len as usize) {
            data_str.push_str(&format!("{:02X} ", self.data[i]));
        }
        write!(f, "ID:{:04X} LEN:{:1X} DATA: {}", self.id, self.len, data_str)
    }
}


pub trait CANAdaptor {
    fn receive(&self) -> Result<CANMessage, &'static str>;
    fn send(&self, message: &CANMessage) -> Result<(), &'static str>;
}


#[cfg(target_os = "macos")]
pub fn create_adaptor() -> Result<Box<CANAdaptor>, &'static str> {
    self::peak::PeakAdaptor::new()
}

#[cfg(not(target_os = "macos"))]
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
        assert_eq!(2, m.len);
        assert_eq!([0x20, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], m.data);
    }
}
