//! # r2-radio-packet
//!
//! A panic-free, `#![forbid(unsafe_code)]` packet encoder and decoder
//! implementing 16-bit CRC (CCITT-FALSE) over physical wire-format radio frames.
//!
//! ## Wire Format
//! - Preamble: 5x `0xAA`
//! - Sync byte: `0x7E`
//! - Length: 1 byte (`0..=128`)
//! - Payload: `0..=128` bytes
//! - CRC: 2 bytes, Big-Endian (`CRC-16/CCITT-FALSE` over `[length] + payload`)

#![forbid(unsafe_code)]
pub mod crc;
pub mod packet;

pub use crc::crc16_ccitt_false;
pub use packet::{build, parse, PacketError, RadioPacket};