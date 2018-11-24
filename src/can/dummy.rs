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
        Err("cannot receive message using dummy CANAdaptor")
    }

    fn send(&self, _message: &CANMessage) -> Result<(), &'static str> {
        Err("cannot send message using dummy CANAdaptor")
    }
}



