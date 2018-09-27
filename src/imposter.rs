use pcan::PCAN;

pub fn run() {
    println!("Running an imposter...");

    let pcan = PCAN::new();
    loop {
        if let Some(message) = pcan.receive() {

        }

    }
}
