#[cfg(test)]
use r2_radio_packet::crc::crc16_ax25;

#[test]
fn test_crc_empty_input() {
    // Returns Initial Value 0xFFFF and doesn't panic
    assert_eq!(crc16_ax25(b""), 0x0000);
}

#[test]
fn test_crc_standard_vector_123456789() {
    assert_eq!(crc16_ax25(b"123456789"), 0x906E);
}

#[test]
fn test_crc_not_ccitt_false_fcs() {
    // Checking whether CCITT-FALSE is NOT implemented
    assert_ne!(
        crc16_ax25(b"123456789"), 0x29B1,
        "Checksum matches AX.25 instead of CRC-16/CCITT-FALSE"
    );
}

#[test]
fn test_crc_single_byte_ascii_a() {
    assert_eq!(crc16_ax25(b"A"), 0xA3F5);
}

#[test]
fn test_crc_single_zero_byte() {
    assert_eq!(crc16_ax25(&[0x00]), 0xF078);
}

#[test]
fn test_crc_single_ff_byte() {
    assert_eq!(crc16_ax25(&[0xFF]), 0xFF00);
}

#[test]
fn test_crc_128_bytes_sequence() {
    // sequence of 128 bytes with values 0x00..=0x7F
    let data: Vec<u8> = (0x00..=0x7F).collect();
    assert_eq!(crc16_ax25(&data), 0xF1B9);
}
