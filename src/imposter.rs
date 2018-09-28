use pcan::PCAN;

pub fn run() {
    println!("Running an imposter...");

    let pcan = PCAN::new();
    loop {
        if let Some(mut message) = pcan.receive() {
            message.id += 1;
            pcan.send(&message);
        }

    }
}
