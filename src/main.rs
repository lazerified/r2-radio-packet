use r2_radio_packet::{build, parse, PacketError};

fn main() {
    let tx_frame = build(b"ping").expect("payload within 128-byte limit");

    match parse(&tx_frame) {
        Ok(packet) => {
            println!("Payload: {:?}", packet.payload);
            println!("CRC: 0x{:04X}", packet.crc);
        }
        Err(PacketError::BadPreamble) | Err(PacketError::BadSync) => {
            // Channel noise ignored
        }
        Err(e) => {
            eprintln!("Packet rejected: {e}");
        }
    }
}
