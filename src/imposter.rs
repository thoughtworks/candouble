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
    id: u64,
    msgtype: u8,
    len: u8,
    data: [u8; 8],
}


#[repr(C)]
struct TPCANTimestamp
{
    millis: u64,
    millis_overflow: u16,
    micros: u16,
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
    fn CAN_Read(channel: u16, message_buffer: *mut TPCANMsg, timestamp_buffer: *mut TPCANTimestamp) -> u32;
}


pub fn run() {
    println!("Running an imposter...");
    unsafe {
        let status = CAN_Initialize(PCAN_USBBUS1, PCAN_BAUD_500K, 0, 0, 0);
        println!("Initialized CAN device (0x{:x})", status);
    }
    let fd: i32 = 0;
    unsafe {
        let status = CAN_GetValue(PCAN_USBBUS1, PCAN_RECEIVE_EVENT, &fd, 4);
        println!("Got file descriptor (0x{:x})", status);
    }

    let mut fdset: fd_set = unsafe { mem::zeroed() };
    let mut fdset2: fd_set = unsafe { mem::zeroed() };
    unsafe {
        FD_ZERO(&mut fdset);
        FD_SET(fd, &mut fdset);
        FD_ZERO(&mut fdset2);
    }

    while unsafe { select(fd + 1, &mut fdset, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()) } > 0 {
        let mut message = TPCANMsg::new();
        let mut timestamp = TPCANTimestamp::new();
        unsafe {
//            let message_rawptr = &mut message as *mut TPCANMsg;
//            let timestamp_rawptr = &mut timestamp as *mut TPCANTimestamp;
            let status = CAN_Read(PCAN_USBBUS1, &mut message, &mut timestamp);
            if status != PCAN_ERROR_OK {
                break;
            }
            print!("- {}-{}-{:03}: ", timestamp.millis_overflow, timestamp.millis, timestamp.micros);
            println!("R ID:{:04X} TYPE:{:X} LEN:{:1X} DATA:{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} ",
                     message.id, message.msgtype, message.len,
                     message.data[0], message.data[1], message.data[2], message.data[3],
                     message.data[4], message.data[5], message.data[6], message.data[7]);
        }


    }
    unsafe {
        let status = CAN_Uninitialize(PCAN_NONEBUS);
        println!("Uninitialized all can devices (0x{:x})", status);
    }
}
