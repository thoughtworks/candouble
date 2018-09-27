use std::mem;
use std::ptr;
use libc::select;
use libc::fd_set;
use libc::FD_ZERO;
use libc::FD_SET;

const PCAN_NONEBUS: u16 = 0x00;
const PCAN_USBBUS1: u16 = 0x51;

const PCAN_BAUD_500K: u16 = 0x001C;

const PCAN_RECEIVE_EVENT: u8 = 0x03;

const PCAN_ERROR_OK: u32 = 0x0000;


#[repr(C)]
struct TPCANMsg {
    id: u32,
    // !< 11/29-bit message identifier
    msgtype: u8,
    // !< Type of the message
    len: u8,
    // !< Data Length Code of the message (0..8)
    data: [u8; 8], // !< Data of the message (DATA[0]..DATA[7])
}


#[repr(C)]
struct TPCANTimestamp
{
    millis: u32,
    // !< Base-value: milliseconds: 0.. 2^32-1
    millis_overflow: u16,
    // !< Roll-arounds of millis
    micros: u16,             // !< Microseconds: 0..999
}


impl TPCANMsg {
    pub fn new() -> TPCANMsg {
        TPCANMsg {
            id: 0,
            msgtype: 0,
            len: 0,
            data: [0, 0, 0, 0, 0, 0, 0, 0],
        }
    }
}

impl TPCANTimestamp {
    pub fn new() -> TPCANTimestamp {
        TPCANTimestamp {
            millis: 0,
            millis_overflow: 0,
            micros: 0,
        }
    }
}


#[link(name = "PCBUSB")]
extern "C" {
    fn CAN_Initialize(channel: u16, bitrate: u16, hwType: u8, ioPort: u32, interrupt: u16) -> u32;
    fn CAN_Uninitialize(channel: u16) -> u32;

    fn CAN_GetValue(channel: u16, parameter: u8, buffer: &i32, buffer_len: u16) -> u32;
    fn CAN_Read(channel: u16, message_buffer: &TPCANMsg, timestamp_buffer: &TPCANTimestamp) -> u32;
}


pub fn run() {
    println!("Running an imposter...");
    println!("Initialising CAN device...");
    unsafe {
        let status = CAN_Initialize(PCAN_USBBUS1, PCAN_BAUD_500K, 0, 0, 0);
        println!("- status: {}", status);
    }
    println!("Initialising all can devices...");
    let fd: i32 = 0;
    unsafe {
        let status = CAN_GetValue(PCAN_USBBUS1, PCAN_RECEIVE_EVENT, &fd, 4);
        println!("- status: {}", status);
        println!("- fd: {}", fd);
    }

    let mut fdset: fd_set = unsafe { mem::zeroed() };
    let mut fdset2: fd_set = unsafe { mem::zeroed() };
    unsafe {
        FD_ZERO(&mut fdset);
        FD_SET(fd, &mut fdset);
        FD_ZERO(&mut fdset2);
    }

    /*

		printf("  - R ID:%4x LEN:%1x DATA:%02x %02x %02x %02x %02x %02x %02x %02x\n",
				(int) message.ID, (int) message.LEN, (int) message.DATA[0],
				(int) message.DATA[1], (int) message.DATA[2],
				(int) message.DATA[3], (int) message.DATA[4],
				(int) message.DATA[5], (int) message.DATA[6],
				(int) message.DATA[7]);
	}

*/


    while unsafe { select(fd + 1, &mut fdset, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()) } > 0 {
        let mut message = TPCANMsg::new();
        let mut timestamp = TPCANTimestamp::new();
        unsafe {
            let status = CAN_Read(PCAN_USBBUS1, &mut message, &mut timestamp);
            println!("- status: {}", status);
            if status != PCAN_ERROR_OK {
                break;
            }
            println!("- R ID:{:4x}", message.id);
        }


    }
    println!("Uninitialising all can devices...");
    unsafe {
        let status = CAN_Uninitialize(PCAN_NONEBUS);
        println!("- status: {}", status);
    }
}
