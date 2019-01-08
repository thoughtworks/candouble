use std::thread;
use core::time;
use crate::can::{CANMessage, CANAdaptor};


pub struct DummyAdaptor {}


impl DummyAdaptor {
    pub fn new() -> Result<Box<CANAdaptor>, &'static str> {
        Ok(Box::new(DummyAdaptor {}))
    }
}


impl CANAdaptor for DummyAdaptor {
    fn receive(&mut self) -> Result<CANMessage, &'static str> {
        println!("DummyAdaptor: Waiting");
        thread::sleep(time::Duration::from_secs(5));
        let message = CANMessage::with_content(0x01, 0x01, &[0xCA, 0xFE]);
        println!("DummyAdaptor: Pretending to receive message {}.", message);
        Ok(message)
    }

    fn send(&mut self, message: &CANMessage) -> Result<(), &'static str> {
        println!("DummyAdaptor: Pretending to send message {}.", message);
        Ok(())
    }
}


