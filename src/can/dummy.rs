use std::thread;
use core::time;
use can::{CANMessage, CANAdaptor};


pub struct DummyAdaptor {
}


impl DummyAdaptor {
    pub fn new() -> Result<Box<CANAdaptor>, &'static str> {
        Ok(Box::new(DummyAdaptor {}))
    }
}


impl CANAdaptor for DummyAdaptor {

    fn receive(&self) -> Result<CANMessage, &'static str> {
        println!("cannot receive message using dummy CANAdaptor; will sleep for one hour");
        thread::sleep(time::Duration::from_secs(3600));
        Err("cannot receive message using dummy CANAdaptor")
    }

    fn send(&self, _message: &CANMessage) -> Result<(), &'static str> {
        Err("cannot send message using dummy CANAdaptor")
    }
}



