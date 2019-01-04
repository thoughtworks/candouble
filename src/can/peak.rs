use std::{fmt, mem, ptr};
use libc::{select, fd_set, FD_ZERO, FD_SET};
use can::{CANMessage, CANAdaptor};
use can::pcbusb::*;


impl CANTimestamp {
    pub fn new() -> CANTimestamp {
        unsafe { mem::zeroed() }
    }
}

impl fmt::Display for CANTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}-{:03}: ", self.millis_overflow, self.millis, self.micros)
    }
}


pub struct PeakAdaptor {
    fd: i32,
}


impl PeakAdaptor {
    pub fn new() -> Result<Box<CANAdaptor>, &'static str> {
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
        Ok(Box::new(PeakAdaptor { fd }))
    }

    pub fn drop(&mut self) {
        let status = unsafe { CAN_Uninitialize(PCAN_NONEBUS) };
        log(&format!("Uninitialized all can devices (0x{:x})", status));
        // no return value, if it fails, it fails...
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


impl CANAdaptor for PeakAdaptor {
    fn receive(&self) -> Result<CANMessage, &'static str> {
        let mut fds = self.get_fd_set();
        if unsafe { select(self.fd + 1, &mut fds, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()) } == 0 {
            return Err("select returned 0 (timeout?)");
        }
        let mut message = CANMessage::new();
        let mut timestamp = CANTimestamp::new();
        let status = unsafe { CAN_Read(PCAN_USBBUS1, &mut message, &mut timestamp) };
        if status != PCAN_ERROR_OK {
            return Err("CAN_Read error"); // TODO: maybe include error code
        }
        log(&format!("<< {}", &message));
        Ok(message.clone())
    }

    fn send(&mut self, message: &CANMessage) -> Result<(), &'static str> {
        let status = unsafe { CAN_Write(PCAN_USBBUS1, message) };
        if status != PCAN_ERROR_OK {
            return Err("CAN_Write error"); // TODO: maybe include error code
        }
        log(&format!(">> {}", &message));
        Ok(())
    }

}


fn log(message: &str) {
    println!("{}", message);
}

