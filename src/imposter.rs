use pcan::PCAN;

pub fn run() {
    println!("Running an imposter...");

    let pcan = PCAN::new().expect("Failed to initialize CAN device.");
    loop {
        if let Ok(mut message) = pcan.receive() {
            message.id += 1;
            pcan.send(&message);
        }

    }
}
