use pcan::PCAN;
use stub::Stub;
use std::io::Error;


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
        println!("Adding stub: {:?}", stub);
        self.stubs.push(stub);
        Ok(())
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

}
