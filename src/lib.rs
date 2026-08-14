//! EDID parsing and serialization for CRU-RS.
//!
//! Platform-independent core: detailed timing model, CVT computation,
//! EDID base-block read/write, and the resolution → EDID serialization
//! pipeline used by the CRU-RS GUI.
//!
//! No OS dependencies; the registry and elevation glue lives in `cru-rs`.

pub mod edid;
pub mod serialize;
pub mod timing;

pub use edid::{EDID_BLOCK_SIZE, EdidBlock};
pub use serialize::{ResolutionSpec, SerializedEdid, TimingKind, serialize_resolutions};
pub use timing::{DetailedTiming, TimingFormula, all_presets, compute_cvt};
