#![no_main]

use edid_seria::{Edid, EdidBlock, serialize_resolutions};
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
});
