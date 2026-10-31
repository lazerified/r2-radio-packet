use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioPacket {
    pub payload: Vec<u8>,
    pub crc: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketError {
    PayloadTooLong { len: usize },
    TooShort { len: usize },
    BadPreamble,
    BadSync,
    LengthMismatch { declared: u8, actual: usize },
    CrcMismatch { expected: u16, actual: u16 },
    TrailingBytes { count: usize },
}

impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PayloadTooLong { len } => write!(f, "Payload too long: {len} bytes (max 128)"),
            Self::TooShort { len } => write!(f, "Packet too short: {len} bytes (min 9)"),
            Self::BadPreamble => write!(f, "Invalid preamble"),
            Self::BadSync => write!(f, "Invalid sync byte"),
            Self::LengthMismatch { declared, actual } => {
                write!(f, "Length mismatch: declared {declared}, got {actual}")
            }
            Self::CrcMismatch { expected, actual } => {
                write!(f, "CRC mismatch: expected 0x{expected:04X}, actual 0x{actual:04X}")
            }
            Self::TrailingBytes { count } => write!(f, "Trailing bytes in packet: {count}"),
        }
    }
}

// State to the compilator that PacketError is a proper error type that should use default Rust error implementation
impl std::error::Error for PacketError {}
