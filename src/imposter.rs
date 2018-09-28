use pcan::PCAN;
use stub::Stub;
use std::io::Error;
use pcan::TPCANMessage;


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

        let pcan = PCAN::new().expect("Failed to initialize CAN device.");
        loop {
            if let Ok(mut message) = pcan.receive() {
                message.id += 1;
                pcan.send(&message).expect("Failed to send CAN message.");
            }
        }
    }


    pub fn response_for_message(&self, message: &TPCANMessage) -> Option<TPCANMessage> {
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
    fn returns_reponse_from_matching_stub() {
        let mut imposter = Imposter::new();
        let stub = Stub::from_str(r#"{
                     "match": { "id": "*" },
                     "response": { "id": "0x0202", "data": [ "12" ] }
                   }"#).expect("");
        imposter.add_stub(stub);

        let opt = imposter.response_for_message(&TPCANMessage::new());

        assert_eq!(true, opt.is_some());
        if let Some(r) = opt {
            assert_eq!(0x202, r.id);
//            assert_eq!(12, r.data[0]);
        }


    }
}

