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
    use edid_seria::{
        CtaDataBlock, CtaDataBlockView, CtaExtendedDataBlockView, CtaVendorSpecificBlock,
        CtaVideoMode,
    };

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

    let vsdb = CtaDataBlock {
        tag: 3,
        payload: vec![0x03, 0x0C, 0x00, 0x10, 0x00],
    };
    assert!(matches!(
        vsdb.view().unwrap(),
        CtaDataBlockView::VendorSpecific(CtaVendorSpecificBlock::Hdmi14b { .. })
    ));

    let unknown = CtaDataBlock {
        tag: 4,
        payload: vec![0xAA, 0xBB],
    };
    assert_eq!(
        unknown.view().unwrap(),
        CtaDataBlockView::Unknown {
            tag: 4,
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

#[test]
fn real_edid_corpus_parses_and_roundtrips_without_loss() {
    // Sample 1: Standard 1080p 60Hz Base Block with Descriptors (Monitor Name "TEST-1080")
    let mut edid_1080p = [0u8; 128];
    edid_1080p[..8].copy_from_slice(&[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]);
    edid_1080p[8..10].copy_from_slice(&[0x04, 0x69]); // "AAA"
    edid_1080p[10..12].copy_from_slice(&[0x01, 0x00]);
    edid_1080p[18] = 0x01;
    edid_1080p[19] = 0x04;
    edid_1080p[20] = 0x80;
    edid_1080p[54..72].copy_from_slice(&[
        0x02, 0x3A, 0x80, 0x18, 0x71, 0x38, 0x2D, 0x40, 0x58, 0x2C, 0x45, 0x00, 0xDD, 0x0C, 0x11,
        0x00, 0x00, 0x1E,
    ]);
    // Monitor name descriptor "TEST-1080" in slot 1
    edid_1080p[72..90].copy_from_slice(&[
        0x00, 0x00, 0x00, 0xFC, 0x00, b'T', b'E', b'S', b'T', b'-', b'1', b'0', b'8', b'0', 0x0A,
        0x20, 0x20, 0x20,
    ]);
    let sum: u8 = edid_1080p[..127]
        .iter()
        .fold(0u8, |acc, &b| acc.wrapping_add(b));
    edid_1080p[127] = (0u8).wrapping_sub(sum);
    let parsed = Edid::from_bytes(&edid_1080p).expect("valid 1080p EDID must parse");
    assert_eq!(parsed.extensions.len(), 0);
    assert_eq!(parsed.base.validate(), Ok(()));
    let decoded_timing = parsed.base.read_detailed(0).unwrap();
    assert_eq!(decoded_timing.h_active, 1920);
    assert_eq!(decoded_timing.v_active, 1080);

    // Sample 2: Multi-block EDID (Base + CTA-861 Extension with Audio/Video/HDR)
    let mut multi_edid = vec![0u8; 256];
    multi_edid[..128].copy_from_slice(&edid_1080p);
    multi_edid[126] = 1; // 1 extension block
    let base_sum: u8 = multi_edid[..127]
        .iter()
        .fold(0u8, |acc, &b| acc.wrapping_add(b));
    multi_edid[127] = (0u8).wrapping_sub(base_sum);

    // Block 1: CTA-861 Rev 3
    multi_edid[128] = 0x02; // Tag: CTA-861
    multi_edid[129] = 0x03; // Rev 3
    multi_edid[130] = 0x04; // DTD offset: 4 (no DTDs, only data blocks)
    multi_edid[131] = 0x00; // Native flags
    let cta_sum: u8 = multi_edid[128..255]
        .iter()
        .fold(0u8, |acc, &b| acc.wrapping_add(b));
    multi_edid[255] = (0u8).wrapping_sub(cta_sum);

    let parsed_multi = Edid::from_bytes(&multi_edid).expect("multi-block EDID must parse");
    assert_eq!(parsed_multi.extensions.len(), 1);
    assert_eq!(parsed_multi.base.validate(), Ok(()));
    assert_eq!(parsed_multi.extensions[0].validate(), Ok(()));
    assert_eq!(
        parsed_multi.extensions[0].extension_kind().cta_revision(),
        Some(3)
    );

    // Sample 3: Multi-block EDID (Base + DisplayID 2.0 Extension)
    let mut displayid_edid = vec![0u8; 256];
    displayid_edid[..128].copy_from_slice(&edid_1080p);
    displayid_edid[126] = 1;
    let base_sum: u8 = displayid_edid[..127]
        .iter()
        .fold(0u8, |acc, &b| acc.wrapping_add(b));
    displayid_edid[127] = (0u8).wrapping_sub(base_sum);

    // Block 1: DisplayID 2.0 with 1 Detailed Timing block
    displayid_edid[128] = 0x70; // Tag: DisplayID
    displayid_edid[129] = 0x20; // Rev 2.0
    displayid_edid[130] = 23; // Payload length
    displayid_edid[131] = 0x02; // Primary use: generic
    displayid_edid[132] = 0x00; // Ext count
    displayid_edid[133..136].copy_from_slice(&[0x22, 0x00, 20]); // Type VII timing block
    displayid_edid[136..139].copy_from_slice(&59_999u32.to_le_bytes()[..3]);
    displayid_edid[139] = 0x80;
    displayid_edid[140..142].copy_from_slice(&1919u16.to_le_bytes());
    displayid_edid[142..144].copy_from_slice(&279u16.to_le_bytes());
    displayid_edid[144..146].copy_from_slice(&(87u16 | 0x8000).to_le_bytes());
    displayid_edid[146..148].copy_from_slice(&43u16.to_le_bytes());
    displayid_edid[148..150].copy_from_slice(&1079u16.to_le_bytes());
    displayid_edid[150..152].copy_from_slice(&44u16.to_le_bytes());
    displayid_edid[152..154].copy_from_slice(&(4u16 | 0x8000).to_le_bytes());
    displayid_edid[154..156].copy_from_slice(&5u16.to_le_bytes());
    // DisplayID section checksum at offset 128 + 5 + 23 = 156
    displayid_edid[156] = (0u8).wrapping_sub(
        displayid_edid[129..156]
            .iter()
            .fold(0u8, |acc, &b| acc.wrapping_add(b)),
    );
    let ext_sum: u8 = displayid_edid[128..255]
        .iter()
        .fold(0u8, |acc, &b| acc.wrapping_add(b));
    displayid_edid[255] = (0u8).wrapping_sub(ext_sum);

    let parsed_dispid =
        Edid::from_bytes(&displayid_edid).expect("DisplayID multi-block must parse");
    assert_eq!(parsed_dispid.extensions.len(), 1);
    assert_eq!(
        parsed_dispid.extensions[0]
            .extension_kind()
            .display_id_version(),
        Some(0x20)
    );
    let dispid_header = parsed_dispid.extensions[0].display_id_header().unwrap();
    assert_eq!(dispid_header.payload_length, 23);
    let db = parsed_dispid.extensions[0]
        .display_id_data_blocks()
        .unwrap();
    assert_eq!(db.len(), 1);
}

#[test]
fn dtd_and_metadata_property_roundtrips() {
    use edid_seria::{
        BaseMetadata, ChromaticityCoordinates, ChromaticityPoint, DetailedTiming,
        EstablishedTiming, EstablishedTimings, StandardTiming, StandardTimingAspectRatio,
        StandardTimingEntry,
    };

    // Property: BaseMetadata roundtrips cleanly on valid inputs
    let mut block = EdidBlock::new_default();
    let meta = BaseMetadata {
        manufacturer_id: String::from("ABC"),
        product_code: 0x1234,
        serial_number: 0x5678_9ABC,
        manufacture_week: 25,
        manufacture_year: 2024,
        input: 0x80,
        horizontal_size_cm: 60,
        vertical_size_cm: 34,
        gamma: Some(220),
        feature_flags: 0x0A,
    };
    block.set_metadata(&meta).unwrap();
    let read_back = block.metadata().unwrap();
    assert_eq!(read_back, meta);

    // Property: Chromaticity roundtrips with exact 10-bit values
    let chrom = ChromaticityCoordinates {
        red: ChromaticityPoint { x: 650, y: 340 },
        green: ChromaticityPoint { x: 300, y: 600 },
        blue: ChromaticityPoint { x: 150, y: 60 },
        white: ChromaticityPoint { x: 313, y: 329 },
    };
    block.set_chromaticity(&chrom).unwrap();
    let read_chrom = block.chromaticity().unwrap();
    assert_eq!(read_chrom, chrom);

    // Property: Established Timings bits are faithfully preserved
    let est = EstablishedTimings::from_raw([0x20, 0x08, 0x00]); // Mode640x480At60 and Mode1024x768At60
    block.set_established_timings(&est).unwrap();
    let read_est = block.established_timings().unwrap();
    assert_eq!(read_est, est);
    assert!(read_est.contains(EstablishedTiming::Mode640x480At60));
    assert!(read_est.contains(EstablishedTiming::Mode1024x768At60));
    assert!(!read_est.contains(EstablishedTiming::Mode800x600At60));

    // Property: Standard Timings roundtrip cleanly
    let std_timings = [
        StandardTimingEntry::Timing(StandardTiming {
            horizontal_pixels: 1920,
            aspect_ratio: StandardTimingAspectRatio::SixteenByNine,
            refresh_rate_hz: 60,
        }),
        StandardTimingEntry::Timing(StandardTiming {
            horizontal_pixels: 1280,
            aspect_ratio: StandardTimingAspectRatio::FiveByFour,
            refresh_rate_hz: 75,
        }),
        StandardTimingEntry::Unused,
        StandardTimingEntry::Unused,
        StandardTimingEntry::Unused,
        StandardTimingEntry::Unused,
        StandardTimingEntry::Unused,
        StandardTimingEntry::Unused,
    ];
    block.set_standard_timings(&std_timings).unwrap();
    let read_std = block.standard_timings().unwrap();
    assert_eq!(read_std, std_timings);
    let timing = DetailedTiming {
        h_active: 2560,
        v_active: 1440,
        h_front: 48,
        h_sync: 32,
        h_back: 80,
        v_front: 3,
        v_sync: 5,
        v_back: 33,
        h_border: 0,
        v_border: 0,
        pixel_clock_khz: 241700,
        h_pol: true,
        v_pol: false,
        v_rate: 60.0,
    };
    block.write_detailed_checked(0, &timing).unwrap();
    let decoded = block.read_detailed(0).unwrap();
    assert_eq!(decoded.h_active, timing.h_active);
    assert_eq!(decoded.v_active, timing.v_active);
    assert_eq!(decoded.pixel_clock_khz, timing.pixel_clock_khz);
    assert_eq!(decoded.h_pol, timing.h_pol);
    assert_eq!(decoded.v_pol, timing.v_pol);
}
