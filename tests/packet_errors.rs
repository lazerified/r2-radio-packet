// use r2_radio_packet::packet::{PacketError, build, parse};

// #[test]
// fn test_parse_too_short() {
//     assert_eq!(parse(b""), Err(PacketError::TooShort { len: 0 }));
//     assert_eq!(parse(&[0xAA; 8]), Err(PacketError::TooShort { len: 8 }));
// }

// #[test]
// fn test_parse_bad_preamble() {
//     // first byte is 0xAB instead of 0xAA
//     let mut data = [0xAA; 9];
//     data[0] = 0xAB;
//     data[5] = 0x7E;
//     assert_eq!(parse(&data), Err(PacketError::BadPreamble));
// }

// #[test]
// fn test_parse_bad_sync() {
//     let mut data = [0xAA; 9];
//     data[5] = 0x7F; // wrong sync
//     assert_eq!(parse(&data), Err(PacketError::BadSync));
// }

// #[test]
// fn test_parse_payload_too_long() {
//     let mut data = vec![0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0x7E, 129]; // length 129 > 128
//     data.extend_from_slice(&[0; 129 + 2]);
//     assert_eq!(parse(&data), Err(PacketError::PayloadTooLong { len: 129 }));
// }

// #[test]
// fn test_parse_length_mismatch_underflow() {
//     // declared length of payload (5) > actual length (4)
//     let mut data = vec![0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0x7E, 5];
//     data.extend_from_slice(&[0; 4 + 2]);
//     assert_eq!(
//         parse(&data),
//         Err(PacketError::LengthMismatch {
//             declared: 5,
//             actual: 4,
//         })
//     );
// }

// #[test]
// fn test_parse_trailing_bytes_overflow() {
//     // decl. length of payload (5) < actual length (6)
//     let mut data = vec![0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0x7E, 5];
//     data.extend_from_slice(&[0; 6 + 2]);
//     assert_eq!(parse(&data), Err(PacketError::TrailingBytes { count: 1 }));
// }

// #[test]
// fn test_parse_crc_mismatch() {
//     let mut packet = build(b"ping").expect("failed to build packet");
//     let last_idx = packet.len() - 1;
//     packet[last_idx] ^= 0xFF;
//     assert!(matches!(
//         parse(&packet),
//         Err(PacketError::CrcMismatch { .. })
//     ));
// }

// #[test]
// fn test_parse_crc_little_endian_mismatch() {
//     let mut packet = build(b"ping").expect("failed to build packet");
//     let len = packet.len();
//     packet.swap(len - 2, len - 1);
//     assert!(matches!(
//         parse(&packet),
//         Err(PacketError::CrcMismatch { .. })
//     ));
// }

// #[test]
// fn test_parse_no_panic_on_any_first_byte() {
//     for byte in 0..=255 {
//         let single_byte = [byte];
//         let _ = parse(&single_byte);
//     }
// }
