// TODO: DOCUMENTATION

#![forbid(unsafe_code)]
pub mod crc;
pub mod packet;


pub use crc::crc16_ax25;
pub use packet::{build, parse, PacketError, RadioPacket};