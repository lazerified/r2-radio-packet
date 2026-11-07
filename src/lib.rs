//! # r2-radio-packet
//!
//! `r2-radio-packet` provides a safe, panic-free frame builder and parser for
//! physical-layer radio packets secured by the AX.25 / HDLC FCS (CRC-16) checksum.
//!
//! ## Wire format
//!
//! Every packet follows the following byte sequence:
//!
//! ```text
//! +-----------+--------+--------+------------------+-----------+
//! | preamble  | sync   | length | payload          | crc       |
//! | 5 x 0xAA  | 0x7E   | 1 byte | 0..=128 bytes    | 2 bytes   |
//! +-----------+--------+--------+------------------+-----------+
//! ```
//!
//! - **Preamble**: Exactly 5 bytes of `0xAA` for receiver clock synchronization.
//! - **Sync**: Fixed marker `0x7E` indicating the start of frame metadata.
//! - **Length**: 1 byte indicating payload size ($0 \le N \le 128$).
//! - **Payload**: $N$ bytes of data.
//! - **CRC**: 16-bit AX.25 FCS computed over `[length] + payload`, stored in Big-Endian.
//!
//! ## Example
//!
//! ```rust
//! use r2_radio_packet::{build, parse, RadioPacket};
//!
//! let tx = build(b"ping").expect("payload is within the 128-byte limit");
//! let rx = parse(&tx).expect("valid frame structure and checksum");
//!
//! assert_eq!(rx.payload, b"ping");
//! assert_eq!(rx.crc, 0xC98D);
//! ```

#![forbid(unsafe_code)]
pub mod crc;
pub mod packet;

pub use crc::crc16_ax25;
pub use packet::{PacketError, RadioPacket, build, parse};
