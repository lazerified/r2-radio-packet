# r2-radio-packet
A safe and panic-free Rust crate for encoding and decoding physical-layer radio packets secured by the AX.25 / HDLC CRC-16 (FCS) checksum algorithm.

## Wire format
| preamble | sync | length | payload | crc |
| --- | --- | --- | --- | --- |
| 5 x 0xAA | 1 x 0x7E | 1 byte | 0..=128 bytes | 2 bytes |
- preamble - used to alert the receiver and synchronize bit clocks,
- sync - byte indicating the start of packet framing,
- length - byte indicating payload's size,
- payload - sent data,
- crc - 16-bit AX.25 FCS, computed only over `[length] + payload` and transmitted in Big-Endian


### Detailed structure of the CRC
| Parameter | Value |
| --- | --- |
| Polynomial | `0x8408` (reflected `0x1021`) |
| Initial value | `0xFFFF` |
| Reflect input | yes |
| Reflect output | yes |
| XOR out | `0xFFFF` |
| Check (`"123456789"`) | `0x906E` |
| Bit order | MSB-first |
| Table implementation | [RFC 1662 Appendix C.2](https://www.rfc-editor.org/info/rfc1662/#appendix-C.2) / [Dire Wolf](https://github.com/wb2osz/direwolf) |


### FCS lookup table (`FCSTAB`)
The implementation uses a 256-entry lookup table (`FCSTAB`) based on RFC 1662 and Dire Wolf `fcs_calc`. Instead of calculating polynomial division bit-by-bit (8 iterations per byte), it precomputes the cumulative XOR modifications () for all 256 possible byte states, updating the register in a single constant-time table lookup.


### Hex representation example
Encoding payload b"ping" (0x70, 0x69, 0x6E, 0x67):
```text
AA AA AA AA AA   7E   04   70 69 6E 67   C9 8D
|____________|   |    |    |_________|   |___|
   Preamble     Sync Len     Payload      CRC (over 04 70 69 6E 67)
```

#### How the CRC value `0xC98D` is calculated
- Computed over [length byte] + payload (`0x04`, `0x70`, `0x69`, `0x6E`, `0x67`). Preamble and sync are ignored.
- Initial register = `0xFFFF`.
- For each byte in payload calculate `FCSTAB` lookup a table index and shift register:
    -  table index is a XOR of current FCS and LSB of payload, afterwards do a bitwise AND which ensures the 0-255 index range
    -  Shift register right by 8 bits and XOR with precomputed table value
- After the last loop iteration XOR the output with `0xFFFF` (XOR out = `0xFFFF`)


## Reference Vectors
### CRC-16 AX.25 FCS (`crc16_ax25`)
| Input | CRC |
|-------|-----|
| `""` (empty) | `0x0000` |
| `"123456789"` | `0x906E` |
| `"A"` (`0x41`) | `0xA3F5` |
| `0x00` | `0xF078` |
| `0xFF` | `0xFF00` |
| 128 bytes `0x00..0x7F` | `0xF1B9` |


### Packets (`build` / `parse`)
| Payload | Packet (hexadecimal) | CRC |
| --- | --- | --- |
| "" | `aaaaaaaaaa7e00f078` | `0xF078` |
| "ping" | `aaaaaaaaaa7e0470696e67c98d` | `0xC98D` |
| "123456789" | `aaaaaaaaaa7e0931323334353637383907ff` | `0x07FF`|
| 128x `0x00` | `aaaaaaaaaa7e80` + 128x `00` + `d834` | `0xD834` |


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
cargo fmt --all --check
```