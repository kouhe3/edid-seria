#![no_main]
use edid_seria::{
    CtaDataBlock, DisplayIdDataBlock, Edid, EdidBlock, serialize_resolutions,
    serialize_resolutions_checked,
};
use libfuzzer_sys::fuzz_target;

fn assert_checked_output_is_stable(bytes: &[u8]) {
    let reparsed = Edid::from_bytes(bytes).expect("checked EDID output must parse");
    let stable = reparsed
        .to_bytes_checked()
        .expect("reparsed checked EDID must serialize");
    assert_eq!(stable, bytes);
}

fn exercise_cta_typed_writer(block: &EdidBlock) {
    let Ok(raw_blocks) = block.cta_data_blocks() else {
        return;
    };
    let Ok(header) = block.cta_header() else {
        return;
    };
    let mut typed_blocks = Vec::with_capacity(raw_blocks.len());
    for raw in raw_blocks {
        let Ok(view) = raw.view() else {
            continue;
        };
        let Ok(encoded) = view.to_data_block() else {
            continue;
        };
        let reparsed = encoded
            .view()
            .expect("a successfully encoded CTA typed block must decode");
        let reencoded = reparsed
            .to_data_block()
            .expect("a decoded CTA typed block must encode");
        assert_eq!(reencoded, encoded);
        typed_blocks.push(encoded);
    }

    let timings = block.cta_detailed_timings().unwrap_or_default();
    if let Ok(rebuilt) = EdidBlock::from_cta_data_blocks_and_timings(
        header.revision,
        &typed_blocks,
        &timings,
    ) {
        let checked = EdidBlock::from_bytes_checked(rebuilt.as_bytes())
            .expect("CTA writer output must pass block checksum validation");
        assert_eq!(checked.as_bytes(), rebuilt.as_bytes());
    }
}

fn exercise_display_id_typed_writer(block: &EdidBlock) {
    let Ok(header) = block.display_id_header() else {
        return;
    };
    let Ok(raw_blocks) = block.display_id_data_blocks() else {
        return;
    };
    let mut typed_blocks = Vec::with_capacity(raw_blocks.len());
    for raw in raw_blocks {
        let Ok(view) = raw.view() else {
            continue;
        };
        let Ok(encoded) = view.to_data_block() else {
            continue;
        };
        let reparsed = encoded
            .view()
            .expect("a successfully encoded DisplayID typed block must decode");
        let reencoded = reparsed
            .to_data_block()
            .expect("a decoded DisplayID typed block must encode");
        assert_eq!(reencoded, encoded);
        typed_blocks.push(encoded);
    }

    if let Ok(rebuilt) = EdidBlock::from_display_id_data_blocks(
        header.revision,
        header.product_type_or_primary_use,
        header.extension_count,
        &typed_blocks,
    ) {
        let checked = EdidBlock::from_bytes_checked(rebuilt.as_bytes())
            .expect("DisplayID writer output must pass block checksum validation");
        assert_eq!(checked.as_bytes(), rebuilt.as_bytes());
    }
}

fuzz_target!(|data: &[u8]| {
    // 1. Fuzz strict EDID parser and high-level methods.
    if let Ok(edid) = Edid::from_bytes(data) {
        let _ = edid.all_detailed_timings();
        let _ = edid.all_detailed_timings_flagged();
        let _ = edid.monitor_name();
        let _ = edid.serial_number();
        let _ = edid.preferred_timing();
        let unchecked = edid.to_bytes();
        let checked = edid
            .to_bytes_checked()
            .expect("strictly parsed EDID must pass checked serialization");
        assert_eq!(checked, unchecked);
        assert_checked_output_is_stable(&checked);

        // Exercise the currently available extension lifecycle operations.
        if let Some(extension) = edid.extensions.first().cloned() {
            let mut edited = edid.clone();
            let _ = edited.replace_extension(0, extension.clone());
            if let Some(removed) = edited.remove_extension(0) {
                let _ = edited.insert_extension(0, removed);
            }
            if let Ok(output) = edited.to_bytes_checked() {
                assert_checked_output_is_stable(&output);
            }
        }
    }

    // 2. Fuzz individual EdidBlock parser & extension inspectors.
    if let Some(block) = EdidBlock::from_bytes(data) {
        let _ = block.validate();
        let _ = block.metadata();
        let _ = block.chromaticity();
        let _ = block.established_timings();
        let _ = block.standard_timings();
        for slot in 0..4 {
            let _ = block.monitor_descriptor(slot);
        }
        let _ = block.cta_header();
        let _ = block.cta_detailed_timings();
        let _ = block.cta_detailed_timings_flagged();
        exercise_cta_typed_writer(&block);
        if let Ok(cta_blocks) = block.cta_data_blocks() {
            for cta in cta_blocks {
                let _ = cta.view();
            }
        }
        let _ = block.display_id_header();
        exercise_display_id_typed_writer(&block);
        if let Ok(did_blocks) = block.display_id_data_blocks() {
            for did in did_blocks {
                // Exercise DisplayID typed views and raw encoders.
                let _ = did.view();
                let _ = did.encode();
            }
        }
    }

    // Keep both compatibility and checked resolution serializers in the fuzz path.
    let _ = serialize_resolutions(Some(data), &[]);
    if let Ok(serialized) = serialize_resolutions_checked(Some(data), &[]) {
        assert_checked_output_is_stable(&serialized.bytes);
    }

    // 4. Fuzz raw extension constructors and mutation paths.
    let cta_payload = data.iter().copied().take(31).collect::<Vec<_>>();
    let cta = CtaDataBlock {
        tag: data.first().copied().unwrap_or(0) & 0x07,
        payload: cta_payload,
    };
    if let Ok(mut block) = EdidBlock::from_cta_data_blocks(3, &[cta]) {
        block
            .validate_extension()
            .expect("CTA constructor output must validate");
        let _ = block.replace_cta_data_blocks(&[]);
        block
            .validate_extension()
            .expect("CTA replacement output must validate");
        assert_eq!(
            EdidBlock::from_bytes_checked(block.as_bytes())
                .unwrap()
                .as_bytes(),
            block.as_bytes()
        );
    }
    let display_payload = data.iter().copied().take(64).collect::<Vec<_>>();
    let display = DisplayIdDataBlock {
        tag: data.first().copied().unwrap_or(0),
        revision: data.get(1).copied().unwrap_or(0),
        payload: display_payload,
    };
    if let Ok(mut block) = EdidBlock::from_display_id_data_blocks(0x20, 2, 0, &[display]) {
        block
            .validate_extension()
            .expect("DisplayID constructor output must validate");
        let _ = block.replace_display_id_data_blocks(&[]);
        block
            .validate_extension()
            .expect("DisplayID replacement output must validate");
        assert_eq!(
            EdidBlock::from_bytes_checked(block.as_bytes())
                .unwrap()
                .as_bytes(),
            block.as_bytes()
        );
    }
});
