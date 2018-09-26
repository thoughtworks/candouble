const PCAN_NONEBUS: u16 = 0x00;
const PCAN_USBBUS1: u16 = 0x51;

const PCAN_BAUD_500K: u16 = 0x001C;

const PCAN_RECEIVE_EVENT: u8 = 0x03;


#[link(name="PCBUSB")]
extern "C" {
    fn CAN_Initialize(channel: u16, bitrate: u16, hwType: u8, ioPort: u32, interrupt: u16) -> u32;
    fn CAN_Uninitialize(channel: u16) -> u32;

    fn CAN_GetValue(channel: u16, parameter: u8, buffer: &u32, buffer_len: u16) -> u32;
}


pub fn run() {
    println!("Running an imposter...");
    println!("Initialising CAN device...");
    unsafe {
        let status = CAN_Initialize(PCAN_USBBUS1, PCAN_BAUD_500K, 0, 0, 0);
        println!("- status: {}", status);
    }
    println!("Uninitialising all can devices...");
    unsafe {
        let fd: u32 = 0;
        let status = CAN_GetValue(PCAN_USBBUS1, PCAN_RECEIVE_EVENT, &fd, 4);
        println!("- status: {}", status);
        println!("- fd: {}", fd);
    }
    println!("Uninitialising all can devices...");
    unsafe {
        let status = CAN_Uninitialize(PCAN_NONEBUS);
        println!("- status: {}", status);
    }
}
