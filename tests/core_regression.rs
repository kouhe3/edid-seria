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

#[test]
fn typed_cta_views_preserve_unknown_and_decode_common_blocks() {
    use edid_seria::{CtaDataBlock, CtaDataBlockView, CtaExtendedDataBlockView, CtaVideoMode};

    let video = CtaDataBlock {
        tag: 2,
        payload: vec![0x80 | 16],
    };
    assert_eq!(
        video.view().unwrap(),
        CtaDataBlockView::Video {
            modes: vec![CtaVideoMode {
                vic: 16,
                native: true,
            }],
        }
    );

    let hdr = CtaDataBlock {
        tag: 7,
        payload: vec![0x06, 0x07, 0x01],
    };
    assert!(matches!(
        hdr.view().unwrap(),
        CtaDataBlockView::Extended(CtaExtendedDataBlockView::HdrStaticMetadata { .. })
    ));

    let unknown = CtaDataBlock {
        tag: 3,
        payload: vec![0xAA, 0xBB],
    };
    assert_eq!(
        unknown.view().unwrap(),
        CtaDataBlockView::Unknown {
            tag: 3,
            payload: vec![0xAA, 0xBB],
        }
    );
}

#[test]
fn displayid_views_preserve_unknown_and_parse_embedded_cta() {
    use edid_seria::{
        CtaDataBlockView, DisplayIdDataBlockView, DisplayIdHeader, EdidBlock, ExtensionError,
    };

    let mut block = EdidBlock::new_default();
    block.raw[0] = 0x70;
    block.raw[1] = 0x20;
    block.raw[4] = 0;
    block.raw[2] = 10;
    block.raw[3] = 2;
    block.raw[5..10].copy_from_slice(&[0x20, 1, 2, 0xAA, 0xBB]);
    block.raw[10..15].copy_from_slice(&[0x81, 1, 2, 0x41, 16]);
    block.raw[15] = block.raw[1..15]
        .iter()
        .fold(0u8, |sum, &byte| sum.wrapping_sub(byte));
    block.update_checksum();

    assert_eq!(
        block.display_id_header().unwrap(),
        DisplayIdHeader {
            revision: 0x20,
            payload_length: 10,
            product_type_or_primary_use: 2,
            extension_count: 0,
        }
    );
    let data_blocks = block.display_id_data_blocks().unwrap();
    assert!(matches!(
        data_blocks[0].view().unwrap(),
        DisplayIdDataBlockView::ProductIdentification { raw }
            if raw == vec![0xAA, 0xBB]
    ));
    assert!(matches!(
        data_blocks[1].view().unwrap(),
        DisplayIdDataBlockView::Cta { data_blocks, .. }
            if data_blocks == vec![edid_seria::CtaDataBlock {
                tag: 2,
                payload: vec![16],
            }]
    ));

    let cta_view = data_blocks[1].view().unwrap();
    assert!(matches!(
        cta_view,
        DisplayIdDataBlockView::Cta { data_blocks, .. }
            if matches!(data_blocks[0].view(), Ok(CtaDataBlockView::Video { .. }))
    ));
    block.raw[15] ^= 1;
    assert!(matches!(
        block.display_id_header(),
        Err(ExtensionError::InvalidDisplayIdChecksum { .. })
    ));
}
