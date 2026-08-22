use edid_seria::{Edid, EdidBlock, all_presets, dtd_fits};

#[test]
fn preset_dtds_roundtrip_through_strict_writer() {
    for timing in all_presets() {
        let mut block = EdidBlock::new_default();
        match block.write_detailed_checked(0, timing) {
            Ok(()) => {
                block.update_checksum();
                let decoded = block.read_detailed_with_flags(0).unwrap();
                assert_eq!(decoded.timing.h_active, timing.h_active);
                assert_eq!(decoded.timing.v_active, timing.v_active);
                assert_eq!(block.validate(), Ok(()));
            }
            Err(_) => assert!(!dtd_fits(timing)),
        }
    }
}

#[test]
fn arbitrary_complete_bytes_never_panic_parser() {
    let mut state = 0x9E37_79B9u32;
    for _ in 0..256 {
        let mut bytes = [0u8; 128];
        for byte in &mut bytes {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            *byte = state as u8;
        }
        let _ = Edid::from_bytes(&bytes);
    }
}
