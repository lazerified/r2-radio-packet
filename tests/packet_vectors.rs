use r2_radio_packet::packet::{build, PacketError};

#[test]
fn test_build_empty_payload() {
    let packet = build(b"").expect("failed to build packet");
    let expected = [
        0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0x7E, 0x00, 0xE1, 0xF0,
    ];
    assert_eq!(packet, expected);
}

#[test]
fn test_build_ping_payload() {
    let packet = build(b"ping").expect("failed to build packet");
    let expected = [
        0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0x7E, 0x04, b'p', b'i', b'n', b'g', 0xEB, 0xE1,
    ];
    assert_eq!(packet, expected);
}

#[test]
fn test_build_123456789_payload() {
    let packet = build(b"123456789").expect("failed to build packet");
    let mut expected = vec![0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0x7E, 0x09];
    expected.extend_from_slice(b"123456789");
    expected.extend_from_slice(&[0xF5, 0xF4]);
    assert_eq!(packet, expected);
}

#[test]
fn test_build_128_zeros_payload() {
    let zeros = [0u8; 128];
    let packet = build(&zeros).expect("failed to build packet");
    // Vector: 5x aa, 7e, 80 + 128 zeroes + 3b, c5
    let mut expected = vec![0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0x7E, 0x80];
    expected.extend_from_slice(&zeros);
    expected.extend_from_slice(&[0x3B, 0xC5]);
    assert_eq!(packet, expected);
}

#[test]
fn test_build_payload_too_long() {
    let too_big = [0u8; 129];
    let result = build(&too_big);
    assert_eq!(result, Err(PacketError::PayloadTooLong { len: 129 }));
}
