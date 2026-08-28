//! EDID parsing and serialization.

#![deny(unsafe_code)]
#![warn(missing_docs)]
//!
//! Platform-independent core: detailed timing model, CVT computation,
//! EDID base-block read/write, and the resolution → EDID serialization
//! pipeline.
//!
//! No OS dependencies; the registry and elevation glue lives in `cru-rs`.

pub mod builder;
pub mod edid;
pub mod error;
pub mod extensions;
pub mod metadata;
pub mod serialize;
pub use builder::{BaseBlockBuilder, BaseBlockError, TimingPlacement};
pub mod timing;
pub use edid::{
    DecodedDtd, DtdFlags, EDID_BLOCK_SIZE, Edid, EdidBlock, MAX_EDID_BLOCKS, MAX_EDID_BYTES,
};
pub use error::{
    DescriptorError, DtdError, DtdField, EdidError, HexError, MetadataError, MetadataWriteError,
    ModelineError, SerializeError,
};
pub use extensions::{
    CtaAudioDescriptor, CtaColorimetry, CtaDataBlock, CtaDataBlockView, CtaExtendedDataBlockView,
    CtaHeader, CtaSpeakerAllocation, CtaVendorSpecificBlock, CtaVideoCapability, CtaVideoMode,
    DisplayIdDataBlock, DisplayIdDataBlockView, DisplayIdDetailedTiming,
    DisplayIdDisplayParameters, DisplayIdHeader, ExtensionError, ExtensionKind,
    ExtensionWriteError,
};
pub use metadata::{
    AdditionalColorPoint, BaseMetadata, ChromaticityCoordinates, ChromaticityPoint,
    ColorManagementDescriptor, Cvt3ByteTimingEntry, CvtAspectRatio, CvtPreferredRate,
    CvtRangeSupport, CvtSupportedRates, EstablishedTiming, EstablishedTiming3, EstablishedTimings,
    EstablishedTimings3, MonitorDescriptor, RangeLimitsExtension, SecondaryGtfParameters,
    StandardTiming, StandardTimingAspectRatio, StandardTimingEntry,
};
pub use serialize::{
    ResolutionSpec, SerializedEdid, TimingKind, serialize_resolutions,
    serialize_resolutions_checked, serialize_timings,
};
pub use timing::{DetailedTiming, TimingFormula, all_presets, compute_cvt, dtd_fits, validate_dtd};
