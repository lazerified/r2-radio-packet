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
                write!(
                    f,
                    "CRC mismatch: expected 0x{expected:04X}, actual 0x{actual:04X}"
                )
            }
            Self::TrailingBytes { count } => write!(f, "Trailing bytes in packet: {count}"),
        }
    }
}

// default Rust error implementation
impl std::error::Error for PacketError {}

use crate::crc::crc16_ax25;

// Builds a radio packet from payload: [preamble x5] [sync x1] [length x1] [payload xN] [crc_be x2]
pub fn build(payload: &[u8]) -> Result<Vec<u8>, PacketError> {
    if payload.len() > 128 {
        return Err(PacketError::PayloadTooLong { len: payload.len() });
    }

    let declared_len = payload.len() as u8;

    let total_len = 5 + 1 + 1 + payload.len() + 2;
    let mut packet = Vec::with_capacity(total_len);

    // preamble (5 x 0xAA) and sync (0x7E)
    packet.extend_from_slice(&[0xAA; 5]);
    packet.push(0x7E);

    // length byte and actual payload
    packet.push(declared_len);
    packet.extend_from_slice(payload);

    // Creating a buffer to calculate CRC: [length] + payload
    let mut crc_input = Vec::with_capacity(1 + payload.len());
    crc_input.push(declared_len);
    crc_input.extend_from_slice(payload);

    let crc = crc16_ax25(&crc_input);

    // CRC written as Big-Endian
    packet.extend_from_slice(&crc.to_be_bytes());

    Ok(packet)
}

pub fn parse(data: &[u8]) -> Result<RadioPacket, PacketError> {
    // Minimum packet length check (5 preamble + 1 sync + 1 length + 0 payload + 2 crc = 9)
    if data.len() < 9 {
        return Err(PacketError::TooShort { len: data.len() });
    }

    // Preamble check (5 x 0xAA)
    if data[0..5] != [0xAA; 5] {
        return Err(PacketError::BadPreamble);
    }

    // Sync marker check (0x7E)
    if data[5] != 0x7E {
        return Err(PacketError::BadSync);
    }

    // Upper bound check (max 128 bytes)
    let declared = data[6];
    if declared > 128 {
        return Err(PacketError::PayloadTooLong {
            len: declared as usize,
        });
    }

    let actual = data.len() - 9;

    // Missing data
    if actual < declared as usize {
        return Err(PacketError::LengthMismatch { declared, actual });
    }

    // Overflow of data
    if actual > declared as usize {
        return Err(PacketError::TrailingBytes {
            count: actual - declared as usize,
        });
    }

    // CRC verification
    let payload_end = 7 + declared as usize;
    let expected_crc = crc16_ax25(&data[6..payload_end]);
    let packet_crc = u16::from_be_bytes([data[payload_end], data[payload_end + 1]]);

    if expected_crc != packet_crc {
        return Err(PacketError::CrcMismatch {
            expected: expected_crc,
            actual: packet_crc,
        });
    }

    // Validated packet
    Ok(RadioPacket {
        payload: data[7..payload_end].to_vec(),
        crc: packet_crc,
    })
}
