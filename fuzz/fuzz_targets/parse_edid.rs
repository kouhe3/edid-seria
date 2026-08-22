#![no_main]

use edid_seria::Edid;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = Edid::from_bytes(data);
});
