use std::io::Error;
use can::create_adaptor;
use can::CANMessage;
use can::CANAdaptor;
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

        let adaptor = create_adaptor().expect("Failed to initialize CAN device.");
        loop {
            match adaptor.receive() {
                Ok(message) => self.handle_message(&adaptor, &message),
                Err(errmsg) => { println!("Error: {}", errmsg); break; }
            }
        }
    }

    pub fn handle_message(&mut self, adaptor: &Box<CANAdaptor>, message: &CANMessage) {
        for response in self.responses_to_message(&message) {
            adaptor.send(&response).expect("Failed to send CAN message.");
        }
    }

    pub fn responses_to_message(&mut self, message: &CANMessage) -> Vec<CANMessage> {
        for i in 0..(self.stubs.len()) {
            let stub = &mut self.stubs[i];
            if stub.matches_message(message) {
                return stub.generate_response(message);
            }
        }
        vec![]
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_reponse_from_first_matching_stub() {
        let mut imposter = Imposter::new();
        let stub = Stub::from_str(r#"{
                     "predicates": [{ "msg": { "id": "0x201", "data": [ "0x00" ] } }],
                     "responses": [{ "id": "0x0201", "data": [ "0x12" ] }]
                   }"#).expect("");
        imposter.add_stub(stub);
        let stub = Stub::from_str(r#"{
                     "predicates": [{ "eq": { "id": "0x202" } }],
                     "responses": [{ "id": "0x0202", "data": [ "0x12" ] }]
                   }"#).expect("");
        imposter.add_stub(stub);
        let stub = Stub::from_str(r#"{
                     "predicates": [{ "eq": { "id": "*" } }],
                     "responses": [{ "id": "0xFFFF", "data": [ "0x12" ] }]
                   }"#).expect("");
        imposter.add_stub(stub);

        let mut message = CANMessage::new();
        message.id = 0x0202;

        let responses = imposter.responses_to_message(&message);

        assert_eq!(1, responses.len());
        assert_eq!(0x0202, responses[0].id);
    }

}

