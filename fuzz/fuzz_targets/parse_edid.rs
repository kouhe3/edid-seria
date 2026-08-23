#![no_main]

use edid_seria::{Edid, EdidBlock, serialize_resolutions};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // 1. Fuzz strict EDID parser
    let _ = Edid::from_bytes(data);

    // 2. Fuzz individual EdidBlock parser & extension inspectors
    if let Some(block) = EdidBlock::from_bytes(data) {
        let _ = block.validate();
        let _ = block.metadata();
        let _ = block.chromaticity();
        let _ = block.established_timings();
        let _ = block.standard_timings();
        let _ = block.cta_data_blocks();
        let _ = block.display_id_header();
        let _ = block.display_id_data_blocks();
    }

    // 3. Fuzz resolution serializer with arbitrary existing binary
    let _ = serialize_resolutions(Some(data), &[]);
});
