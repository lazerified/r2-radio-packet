# r2-radio-packet

A safe and panic-free Rust crate for encoding and decoding physical-layer radio packets secured with CRC-16/CCITT-FALSE checksum algorithm.

## Wire format

| preamble | sync | length | payload | crc |
| --- | --- | --- | --- | --- |
| 5 x 0xAA | 1 x 0x7E | 1 byte | 0..=128 bytes | 2 bytes |

- preamble - used to alert the receiver and synchronize bit clocks,
- sync - byte indicating the start of packet framing,
- length - byte indicating payload's size,
- payload - sent data,
- crc - 16-bit CCITT-FALSE, computed only over `[length] + payload` and transmitted in Big-Endian

### Detailed structure of the CRC

| Parameter | Value |
| --- | --- |
| Polynomial | `0x1021` |
| Initial value | `0xFFFF` |
| Reflect input | no |
| Reflect output | no |
| XOR out | `0x0000` |
| Check (`"123456789"`) | `0x29B1` |
| Bit order | MSB-first |

### Hex representation example
Encoding payload b"ping" (0x70, 0x69, 0x6E, 0x67):
```text
AA AA AA AA AA   7E   04   70 69 6E 67   EB E1
|____________|   |    |    |_________|   |___|
   Preamble     Sync Len     Payload      CRC (over 04 70 69 6E 67)
```

#### How the CRC value `0xEBE1` is calculated

- Computed over [length byte] + payload (0x04, 0x70, 0x69, 0x6E, 0x67). Preamble and sync are ignored.
- Initial register = 0xFFFF.
- For each byte XOR with high byte (crc ^= byte << 8), then process 8 bits:
  - If MSb is 1 then (crc << 1) ^ 0x1021
  - If MSb is 0 then crc << 1

**Step-by-step register progression:**

Step | Input Byte |	Register State (Hex)
| --- | --- | --- |
Init	|	- | 0xFFFF
1 |	0x04 (length) |	0x24C8
2	| 0x70 ('p')	| 0xDDB1
3	| 0x69 ('i')	| 0xA26F
4	| 0x6E ('n')	| 0x56E1
5	| 0x67 ('g')	| 0xEBE1

_Stored as Big-Endian (high byte 0xEB first, then 0xE1)._

## CRC-16 Reference Vectors
| Input | CRC |
|-------|-----|
| `""` (empty) | `0xFFFF` |
| `"123456789"` | `0x29B1` |
| `"A"` (`0x41`) | `0xB915` |
| `0x00` | `0xE1F0` |
| `0xFF` | `0xFF00` |
| 128 bytes `0x00..0x7F` | `0x1800` |

## Safety measures
- Usage of `#![forbid(unsafe_code)]` attribute means zero `unsafe` blocks, which verifies memory safety, bounds checks, and pointer lifetimes at compile time.
- Panic-free implementation of `parse()` guarantees thorough validation of raw network input

## Usage
Add the crate to your project dependencies:
```toml
[dependencies]
r2-radio-packet = { path = "../r2-radio-packet" }
```

### Encoding and decoding example
```rust
use r2_radio_packet::packet::{build, parse, PacketError};

fn main() {
    // 1. Encoding data to wire-format bytes
    let data = b"ping";
    let tx_frame = build(data).expect("payload is within the 128-byte limit");

    // 2. Receiving and decoding raw bytes
    match parse(&tx_frame) {
        Ok(packet) => {
            println!("Packet received successfully!");
            println!("Payload: {:?}", packet.payload);
            println!("CRC: 0x{:04X}", packet.crc);
        }
        Err(PacketError::BadPreamble) | Err(PacketError::BadSync) => {
            // Ignored background channel noise
        }
        Err(PacketError::CrcMismatch { expected, actual }) => {
            eprintln!("Corrupted packet: expected 0x{expected:04X}, got 0x{actual:04X}");
        }
        Err(e) => {
            eprintln!("Frame rejected: {e}");
        }
    }
}

```
## Running verification and tests

```sh
# Run unit and integration tests
cargo test

# Run clippy analysis
cargo clippy --all-targets -- -D warnings

# Ensure correct code formatting style
cargo fmt --all -- --check
```