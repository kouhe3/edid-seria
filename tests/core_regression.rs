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

#[test]
fn extension_kind_exposes_extension_metadata() {
    let mut cta = EdidBlock::new_default();
    cta.raw[0] = 0x02;
    cta.raw[1] = 3;
    cta.update_checksum();
    let cta_kind = cta.extension_kind();
    assert_eq!(cta_kind.cta_revision(), Some(3));
    assert_eq!(cta_kind.display_id_version(), None);
    assert!(matches!(
        cta_kind,
        edid_seria::ExtensionKind::Cta861 { revision: 3 }
    ));

    let mut display_id = EdidBlock::new_default();
    display_id.raw[0] = 0x70;
    display_id.raw[1] = 0x20;
    display_id.update_checksum();
    let display_kind = display_id.extension_kind();
    assert_eq!(display_kind.display_id_version(), Some(0x20));
    assert!(matches!(
        display_kind,
        edid_seria::ExtensionKind::DisplayId { version: 0x20 }
    ));
}
