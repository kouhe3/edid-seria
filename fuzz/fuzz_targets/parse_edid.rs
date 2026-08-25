#![no_main]
use edid_seria::{
    CtaDataBlock, DisplayIdDataBlock, Edid, EdidBlock, serialize_resolutions,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // 1. Fuzz strict EDID parser and high-level methods
    if let Ok(edid) = Edid::from_bytes(data) {
        let _ = edid.all_detailed_timings();
        let _ = edid.all_detailed_timings_flagged();
        let _ = edid.monitor_name();
        let _ = edid.serial_number();
        let _ = edid.preferred_timing();
        let _ = edid.to_bytes();
        let _ = edid.to_hex();
    }

    // 2. Fuzz individual EdidBlock parser & extension inspectors
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
        if let Ok(cta_blocks) = block.cta_data_blocks() {
            for cta in cta_blocks {
                let _ = cta.view();
            }
        }
        let _ = block.display_id_header();
        if let Ok(did_blocks) = block.display_id_data_blocks() {
            for did in did_blocks {
                let _ = did.view();
            }
        }
    }

    // 3. Fuzz resolution serializer with arbitrary existing binary
    let _ = serialize_resolutions(Some(data), &[]);
    // 4. Fuzz raw extension constructors and mutation paths.
    let cta_payload = data.iter().copied().take(31).collect::<Vec<_>>();
    let cta = CtaDataBlock {
        tag: data.first().copied().unwrap_or(0) & 0x07,
        payload: cta_payload,
    };
    if let Ok(mut block) = EdidBlock::from_cta_data_blocks(3, &[cta]) {
        let _ = block.replace_cta_data_blocks(&[]);
    }
    let display_payload = data.iter().copied().take(64).collect::<Vec<_>>();
    let display = DisplayIdDataBlock {
        tag: data.first().copied().unwrap_or(0),
        revision: data.get(1).copied().unwrap_or(0),
        payload: display_payload,
    };
    if let Ok(mut block) = EdidBlock::from_display_id_data_blocks(0x20, 2, 0, &[display]) {
        let _ = block.replace_display_id_data_blocks(&[]);
    }

    if let Ok(mut edid) = Edid::from_bytes(data) {
        if let Some(extension) = edid.extensions.first().cloned() {
            let _ = edid.replace_extension(0, extension);
        }
        let _ = edid.to_bytes_checked();
    }
    let _ = serialize_resolutions(Some(data), &[]);
});
