use std::fmt;
use std::mem;
use std::ptr;
use libc::select;
use libc::fd_set;
use libc::FD_ZERO;
use libc::FD_SET;

pub mod constants;
use self::constants::*;


#[link(name = "PCBUSB")]
extern "C" {
    fn CAN_Initialize(channel: u16, bitrate: u16, hw_type: u8, io_port: u64, interrupt: u16) -> u64;
    fn CAN_Uninitialize(channel: u16) -> u64;
    fn CAN_GetValue(channel: u16, parameter: u8, buffer: &i32, buffer_len: usize) -> u64;
    fn CAN_Read(channel: u16, message_buffer: *mut TPCANMessage, timestamp_buffer: *mut TPCANTimestamp) -> u64;
    fn CAN_Write(channel: u16, message_buffer: *const TPCANMessage) -> u64;
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TPCANMessage {
    pub id: u64,
    pub message_type: u8,
    pub len: u8,
    pub data: [u8; 8],
}

impl TPCANMessage {
    pub fn new() -> TPCANMessage {
        unsafe { mem::zeroed() }
    }

    pub fn with_content(id: u64, message_type: u8, data: &[u8]) -> TPCANMessage {
        let mut m = TPCANMessage::new();
        m.id = id;
        m.message_type = message_type;
        m.len = data.len() as u8;
        for i in 0..data.len() {
            m.data[i] = data[i];
        }
        m
    }
}

impl fmt::Display for TPCANMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut data_str = String::new();
        for i in 0..(self.len as usize) {
            data_str.push_str(&format!("{:02X} ", self.data[i]));
        }
        write!(f, "ID:{:04X} LEN:{:1X} DATA: {}", self.id, self.len, data_str)
    }
}


#[repr(C)]
pub struct TPCANTimestamp
{
    pub millis: u64,
    pub millis_overflow: u16,
    pub micros: u16,
}

impl TPCANTimestamp {
    pub fn new() -> TPCANTimestamp {
        unsafe { mem::zeroed() }
    }
}

impl fmt::Display for TPCANTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}-{:03}: ", self.millis_overflow, self.millis, self.micros)
    }
}



pub struct PCAN {
    fd: i32,
}

impl PCAN {
    pub fn new() -> Result<PCAN, &'static str> {
        let status = unsafe { CAN_Initialize(PCAN_USBBUS1, PCAN_BAUD_500K, 0, 0, 0) };
        log(&format!("Initialized CAN device (0x{:x})", status));
        if status != PCAN_ERROR_OK {
            return Err("CAN_Initialize error");
        }
        let fd: i32 = 0;
        let status = unsafe { CAN_GetValue(PCAN_USBBUS1, PCAN_RECEIVE_EVENT, &fd, mem::size_of::<i32>()) };
        log(&format!("Got file descriptor for CAN device (0x{:x})", status));
        if status != PCAN_ERROR_OK {
            return Err("CAN_GetValue error when retrieving file descriptor for reading");
        }
        Ok(PCAN { fd })
    }

    pub fn drop(&mut self) {
        let status = unsafe { CAN_Uninitialize(PCAN_NONEBUS) };
        log(&format!("Uninitialized all can devices (0x{:x})", status));
        // no return value, if it fails, it fails...
    }

    pub fn receive(&self) -> Result<TPCANMessage, &'static str> {
        let mut fds = self.get_fd_set();
        if unsafe { select(self.fd + 1, &mut fds, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()) } == 0 {
            return Err("select returned 0 (timeout?)");
        }
        let mut message = TPCANMessage::new();
        let mut timestamp = TPCANTimestamp::new();
        let status = unsafe { CAN_Read(PCAN_USBBUS1, &mut message, &mut timestamp) };
        if status != PCAN_ERROR_OK {
            return Err("CAN_Read error"); // TODO: maybe include error code
        }
        log(&format!("<< {}", &message));
        Ok(message.clone())
    }

    pub fn send(&self, message: &TPCANMessage) -> Result<(), &'static str> {
        let status = unsafe { CAN_Write(PCAN_USBBUS1, message) };
        if status != PCAN_ERROR_OK {
            return Err("CAN_Write error"); // TODO: maybe include error code
        }
        log(&format!(">> {}", &message));
        Ok(())
    }

    fn get_fd_set(&self) -> fd_set {
        let mut fds: fd_set = unsafe { mem::zeroed() };
        unsafe {
            FD_ZERO(&mut fds); // just for the looks, it's zero'd anyway
            FD_SET(self.fd, &mut fds);
        }
        fds
    }
}


fn log(message: &str) {
    println!("{}", message);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_message_with_content() {
        let m = TPCANMessage::with_content(0x101, 7, &[0x20, 0x30]);

        assert_eq!(0x101, m.id);
        assert_eq!(7, m.message_type);
        assert_eq!(2, m.len);
        assert_eq!([0x20, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], m.data);
    }
}
