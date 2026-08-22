//! EDID parsing and serialization.

#![deny(unsafe_code)]
#![warn(missing_docs)]
//!
//! Platform-independent core: detailed timing model, CVT computation,
//! EDID base-block read/write, and the resolution → EDID serialization
//! pipeline.
//!
//! No OS dependencies; the registry and elevation glue lives in `cru-rs`.

pub mod edid;
pub mod error;
pub mod extensions;
pub mod metadata;
pub mod serialize;
pub mod timing;
pub use edid::{DecodedDtd, DtdFlags, EDID_BLOCK_SIZE, Edid, EdidBlock};
pub use error::{DescriptorError, DtdError, DtdField, EdidError, MetadataError, SerializeError};
pub use extensions::{CtaDataBlock, ExtensionError, ExtensionKind};
pub use metadata::{BaseMetadata, MonitorDescriptor};
pub use serialize::{
    ResolutionSpec, SerializedEdid, TimingKind, serialize_resolutions,
    serialize_resolutions_checked, serialize_timings,
};
pub use timing::{DetailedTiming, TimingFormula, all_presets, compute_cvt, dtd_fits, validate_dtd};
