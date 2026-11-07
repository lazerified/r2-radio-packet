# Answers to Specification Questions

## 1. Why CCITT-FALSE vs AX.25 FCS? What would happen if you used 0x8408?
The original task asked for CRC-16/CCITT-FALSE (polynomial `0x1021`, MSB-first), that said AX.25 FCS uses different math: the reflected polynomial `0x8408`, LSB-first bit order, and inverted output bits. This generates a different checksum. If a receiver expects CCITT-FALSE, an AX.25 FCS frame will fail the check and be dropped as corrupted.
However, the requirements changed to match the EnduroSat II radio, which needs AX.25 FCS directly on the outer frame. I updated the code to use the standard AX.25 lookup table (`FCSTAB`) from RFC 1662 and Dire Wolf, processing bytes LSB-first and inverting the final result (`fcs ^ 0xFFFF`).

## 2. Why is the CRC computed over `length + payload`, and not over `preamble + sync + length + payload`?
The preamble (`5x 0xAA`) and sync byte (`0x7E`) exist only to help the hardware detect a signal and find where the packet starts.
In wireless transmission, radio noise often corrupts or cuts the opening preamble bytes before the receiver picks up the data. The receiver hunts for the `0x7E` marker to find the packet boundary. If the preamble or sync byte were included in the CRC, missing or messed up preamble bits would ruin the entire packet even when the payload arrived correctly. Because of this, the CRC protects only the actual content: the length byte and payload data.

## 3. How did you choose the order of checks in `parse`, and why does it matter?
The checks were arranged from simplest to most expensive to keep the parser quick but also crash-free:
1. Minimum length check (`data.len() < 9`) makes sure the buffer has enough bytes so the code never panics trying to read outside the slice.
2. Preamble & Sync checks (`BadPreamble`, `BadSync`) drops random radio noise without running more resource-heavy checks.
3. Payload limit check (`declared > 128`) stops corrupted length fields right away so they cannot trigger memory errors or oversized buffers.
4. Buffer size check (`LengthMismatch`, `TrailingBytes`) checks whether the actual number of received bytes matches the declared length.
5. CRC verification (`CrcMismatch`). CRC math takes the most CPU work. Running it last ensures we only calculate checksums on safe, complete packets.

## 4. What would you do if data arrived from UART in chunks and you had to resync?
Data from a serial UART port arrives in pieces and can get mixed with noise. To handle this, the receiver should keep an incoming buffer (`Vec<u8>`) and follow a procedure:
1. Append incoming UART bytes to the buffer.
2. Look for the header by scanning through the buffer until finding the `0xAA 0xAA 0xAA 0xAA 0xAA 0x7E` pattern. Discard any noise bytes sitting before this header.
3. Once the sync byte is found, read length byte $N$ at position 6. If the buffer contains fewer than $9 + N$ bytes, do nothing and wait for more UART data to arrive.
4. Once at least $9 + N$ bytes are available, run `parse()`:
  - If `parse()` succeeds, remove said $9 + N$ bytes from the buffer and pass the payload to the higher layer.
   - If `parse()` fails with `CrcMismatch`, do not throw away the whole buffer. Drop the very first byte and start scanning again from the next position. This makes sure we do not miss a valid packet that just happened to start inside the discarded data. One example:

  |Index|0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8-11 | 12-13 |
| ---------- | ------------------------------ | ---------- | ---------- | ---------- | ---------- | ---------- | --------- | ------------ | ------------------ | --------------- |
| Byte (hex) | 0xAA                           | 0xAA       | 0xAA       | 0xAA       | 0xAA       | 0xAA       | 0x7E      | 0x04         | 'p', 'i', 'n', 'g' | 0xC9, 0x8D      |
| Role       | Noise / extra preamble | Preamble 1 | Preamble 2 | Preamble 3 | Preamble 4 | Preamble 5 | Sync byte | Length (N=4) | Payload            | CRC |

When attempting to parse a packet at index 0 the scanner encounters five `0xAA` bytes and an extra `0xAA` byte at index 5, where it expects a sync marker `0x7E` consequently triggering a `BadSync` error.
However, shifting the buffer by one bit it parses starting from index 1. Now it matches five `0xAA` bytes of preamble as well as a correct sync marker, reading the packet correctly.


