#[cfg(test)]
use r2_radio_packet::crc::crc16_ccitt_false;

#[test]
fn test_crc_empty_input() {
    // Returns Initial Value 0xFFFF and doesn't panic
    assert_eq!(crc16_ccitt_false(b""), 0xFFFF);
}

#[test]
fn test_crc_standard_vector_123456789() {
    assert_eq!(crc16_ccitt_false(b"123456789"), 0x29B1);
}

#[test]
fn test_crc_not_ax25_fcs() {
    // Checking whether AX.25 is NOT implemented (0x906E)
    assert_ne!(
        crc16_ccitt_false(b"123456789"),
        0x906E,
        "Checksum matches AX.25 instead of CRC-16/CCITT-FALSE"
    );
}

#[test]
fn test_crc_single_byte_ascii_a() {
    assert_eq!(crc16_ccitt_false(b"A"), 0xB915);
}

#[test]
fn test_crc_single_zero_byte() {
    assert_eq!(crc16_ccitt_false(&[0x00]), 0xE1F0);
}

#[test]
fn test_crc_single_ff_byte() {
    assert_eq!(crc16_ccitt_false(&[0xFF]), 0xFF00);
}

#[test]
fn test_crc_128_bytes_sequence() {
    // Ciąg 128 bajtów o wartościach 0x00..=0x7F
    let data: Vec<u8> = (0x00..=0x7F).collect();
    assert_eq!(crc16_ccitt_false(&data), 0x1800);
}
