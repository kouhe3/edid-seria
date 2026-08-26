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
        CtaColorimetry, CtaDataBlock, CtaDataBlockView, CtaExtendedDataBlockView,
        CtaSpeakerAllocation, CtaVendorSpecificBlock, CtaVideoCapability, CtaVideoMode,
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

    let spk = CtaDataBlock {
        tag: 4,
        payload: vec![0xAA, 0xBB],
    };
    assert_eq!(
        spk.view().unwrap(),
        CtaDataBlockView::SpeakerAllocation(CtaSpeakerAllocation {
            raw_mask: [0xAA, 0xBB, 0],
        })
    );
    let color = CtaDataBlock {
        tag: 7,
        payload: vec![0x05, 0x80, 0x01],
    };
    assert!(matches!(
        color.view().unwrap(),
        CtaDataBlockView::Extended(CtaExtendedDataBlockView::Colorimetry(CtaColorimetry {
            bt2020_rgb: true,
            ..
        }))
    ));

    let vcap = CtaDataBlock {
        tag: 7,
        payload: vec![0x00, 0xC0],
    };
    assert!(matches!(
        vcap.view().unwrap(),
        CtaDataBlockView::Extended(CtaExtendedDataBlockView::VideoCapability(
            CtaVideoCapability {
                selectable_quantization_range_rgb: true,
                selectable_quantization_range_ycc: true,
                ..
            }
        ))
    ));

    let freesync = CtaDataBlock {
        tag: 3,
        payload: vec![0x1A, 0x00, 0x00, 0x01, 48, 144, 0x01],
    };
    assert!(matches!(
        freesync.view().unwrap(),
        CtaDataBlockView::VendorSpecific(CtaVendorSpecificBlock::AmdFreeSync {
            min_refresh_hz: Some(48),
            max_refresh_hz: Some(144),
            ..
        })
    ));

    let unknown = CtaDataBlock {
        tag: 5,
        payload: vec![0xAA, 0xBB],
    };
    assert_eq!(
        unknown.view().unwrap(),
        CtaDataBlockView::Unknown {
            tag: 5,
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
    assert_eq!(parsed.to_bytes(), edid_1080p);
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
    assert_eq!(parsed_multi.to_bytes(), multi_edid);
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
    assert_eq!(parsed_dispid.to_bytes(), displayid_edid);
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
fn extension_validation_distinguishes_standalone_blocks_from_extension_slots() {
    use edid_seria::{EdidBlock, EdidError, serialize_timings};

    let extension_like_base = EdidBlock::new_default();
    assert_eq!(
        EdidBlock::from_bytes_checked(extension_like_base.as_bytes()),
        Ok(extension_like_base.clone())
    );
    assert_eq!(
        extension_like_base.validate_extension(),
        Err(EdidError::InvalidHeader)
    );

    let mut base = EdidBlock::new_default();
    base.raw[126] = 1;
    base.update_checksum();
    let mut complete = Vec::with_capacity(256);
    complete.extend_from_slice(base.as_bytes());
    complete.extend_from_slice(extension_like_base.as_bytes());

    assert_eq!(Edid::from_bytes(&complete), Err(EdidError::InvalidHeader));
    assert!(matches!(
        serialize_timings(Some(&complete), &[]),
        Err(edid_seria::SerializeError::InvalidExistingBlock {
            index: 1,
            source: EdidError::InvalidHeader,
        })
    ));
    assert!(matches!(
        edid_seria::serialize_resolutions_checked(Some(&complete), &[]),
        Err(edid_seria::SerializeError::InvalidExistingBlock {
            index: 1,
            source: EdidError::InvalidHeader,
        })
    ));
}

#[test]
fn extension_lifecycle_rejects_base_blocks_without_mutation() {
    use edid_seria::{EdidBlock, EdidError};

    let valid_extension = EdidBlock::from_cta_data_blocks(3, &[]).unwrap();
    let mut edid = Edid {
        base: EdidBlock::new_default(),
        extensions: vec![valid_extension],
    };
    edid.base.raw[126] = 1;
    edid.base.update_checksum();
    let before = edid.clone();
    let invalid_extension = EdidBlock::new_default();

    assert_eq!(
        edid.insert_extension(1, invalid_extension.clone()),
        Err(EdidError::InvalidHeader)
    );
    assert_eq!(edid, before);
    assert_eq!(
        edid.replace_extension(0, invalid_extension),
        Err(EdidError::InvalidHeader)
    );
    assert_eq!(edid, before);
    edid.extensions[0] = EdidBlock::new_default();
    assert_eq!(edid.validate(), Err(EdidError::InvalidHeader));
}

#[test]
fn extension_move_reorders_existing_indices_atomically() {
    use edid_seria::{CtaDataBlock, EdidError};

    let first = EdidBlock::from_cta_data_blocks(
        3,
        &[CtaDataBlock {
            tag: 2,
            payload: vec![16],
        }],
    )
    .unwrap();
    let second = EdidBlock::from_cta_data_blocks(
        3,
        &[CtaDataBlock {
            tag: 2,
            payload: vec![31],
        }],
    )
    .unwrap();
    let third = EdidBlock::from_cta_data_blocks(
        3,
        &[CtaDataBlock {
            tag: 2,
            payload: vec![34],
        }],
    )
    .unwrap();
    let mut edid = Edid {
        base: EdidBlock::new_default(),
        extensions: vec![first.clone(), second.clone(), third.clone()],
    };
    edid.base.raw[126] = 3;
    edid.base.update_checksum();
    let base_before = edid.base.clone();

    edid.move_extension(0, 2).unwrap();
    assert_eq!(
        edid.extensions,
        vec![second.clone(), third.clone(), first.clone()]
    );
    assert_eq!(edid.base, base_before);

    let before_invalid_source = edid.clone();
    assert_eq!(
        edid.move_extension(3, 0),
        Err(EdidError::ExtensionIndexOutOfRange { index: 3, count: 3 })
    );
    assert_eq!(edid, before_invalid_source);

    let before_invalid_target = edid.clone();
    assert_eq!(
        edid.move_extension(0, 3),
        Err(EdidError::ExtensionIndexOutOfRange { index: 3, count: 3 })
    );
    assert_eq!(edid, before_invalid_target);

    let serialized = edid.to_bytes_checked().unwrap();
    let reparsed = Edid::from_bytes(&serialized).unwrap();
    assert_eq!(reparsed, edid);
    assert_eq!(reparsed.base.raw[126], 3);
}

#[test]
fn dtd_and_metadata_property_roundtrips() {
    use edid_seria::{
        BaseMetadata, ChromaticityCoordinates, ChromaticityPoint, ColorManagementDescriptor,
        Cvt3ByteTimingEntry, CvtAspectRatio, CvtPreferredRate, CvtRangeSupport, CvtSupportedRates,
        DetailedTiming, EstablishedTiming, EstablishedTiming3, EstablishedTimings,
        EstablishedTimings3, MonitorDescriptor, RangeLimitsExtension, SecondaryGtfParameters,
        StandardTiming, StandardTimingAspectRatio, StandardTimingEntry,
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

    // Property: EstablishedTimings3 roundtrips cleanly
    let mut et3 = EstablishedTimings3::default();
    et3.set_timing(EstablishedTiming3::Res1920x1200_60HzRb, true);
    et3.set_timing(EstablishedTiming3::Res1680x1050_60Hz, true);
    let et3_desc = MonitorDescriptor::EstablishedTimings3(et3);
    block.set_monitor_descriptor(1, &et3_desc).unwrap();
    assert_eq!(block.monitor_descriptor(1).unwrap(), Some(et3_desc));

    // Property: Cvt3ByteTimings roundtrips cleanly
    let cvt_desc = MonitorDescriptor::Cvt3ByteTimings([
        Cvt3ByteTimingEntry::Active {
            addressable_lines: 1200,
            aspect_ratio: CvtAspectRatio::Ratio16x10,
            preferred_rate: CvtPreferredRate::Hz60Standard,
            supported_rates: CvtSupportedRates { raw: 0x09 },
        },
        Cvt3ByteTimingEntry::Unused,
        Cvt3ByteTimingEntry::Unused,
        Cvt3ByteTimingEntry::Unused,
    ]);
    block.set_monitor_descriptor(2, &cvt_desc).unwrap();
    assert_eq!(block.monitor_descriptor(2).unwrap(), Some(cvt_desc));

    // Property: ColorManagementDescriptor and RangeLimits with GTF/CVT extensions roundtrip
    let dcm_desc = MonitorDescriptor::ColorManagement(ColorManagementDescriptor {
        revision: 3,
        red_a3: 100,
        red_a2: 200,
        green_a3: 300,
        green_a2: 400,
        blue_a3: 500,
        blue_a2: 600,
    });
    block.set_monitor_descriptor(2, &dcm_desc).unwrap();
    assert_eq!(block.monitor_descriptor(2).unwrap(), Some(dcm_desc));
    let gtf_range = MonitorDescriptor::RangeLimits {
        min_vertical_hz: 50,
        max_vertical_hz: 120,
        min_horizontal_khz: 31,
        max_horizontal_khz: 135,
        max_pixel_clock_mhz: 300,
        extension: RangeLimitsExtension::SecondaryGtf(SecondaryGtfParameters {
            start_horizontal_frequency_khz: 80,
            parameter_c: 80,
            slope_m: 600,
            offset_k: 40,
            scaling_j: 20,
        }),
    };
    block.set_monitor_descriptor(3, &gtf_range).unwrap();
    assert_eq!(block.monitor_descriptor(3).unwrap(), Some(gtf_range));
    assert_eq!(block.validate(), Ok(()));

    let cvt_range = MonitorDescriptor::RangeLimits {
        min_vertical_hz: 48,
        max_vertical_hz: 165,
        min_horizontal_khz: 30,
        max_horizontal_khz: 250,
        max_pixel_clock_mhz: 650,
        extension: RangeLimitsExtension::Cvt(CvtRangeSupport {
            revision: 0x11,
            max_pixel_clock_precision: 0x10,
            max_active_pixels: 2560,
            supported_aspect_ratios: 0xC0,
            preferred_aspect_ratio_and_flags: 0x28,
            scaling_support: 0x00,
            preferred_vertical_rate_hz: 144,
        }),
    };
    block.set_monitor_descriptor(3, &cvt_range).unwrap();
    assert_eq!(block.monitor_descriptor(3).unwrap(), Some(cvt_range));
    assert_eq!(block.validate(), Ok(()));
}

#[test]
fn displayid_typed_encoder_roundtrips_view_and_bytes() {
    use edid_seria::{DisplayIdDataBlockView, DisplayIdDetailedTiming, EdidBlock};

    let timing = DisplayIdDetailedTiming {
        pixel_clock_khz: 14_850,
        h_active: 1_920,
        h_blank: 280,
        h_sync_offset: 88,
        h_sync_width: 44,
        v_active: 1_080,
        v_blank: 45,
        v_sync_offset: 4,
        v_sync_width: 5,
        h_sync_positive: true,
        v_sync_positive: false,
        preferred: true,
    };
    let view = DisplayIdDataBlockView::DetailedTiming {
        timings: vec![timing],
    };
    let data_block = view.to_data_block_with_tag(0x03).unwrap();
    assert_eq!(
        data_block.encode().unwrap(),
        vec![
            0x03, 0x00, 20, 0xCC, 0x05, 0x00, 0x80, 0x7F, 0x07, 0x17, 0x01, 0x57, 0x80, 0x2B, 0x00,
            0x37, 0x04, 0x2C, 0x00, 0x03, 0x00, 0x04, 0x00,
        ]
    );

    let block = EdidBlock::from_display_id_data_blocks(0x20, 2, 0, &[data_block]).unwrap();
    let parsed = block.display_id_data_blocks().unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].encode().unwrap(), block.raw[5..28].to_vec());
    assert_eq!(
        parsed[0].view().unwrap(),
        DisplayIdDataBlockView::DetailedTiming {
            timings: vec![timing],
        }
    );
    assert_eq!(
        block
            .as_bytes()
            .iter()
            .fold(0u8, |sum, &byte| sum.wrapping_add(byte)),
        0
    );
}
#[test]
fn displayid_type_vii_encoder_roundtrips_maximum_pixel_clock() {
    use edid_seria::{DisplayIdDataBlockView, DisplayIdDetailedTiming, EdidBlock};

    let timing = DisplayIdDetailedTiming {
        pixel_clock_khz: 16_777_216,
        h_active: 1,
        h_blank: 1,
        h_sync_offset: 1,
        h_sync_width: 1,
        v_active: 1,
        v_blank: 1,
        v_sync_offset: 1,
        v_sync_width: 1,
        h_sync_positive: false,
        v_sync_positive: false,
        preferred: false,
    };
    let view = DisplayIdDataBlockView::DetailedTiming {
        timings: vec![timing],
    };
    let data_block = view.to_data_block().unwrap();
    assert_eq!(data_block.tag, 0x22);
    assert_eq!(&data_block.payload[..3], &[0xFF, 0xFF, 0xFF][..]);

    let block =
        EdidBlock::from_display_id_data_blocks(0x20, 2, 0, std::slice::from_ref(&data_block))
            .unwrap();
    let parsed = block.display_id_data_blocks().unwrap();
    assert_eq!(
        parsed[0].view().unwrap(),
        DisplayIdDataBlockView::DetailedTiming {
            timings: vec![timing],
        }
    );
    assert_eq!(
        block
            .as_bytes()
            .iter()
            .fold(0u8, |sum, &byte| sum.wrapping_add(byte)),
        0
    );
}

#[test]
fn displayid_typed_timing_encoder_rejects_empty_payload() {
    use edid_seria::{DisplayIdDataBlockView, ExtensionWriteError};

    let view = DisplayIdDataBlockView::DetailedTiming { timings: vec![] };
    assert!(matches!(
        view.to_data_block(),
        Err(ExtensionWriteError::DisplayIdPayloadTooShort {
            tag: 0x22,
            length: 0,
            minimum: 20,
        })
    ));
    assert!(matches!(
        view.to_data_block_with_tag(0x03),
        Err(ExtensionWriteError::DisplayIdPayloadTooShort {
            tag: 0x03,
            length: 0,
            minimum: 20,
        })
    ));
}

#[test]
fn displayid_typed_timing_encoder_rejects_sync_offsets_that_overlap_polarity() {
    use edid_seria::{DisplayIdDataBlockView, DisplayIdDetailedTiming, ExtensionWriteError};

    let mut timing = DisplayIdDetailedTiming {
        pixel_clock_khz: 14_850,
        h_active: 1_920,
        h_blank: 280,
        h_sync_offset: 32_769,
        h_sync_width: 44,
        v_active: 1_080,
        v_blank: 45,
        v_sync_offset: 4,
        v_sync_width: 5,
        h_sync_positive: true,
        v_sync_positive: false,
        preferred: true,
    };
    let view = DisplayIdDataBlockView::DetailedTiming {
        timings: vec![timing],
    };
    assert!(matches!(
        view.to_data_block_with_tag(0x22),
        Err(ExtensionWriteError::InvalidDisplayIdTimingField {
            field: "h_sync_offset",
            value: 32_769,
            maximum: 32_768,
            ..
        })
    ));

    timing.h_sync_offset = 4;
    timing.v_sync_offset = 32_769;
    let view = DisplayIdDataBlockView::DetailedTiming {
        timings: vec![timing],
    };
    assert!(matches!(
        view.to_data_block_with_tag(0x22),
        Err(ExtensionWriteError::InvalidDisplayIdTimingField {
            field: "v_sync_offset",
            value: 32_769,
            maximum: 32_768,
            ..
        })
    ));
}

#[test]
fn cta_header_and_dtd_mutations_update_checksum_and_are_atomic_on_failure() {
    use edid_seria::{CtaDataBlock, EdidBlock};
    let mut block = EdidBlock::from_cta_data_blocks(
        3,
        &[CtaDataBlock {
            tag: 2,
            payload: vec![0x90],
        }],
    )
    .unwrap();
    let original_offset = block.cta_header().unwrap().dtd_offset;
    block.set_cta_capabilities(true, true, false, true).unwrap();
    let header = block.cta_header().unwrap();
    assert_eq!(header.revision, 3);
    assert_eq!(header.dtd_offset, original_offset);
    assert!(header.underscan);
    assert!(header.basic_audio);
    assert!(!header.ycbcr_444);
    assert!(header.ycbcr_422);
    assert_eq!(
        block
            .as_bytes()
            .iter()
            .fold(0u8, |sum, &byte| sum.wrapping_add(byte)),
        0
    );

    let timing = all_presets()[0].clone();
    block
        .replace_cta_detailed_timings(std::slice::from_ref(&timing))
        .unwrap();
    let decoded_timings = block.cta_detailed_timings().unwrap();
    assert_eq!(decoded_timings.len(), 1);
    assert_eq!(decoded_timings[0].h_active, timing.h_active);
    assert_eq!(decoded_timings[0].v_active, timing.v_active);
    assert_eq!(decoded_timings[0].pixel_clock_khz, timing.pixel_clock_khz);
    assert_eq!(
        block
            .as_bytes()
            .iter()
            .fold(0u8, |sum, &byte| sum.wrapping_add(byte)),
        0
    );

    let before_invalid_header = block.clone();
    let mut invalid_header = header;
    invalid_header.dtd_offset = invalid_header.dtd_offset.saturating_add(1);
    assert!(block.set_cta_header(invalid_header).is_err());
    assert_eq!(block, before_invalid_header);

    let before_too_many_dtds = block.clone();
    let too_many = vec![all_presets()[0].clone(); 7];
    assert!(block.replace_cta_detailed_timings(&too_many).is_err());
    assert_eq!(block, before_too_many_dtds);
}

#[test]
fn replace_cta_detailed_timings_treats_zero_offset_as_empty_data_collection() {
    let timing = all_presets()[0].clone();
    let mut block = EdidBlock::from_cta_data_blocks(3, &[]).unwrap();
    block.raw[2] = 0;
    block.update_checksum();

    let expected =
        EdidBlock::from_cta_data_blocks_and_timings(3, &[], std::slice::from_ref(&timing)).unwrap();
    block
        .replace_cta_detailed_timings(std::slice::from_ref(&timing))
        .unwrap();
    assert_eq!(block.as_bytes(), expected.as_bytes());
}

#[test]
fn replace_cta_detailed_timings_preserves_data_blocks_with_zero_offset() {
    use edid_seria::CtaDataBlock;

    let timing = all_presets()[0].clone();
    let data_block = CtaDataBlock {
        tag: 2,
        payload: vec![0x90],
    };
    let mut block = EdidBlock::from_cta_data_blocks(3, std::slice::from_ref(&data_block)).unwrap();
    block.raw[2] = 0;
    block.update_checksum();
    let original_data_block_bytes = block.raw[4..6].to_vec();

    assert_eq!(block.cta_data_blocks().unwrap(), vec![data_block.clone()]);
    block
        .replace_cta_detailed_timings(std::slice::from_ref(&timing))
        .unwrap();

    assert_eq!(&block.raw[4..6], original_data_block_bytes.as_slice());
    assert_eq!(block.cta_data_blocks().unwrap(), vec![data_block]);
    let decoded = block.cta_detailed_timings().unwrap();
    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded[0].h_active, timing.h_active);
    assert_eq!(decoded[0].v_active, timing.v_active);
    assert_eq!(decoded[0].pixel_clock_khz, timing.pixel_clock_khz);
    assert_eq!(
        block
            .as_bytes()
            .iter()
            .fold(0u8, |sum, &byte| sum.wrapping_add(byte)),
        0
    );
}

#[test]
fn set_cta_header_rejects_malformed_layout_without_mutation() {
    use edid_seria::{CtaDataBlock, CtaHeader, ExtensionWriteError};

    let mut block = EdidBlock::from_cta_data_blocks(
        3,
        &[CtaDataBlock {
            tag: 2,
            payload: vec![0x90],
        }],
    )
    .unwrap();
    block.raw[2] = 5;
    block.update_checksum();
    let before = block.as_bytes().to_vec();
    let error = block
        .set_cta_header(CtaHeader {
            revision: 3,
            dtd_offset: 5,
            native_dtd_count: 0,
            underscan: false,
            basic_audio: false,
            ycbcr_444: false,
            ycbcr_422: false,
        })
        .unwrap_err();
    assert!(matches!(
        error,
        ExtensionWriteError::InvalidCtaLayout { .. }
    ));
    assert_eq!(block.as_bytes(), before);
}

#[test]
fn set_cta_header_rejects_native_count_above_populated_dtds_without_mutation() {
    use edid_seria::{CtaHeader, ExtensionWriteError};

    let timing = all_presets()[0].clone();
    let mut block =
        EdidBlock::from_cta_data_blocks_and_timings(3, &[], std::slice::from_ref(&timing)).unwrap();
    let before = block.as_bytes().to_vec();
    let error = block
        .set_cta_header(CtaHeader {
            revision: 3,
            dtd_offset: 4,
            native_dtd_count: 2,
            underscan: false,
            basic_audio: false,
            ycbcr_444: false,
            ycbcr_422: false,
        })
        .unwrap_err();
    assert!(matches!(
        error,
        ExtensionWriteError::CtaDtdsTooLong {
            count: 2,
            maximum: 1,
        }
    ));
    assert_eq!(block.as_bytes(), before);
}

#[test]
fn malformed_cta_mutation_errors_are_not_reported_as_not_cta() {
    use edid_seria::{CtaDataBlock, ExtensionWriteError};

    let mut block = EdidBlock::from_cta_data_blocks(
        3,
        &[CtaDataBlock {
            tag: 2,
            payload: vec![0x90],
        }],
    )
    .unwrap();
    block.raw[2] = 5;
    block.update_checksum();
    let error = block
        .set_cta_capabilities(true, false, false, false)
        .unwrap_err();
    assert!(matches!(
        error,
        ExtensionWriteError::InvalidCtaLayout { .. }
    ));

    let before = block.as_bytes().to_vec();
    let timing = all_presets()[0].clone();
    let replace_error = block
        .replace_cta_detailed_timings(std::slice::from_ref(&timing))
        .unwrap_err();
    assert!(matches!(
        replace_error,
        ExtensionWriteError::InvalidCtaLayout { .. }
    ));
    assert_eq!(block.as_bytes(), before);
}
