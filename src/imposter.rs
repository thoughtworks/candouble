use std::io::Error;
use can::create_adaptor;
use can::CANMessage;
use stub::Stub;


pub struct Imposter {
    stubs: Vec<Stub>,
}


impl Imposter {
    pub fn new() -> Imposter {
        Imposter {
            stubs: Vec::new(),
        }
    }


    pub fn load_stub(&mut self, filename: &str) -> Result<(), Error> {
        println!("Reading stub from file: {}", filename);
        let stub = Stub::from_file(filename).expect("Failed to parse JSON");
        self.add_stub(stub);
        Ok(())
    }

    fn add_stub(&mut self, stub: Stub) {
        println!("Adding stub: {:?}", stub);
        self.stubs.push(stub);
    }


    pub fn run(&mut self) {
        println!("Running an imposter...");

        let pcan = create_adaptor().expect("Failed to initialize CAN device.");
        loop {
            if let Ok(message) = pcan.receive() {
                if let Some(response) = self.response_for_message(&message) {
                    pcan.send(&response).expect("Failed to send CAN message.");
                } else {
                    println!("No stub for message.");
                }

            }
        }
    }


    pub fn response_for_message(&self, message: &CANMessage) -> Option<CANMessage> {
        for i in 0..(self.stubs.len()) {
            let s = &self.stubs[i];
            if s.matches_message(message) {
                return Some(s.generate_response(message));
            }
        }
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_reponse_from_first_matching_stub() {
        let mut imposter = Imposter::new();
        let stub = Stub::from_str(r#"{
                     "match": { "id": "0x201" },
                     "response": { "id": "0x0201", "data": [ "0x12" ] }
                   }"#).expect("");
        imposter.add_stub(stub);
        let stub = Stub::from_str(r#"{
                     "match": { "id": "0x202" },
                     "response": { "id": "0x0202", "data": [ "0x12" ] }
                   }"#).expect("");
        imposter.add_stub(stub);
        let stub = Stub::from_str(r#"{
                     "match": { "id": "*" },
                     "response": { "id": "0xFFFF", "data": [ "0x12" ] }
                   }"#).expect("");
        imposter.add_stub(stub);

        let mut message = CANMessage::new();
        message.id = 0x0202;

        let opt = imposter.response_for_message(&message);

        assert_eq!(true, opt.is_some());
        if let Some(r) = opt {
            assert_eq!(0x0202, r.id);
        }


    }
}

