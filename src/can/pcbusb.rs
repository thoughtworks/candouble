use can::CANMessage;


/* functions defined in libPCBUSB, which we use on the Mac */

extern "C" {
    pub fn CAN_Initialize(channel: u16, bitrate: u16, hw_type: u8, io_port: u64, interrupt: u16) -> u64;
    pub fn CAN_Uninitialize(channel: u16) -> u64;
    pub fn CAN_GetValue(channel: u16, parameter: u8, buffer: &i32, buffer_len: usize) -> u64;
    pub fn CAN_Read(channel: u16, message_buffer: *mut CANMessage, timestamp_buffer: *mut CANTimestamp) -> u64;
    pub fn CAN_Write(channel: u16, message_buffer: *const CANMessage) -> u64;
}


/* types used for arguments to functions (note we use CANMessage from module, too) */

#[repr(C)]
pub struct CANTimestamp
{
    pub millis: u64,
    pub millis_overflow: u16,
    pub micros: u16,
}


/* constants used as arguments to functions */

pub const PCAN_NONEBUS: u16 = 0x00;
pub const PCAN_USBBUS1: u16 = 0x51;

pub const PCAN_RECEIVE_EVENT: u8 = 0x03; // PCAN receive event handler parameter

pub const PCAN_BAUD_1M  : u16 = 0x0014;
pub const PCAN_BAUD_800K: u16 = 0x0016;
pub const PCAN_BAUD_500K: u16 = 0x001C;
pub const PCAN_BAUD_250K: u16 = 0x011C;
pub const PCAN_BAUD_125K: u16 = 0x031C;
pub const PCAN_BAUD_100K: u16 = 0x432F;
pub const PCAN_BAUD_95K : u16 = 0xC34E;
pub const PCAN_BAUD_83K : u16 = 0x852B;
pub const PCAN_BAUD_50K : u16 = 0x472F;
pub const PCAN_BAUD_47K : u16 = 0x1414;
pub const PCAN_BAUD_33K : u16 = 0x8B2F;
pub const PCAN_BAUD_20K : u16 = 0x532F;
pub const PCAN_BAUD_10K : u16 = 0x672F;
pub const PCAN_BAUD_5K  : u16 = 0x7F7F;

pub const PCAN_ERROR_OK          : u64 = 0x00000; // No error
pub const PCAN_ERROR_XMTFULL     : u64 = 0x00001; // Transmit buffer in CAN controller is full
pub const PCAN_ERROR_OVERRUN     : u64 = 0x00002; // CAN controller was read too late
pub const PCAN_ERROR_BUSLIGHT    : u64 = 0x00004; // Bus error: an error counter reached the 'light' limit
pub const PCAN_ERROR_BUSHEAVY    : u64 = 0x00008; // Bus error: an error counter reached the 'heavy' limit
pub const PCAN_ERROR_BUSWARNING  : u64 = PCAN_ERROR_BUSHEAVY; // Bus error: an error counter reached the 'warning' limit
pub const PCAN_ERROR_BUSPASSIVE  : u64 = 0x40000; // Bus error: the CAN controller is error passive
pub const PCAN_ERROR_BUSOFF      : u64 = 0x00010; // Bus error: the CAN controller is in bus-off state
pub const PCAN_ERROR_ANYBUSERR   : u64 = (PCAN_ERROR_BUSWARNING | PCAN_ERROR_BUSLIGHT | PCAN_ERROR_BUSHEAVY | PCAN_ERROR_BUSOFF | PCAN_ERROR_BUSPASSIVE); // Mask for all bus errors
pub const PCAN_ERROR_QRCVEMPTY   : u64 = 0x00020; // Receive queue is empty
pub const PCAN_ERROR_QOVERRUN    : u64 = 0x00040; // Receive queue was read too late
pub const PCAN_ERROR_QXMTFULL    : u64 = 0x00080; // Transmit queue is full
pub const PCAN_ERROR_REGTEST     : u64 = 0x00100; // Test of the CAN controller hardware registers failed (no hardware found)
pub const PCAN_ERROR_NODRIVER    : u64 = 0x00200; // Driver not loaded
pub const PCAN_ERROR_HWINUSE     : u64 = 0x00400; // Hardware already in use by a Net
pub const PCAN_ERROR_NETINUSE    : u64 = 0x00800; // A Client is already connected to the Net
pub const PCAN_ERROR_ILLHW       : u64 = 0x01400; // Hardware handle is invalid
pub const PCAN_ERROR_ILLNET      : u64 = 0x01800; // Net handle is invalid
pub const PCAN_ERROR_ILLCLIENT   : u64 = 0x01C00; // Client handle is invalid
pub const PCAN_ERROR_ILLHANDLE   : u64 = (PCAN_ERROR_ILLHW | PCAN_ERROR_ILLNET | PCAN_ERROR_ILLCLIENT); // Mask for all handle errors
pub const PCAN_ERROR_RESOURCE    : u64 = 0x02000; // Resource (FIFO, Client, timeout) cannot be created
pub const PCAN_ERROR_ILLPARAMTYPE: u64 = 0x04000; // Invalid parameter
pub const PCAN_ERROR_ILLPARAMVAL : u64 = 0x08000; // Invalid parameter value
pub const PCAN_ERROR_UNKNOWN     : u64 = 0x10000; // Unknown error
pub const PCAN_ERROR_ILLDATA     : u64 = 0x20000; // Invalid data, function, or action
pub const PCAN_ERROR_CAUTION     : u64 = 0x2000000; // An operation was successfully carried out, however, irregularities were registered
pub const PCAN_ERROR_INITIALIZE  : u64 = 0x4000000; // Channel is not initialized [Value was changed from 0x40000 to 0x4000000]
pub const PCAN_ERROR_ILLOPERATION: u64 = 0x8000000; // Invalid operation [Value was changed from 0x80000 to 0x8000000]



