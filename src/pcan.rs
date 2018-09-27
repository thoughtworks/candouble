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

const PCAN_ERROR_OK: u64 = 0x0000;


#[link(name = "PCBUSB")]
extern "C" {
    fn CAN_Initialize(channel: u16, bitrate: u16, hw_type: u8, io_port: u64, interrupt: u16) -> u64;
    fn CAN_Uninitialize(channel: u16) -> u64;
    fn CAN_GetValue(channel: u16, parameter: u8, buffer: &i32, buffer_len: u16) -> u64;
    fn CAN_Read(channel: u16, message_buffer: *mut TPCANMessage, timestamp_buffer: *mut TPCANTimestamp) -> u64;
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TPCANMessage {
    id: u64,
    message_type: u8,
    len: u8,
    data: [u8; 8],
}


impl TPCANMessage {
    pub fn new() -> TPCANMessage {
        unsafe { mem::zeroed() }
    }

    pub fn as_string(&self) -> String {
        let mut strrep = String::new();
        strrep.push_str(&format!("<= ID:{:04X} LEN:{:1X} DATA:", self.id, self.len));
        for i in 0..self.len {
            strrep.push_str(&format!("{:02X} ", self.data[i as usize]));
        }
        strrep
    }
}


#[repr(C)]
pub struct TPCANTimestamp
{
    millis: u64,
    millis_overflow: u16,
    micros: u16,
}

impl TPCANTimestamp {
    pub fn new() -> TPCANTimestamp {
        unsafe { mem::zeroed() }
    }

    pub fn as_string(&self) -> String {
        format!("{}-{}-{:03}: ", self.millis_overflow, self.millis, self.micros)
    }
}


pub struct PCAN {
    fd: i32,
}

impl PCAN {
    pub fn new() -> PCAN {
        let status = unsafe { CAN_Initialize(PCAN_USBBUS1, PCAN_BAUD_500K, 0, 0, 0) };
        log(&format!("Initialized CAN device (0x{:x})", status));
        let fd: i32 = 0;
        let status = unsafe { CAN_GetValue(PCAN_USBBUS1, PCAN_RECEIVE_EVENT, &fd, mem::size_of::<i32>() as u16) };
        log(&format!("Got file descriptor for CAN device (0x{:x})", status));
        PCAN {
            fd
        }
    }

    pub fn drop(&mut self) {
        let status = unsafe { CAN_Uninitialize(PCAN_NONEBUS) };
        log(&format!("Uninitialized all can devices (0x{:x})", status));
    }

    pub fn receive(&self) -> Option<TPCANMessage> {
        let mut fds = self.get_fd_set();
        if unsafe { select(self.fd + 1, &mut fds, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()) } > 0 {
            let mut message = TPCANMessage::new();
            let mut timestamp = TPCANTimestamp::new();
            let status = unsafe { CAN_Read(PCAN_USBBUS1, &mut message, &mut timestamp) };
            if status == PCAN_ERROR_OK {
                let mut output = String::new();
                output.push_str(&timestamp.as_string());
                output.push_str(&message.as_string());
                log(&output);
                return Some(message.clone());
            }
        }
        None
    }

    pub fn send(&self, message: &TPCANMessage) {

    }


    fn get_fd_set(&self) -> fd_set {
        let mut fds: fd_set = unsafe { mem::zeroed() };
        unsafe {
            FD_ZERO(&mut fds);
            FD_SET(self.fd, &mut fds);
        }
        fds
    }

}


fn log(message: &str) {
    println!("{}", message);
}
