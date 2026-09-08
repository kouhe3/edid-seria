//! Read-only views for EDID extension blocks, including selected CTA data blocks.

use crate::edid::EdidBlock;

mod cta;

pub use cta::{
    CtaAdaptiveSync, CtaAudioDescriptor, CtaAudioFormat, CtaColorimetry, CtaDataBlock,
    CtaDataBlockView, CtaExtendedDataBlockView, CtaHdrDynamicMetadataEntry, CtaHeader,
    CtaSpeakerAllocation, CtaVendorSpecificBlock, CtaVideoCapability, CtaVideoMode, CtaY420Support,
};

/// Recognized kind of an EDID extension block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtensionKind {
    /// CTA-861 extension with its revision.
    Cta861 {
        /// CTA extension revision.
        revision: u8,
    },
    /// DisplayID extension with its version byte.
    DisplayId {
        /// DisplayID version byte.
        version: u8,
    },
    /// Unknown extension tag, preserved as raw bytes.
    Unknown {
        /// Extension tag byte.
        tag: u8,
    },
}

impl ExtensionKind {
    /// Return the CTA-861 revision when this is a CTA extension.
    #[must_use]
    pub const fn cta_revision(self) -> Option<u8> {
        match self {
            Self::Cta861 { revision } => Some(revision),
            _ => None,
        }
    }

    /// Return the DisplayID version when this is a DisplayID extension.
    #[must_use]
    pub const fn display_id_version(self) -> Option<u8> {
        match self {
            Self::DisplayId { version } => Some(version),
            _ => None,
        }
    }

    /// Return the raw extension tag when this is an unknown extension.
    #[must_use]
    pub const fn unknown_tag(self) -> Option<u8> {
        match self {
            Self::Unknown { tag } => Some(tag),
            _ => None,
        }
    }
}

/// Parsed DisplayID base-section header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayIdHeader {
    /// DisplayID structure revision byte.
    pub revision: u8,
    /// Number of bytes occupied by data blocks after the four-byte header.
    pub payload_length: usize,
    /// Product type in DisplayID 1.x, or primary use case in 2.x.
    pub product_type_or_primary_use: u8,
    /// Number of following DisplayID extension sections.
    pub extension_count: u8,
}

/// A DisplayID data block with its header fields and uninterpreted payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayIdDataBlock {
    /// DisplayID data-block tag.
    pub tag: u8,
    /// DisplayID data-block revision.
    pub revision: u8,
    /// Data-block payload without the three-byte header.
    pub payload: Vec<u8>,
}

/// Aspect ratio of a DisplayID Type I/VII detailed timing (byte 3 bits 3:0).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayIdAspectRatio {
    /// 1:1.
    OneToOne,
    /// 5:4.
    FiveToFour,
    /// 4:3.
    FourToThree,
    /// 15:9.
    FifteenToNine,
    /// 16:9.
    SixteenToNine,
    /// 16:10.
    SixteenToTen,
    /// 64:27.
    SixtyFourToTwentySeven,
    /// 256:135.
    TwoHundredFiftySixToOneThirtyFive,
    /// Value 8: derive the ratio from the active image size.
    Calculated,
    /// Reserved value (9..=15), preserved verbatim.
    Reserved(u8),
}

impl DisplayIdAspectRatio {
    const fn from_nibble(value: u8) -> Self {
        match value {
            0 => Self::OneToOne,
            1 => Self::FiveToFour,
            2 => Self::FourToThree,
            3 => Self::FifteenToNine,
            4 => Self::SixteenToNine,
            5 => Self::SixteenToTen,
            6 => Self::SixtyFourToTwentySeven,
            7 => Self::TwoHundredFiftySixToOneThirtyFive,
            8 => Self::Calculated,
            v => Self::Reserved(v),
        }
    }
    const fn nibble(self) -> u8 {
        match self {
            Self::OneToOne => 0,
            Self::FiveToFour => 1,
            Self::FourToThree => 2,
            Self::FifteenToNine => 3,
            Self::SixteenToNine => 4,
            Self::SixteenToTen => 5,
            Self::SixtyFourToTwentySeven => 6,
            Self::TwoHundredFiftySixToOneThirtyFive => 7,
            Self::Calculated => 8,
            Self::Reserved(v) => v & 0x0F,
        }
    }
}

/// Stereoscopic 3D mode of a DisplayID Type I/VII detailed timing (byte 3 bits 6:5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayIdStereo3d {
    /// Mono timing.
    Mono,
    /// 3D stereo timing.
    Stereo3d,
    /// Mono or 3D stereo depending on user action.
    UserAction,
    /// Reserved.
    Reserved,
}

impl DisplayIdStereo3d {
    const fn from_bits(value: u8) -> Self {
        match value & 0x03 {
            0 => Self::Mono,
            1 => Self::Stereo3d,
            2 => Self::UserAction,
            _ => Self::Reserved,
        }
    }
    const fn bits(self) -> u8 {
        match self {
            Self::Mono => 0,
            Self::Stereo3d => 1,
            Self::UserAction => 2,
            Self::Reserved => 3,
        }
    }
}

/// One DisplayID detailed timing entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayIdDetailedTiming {
    /// Pixel clock in kHz.
    pub pixel_clock_khz: u32,
    /// Active horizontal pixels.
    pub h_active: u32,
    /// Horizontal blanking pixels.
    pub h_blank: u32,
    /// Horizontal sync offset in pixels.
    pub h_sync_offset: u32,
    /// Horizontal sync width in pixels.
    pub h_sync_width: u32,
    /// Active vertical lines.
    pub v_active: u32,
    /// Vertical blanking lines.
    pub v_blank: u32,
    /// Vertical sync offset in lines.
    pub v_sync_offset: u32,
    /// Vertical sync width in lines.
    pub v_sync_width: u32,
    /// Horizontal sync polarity.
    pub h_sync_positive: bool,
    /// Vertical sync polarity.
    pub v_sync_positive: bool,
    /// Aspect ratio (byte 3 bits 3:0).
    pub aspect_ratio: DisplayIdAspectRatio,
    /// Whether the timing is interlaced (byte 3 bit 4).
    pub interlaced: bool,
    /// Stereoscopic 3D mode (byte 3 bits 6:5).
    pub stereo_3d: DisplayIdStereo3d,
    /// Preferred timing (byte 3 bit 7, block revision < 2).
    pub preferred: bool,
    /// YCbCr 4:2:0 support (byte 3 bit 7, block revision >= 2).
    pub ycbcr420: bool,
}

/// Typed DisplayID 1.x or 2.x Display Parameters Data Block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayIdDisplayParameters {
    /// Horizontal image size in millimetres.
    pub horizontal_image_size_mm: u16,
    /// Vertical image size in millimetres.
    pub vertical_image_size_mm: u16,
    /// Horizontal native pixel count.
    pub horizontal_pixel_count: u16,
    /// Vertical native pixel count.
    pub vertical_pixel_count: u16,
    /// Display feature flags.
    pub features: u8,
    /// Primary color 1 chromaticity bytes.
    pub primary_color_1: [u8; 3],
    /// Primary color 2 chromaticity bytes.
    pub primary_color_2: [u8; 3],
    /// Primary color 3 chromaticity bytes.
    pub primary_color_3: [u8; 3],
    /// White-point chromaticity bytes.
    pub white_point: [u8; 3],
    /// Maximum luminance for full-screen white.
    pub max_luminance_full: u16,
    /// Maximum luminance for a 10-percent window.
    pub max_luminance_10_percent: u16,
    /// Minimum luminance.
    pub min_luminance: u16,
    /// Color depth and display technology flags.
    pub color_depth_and_technology: u8,
    /// Gamma and EOTF byte.
    pub gamma_eotf: u8,
    /// Original payload bytes.
    pub raw: Vec<u8>,
}

/// DisplayID Dynamic Video Timing Range Limits Data Block (Tag 0x25 in 2.0, Tag 0x09 in 1.x).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayIdDynamicVideoTimingRange {
    /// Minimum pixel clock in kHz (1 kHz .. 16,777,216 kHz).
    pub min_pixel_clock_khz: u32,
    /// Maximum pixel clock in kHz (1 kHz .. 16,777,216 kHz).
    pub max_pixel_clock_khz: u32,
    /// Minimum vertical refresh rate in Hz.
    pub min_vfreq_hz: u8,
    /// Maximum vertical refresh rate in Hz.
    pub max_vfreq_hz: u16,
    /// Seamless dynamic video timing change / VRR support flag.
    pub seamless_dynamic_video_timing: bool,
    /// Original payload bytes.
    pub raw: Vec<u8>,
}

/// DisplayID 1.x Video Timing Range Limits Data Block (Tag 0x09).
///
/// Distinct from the 2.0 [`DisplayIdDynamicVideoTimingRange`] (Tag 0x25):
/// the 1.x block is a fixed 15-byte structure that additionally carries
/// horizontal-frequency, horizontal-blanking and vertical-blanking limits,
/// plus CVT/interlaced/device-class capability flags.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayIdVideoTimingRangeLimits {
    /// Minimum pixel clock in kHz.
    pub min_pixel_clock_khz: u32,
    /// Maximum pixel clock in kHz.
    pub max_pixel_clock_khz: u32,
    /// Minimum horizontal frequency in kHz.
    pub min_hfreq_khz: u8,
    /// Maximum horizontal frequency in kHz.
    pub max_hfreq_khz: u8,
    /// Minimum horizontal blanking in pixels.
    pub min_h_blanking: u16,
    /// Minimum vertical refresh rate in Hz.
    pub min_vfreq_hz: u8,
    /// Maximum vertical refresh rate in Hz.
    pub max_vfreq_hz: u8,
    /// Minimum vertical blanking in lines.
    pub min_v_blanking: u16,
    /// Supports interlaced timings (flag bit 7).
    pub supports_interlaced: bool,
    /// Supports CVT timings (flag bit 6).
    pub supports_cvt: bool,
    /// Supports CVT reduced blanking (flag bit 5).
    pub supports_cvt_reduced_blanking: bool,
    /// Discrete-frequency display device (flag bit 4).
    pub discrete_frequency: bool,
    /// Original payload bytes.
    pub raw: Vec<u8>,
}

/// DisplayID Display Interface Features Data Block (Tag 0x26 in 2.0, Tag 0x0F in 1.x).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayIdInterfaceFeatures {
    /// Color-depth support for RGB encoding (byte 0): bit0=6bpc, bit1=8bpc,
    /// bit2=10bpc, bit3=12bpc, bit4=14bpc, bit5=16bpc.
    pub color_depth_rgb: u8,
    /// Color-depth support for YCbCr 4:4:4 encoding (byte 1).
    pub color_depth_ycbcr444: u8,
    /// Color-depth support for YCbCr 4:2:2 encoding (byte 2).
    pub color_depth_ycbcr422: u8,
    /// Color-depth support for YCbCr 4:2:0 encoding (byte 3).
    pub color_depth_ycbcr420: u8,
    /// Minimum pixel rate for YCbCr 4:2:0 in 74.25 MP/s units, 0 = supported at all modes (byte 4).
    pub min_ycbcr420_pixel_rate: u8,
    /// Audio capability and feature support flags (byte 5).
    pub audio_flags: u8,
    /// Color space and EOTF combination 1 flags (byte 6).
    pub colorspace_eotf_1: u8,
    /// Color space and EOTF combination 2 flags, reserved (byte 7).
    pub colorspace_eotf_2: u8,
    /// Number of additional color space and EOTF bytes (byte 8).
    pub additional_colorspace_count: u8,
    /// Original payload bytes.
    pub raw: Vec<u8>,
}

impl DisplayIdInterfaceFeatures {
    /// Return whether the given RGB bit depth (6, 8, 10, 12, 14, or 16 bpc) is supported.
    #[must_use]
    pub const fn supports_rgb_bpc(&self, bpc: u8) -> bool {
        Self::supports_rgb_bpc_flags(self.color_depth_rgb, bpc)
    }

    /// Return whether the given YCbCr 4:4:4 bit depth is supported.
    #[must_use]
    pub const fn supports_ycbcr444_bpc(&self, bpc: u8) -> bool {
        Self::supports_ycbcr_bpc_flags(self.color_depth_ycbcr444, bpc)
    }

    /// Return whether the given YCbCr 4:2:2 bit depth is supported.
    #[must_use]
    pub const fn supports_ycbcr422_bpc(&self, bpc: u8) -> bool {
        Self::supports_ycbcr_bpc_flags(self.color_depth_ycbcr422, bpc)
    }

    /// Return whether the given YCbCr 4:2:0 bit depth is supported.
    #[must_use]
    pub const fn supports_ycbcr420_bpc(&self, bpc: u8) -> bool {
        Self::supports_ycbcr_bpc_flags(self.color_depth_ycbcr420, bpc)
    }

    /// RGB encoding bit-depth bit positions: bit0=6bpc, bit1=8bpc, ..., bit5=16bpc.
    const fn supports_rgb_bpc_flags(flags: u8, bpc: u8) -> bool {
        matches!(bpc, 6 | 8 | 10 | 12 | 14 | 16) && (flags & (1 << ((bpc / 2) - 3))) != 0
    }

    /// YCbCr (4:4:4 / 4:2:2 / 4:2:0) bit-depth bit positions: bit0=8bpc, bit1=10bpc, ..., bit4=16bpc.
    const fn supports_ycbcr_bpc_flags(flags: u8, bpc: u8) -> bool {
        matches!(bpc, 8 | 10 | 12 | 14 | 16) && (flags & (1 << ((bpc / 2) - 4))) != 0
    }

    /// Return whether BT.2020 color space with SMPTE ST 2084 (PQ) EOTF is supported.
    #[must_use]
    pub const fn supports_bt2020_st2084(&self) -> bool {
        self.colorspace_eotf_1 & (1 << 6) != 0
    }

    /// Return whether BT.2020 color space with the BT.2020 EOTF is supported.
    #[must_use]
    pub const fn supports_bt2020(&self) -> bool {
        self.colorspace_eotf_1 & (1 << 5) != 0
    }

    /// Return whether BT.709 color space with BT.1886 EOTF is supported.
    #[must_use]
    pub const fn supports_bt709(&self) -> bool {
        self.colorspace_eotf_1 & (1 << 2) != 0
    }
}

/// DisplayID Product Identification Data Block (Tag 0x20 in 2.0, Tag 0x00 in 1.x).
///
/// 2.0 uses an IEEE OUI for the vendor; 1.x uses a three-character vendor ID.
/// The 3-byte `vendor_id` is stored verbatim and `vendor_id_is_oui` distinguishes
/// the two layouts so a typed round-trip never mixes the field rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayIdProductIdentification {
    /// Vendor identifier: 2.0 IEEE OUI bytes or 1.x three-character vendor ID.
    pub vendor_id: [u8; 3],
    /// Whether `vendor_id` is a 2.0 IEEE OUI (true) or a 1.x character ID (false).
    pub vendor_id_is_oui: bool,
    /// Product code (2 bytes, LSB/MSB).
    pub product_code: u16,
    /// Serial number (4 bytes, little-endian), 0 if unspecified.
    pub serial_number: u32,
    /// Week of manufacture, 0 = unspecified, 255 = model-year tag.
    pub week_of_manufacture: u8,
    /// Year of manufacture / model year (2000 + stored value).
    pub year: u16,
    /// Product name raw bytes (exact, possibly non-UTF-8), up to 236 bytes.
    pub product_name: Vec<u8>,
    /// Original payload bytes.
    pub raw: Vec<u8>,
}

impl DisplayIdProductIdentification {
    /// Return the product name as a UTF-8 slice, or `None` if it is not valid UTF-8.
    #[must_use]
    pub fn product_name_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.product_name).ok()
    }

    /// Return a lossy UTF-8 view of the product name, replacing invalid sequences.
    #[must_use]
    pub fn product_name_lossy(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.product_name)
    }

    /// Whether byte 12 uses the model-year tag (`0xFF`) instead of a manufacture week.
    #[must_use]
    pub const fn is_model_year(&self) -> bool {
        self.week_of_manufacture == 0xFF
    }
}

/// DisplayID Tiled Display Topology Data Block (Tag 0x28 in 2.0, Tag 0x12 in 1.x).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayIdTiledDisplayTopology {
    /// Raw capability flags byte.
    pub caps: u8,
    /// Horizontal tile count (1..=64).
    pub tiles_h: u8,
    /// Vertical tile count (1..=64).
    pub tiles_v: u8,
    /// Horizontal tile location (0-based).
    pub tile_location_h: u8,
    /// Vertical tile location (0-based).
    pub tile_location_v: u8,
    /// Tile width in pixels (1..=65536).
    pub tile_width: u16,
    /// Tile height in pixels (1..=65536).
    pub tile_height: u16,
    /// Pixel multiplier for bevel sizes (0 = no bevel scale).
    pub pixel_multiplier: u8,
    /// Top bevel size, present when bevel info is available.
    pub bevel_top: Option<u8>,
    /// Bottom bevel size, present when bevel info is available.
    pub bevel_bottom: Option<u8>,
    /// Right bevel size, present when bevel info is available.
    pub bevel_right: Option<u8>,
    /// Left bevel size, present when bevel info is available.
    pub bevel_left: Option<u8>,
    /// Vendor identifier: 2.0 IEEE OUI or 1.x character ID.
    pub vendor_id: [u8; 3],
    /// Whether `vendor_id` is a 2.0 IEEE OUI (true) or a 1.x character ID (false).
    pub vendor_id_is_oui: bool,
    /// Tiled display product code.
    pub product_code: u16,
    /// Tiled display serial number.
    pub serial_number: u32,
    /// Original payload bytes.
    pub raw: Vec<u8>,
}

impl DisplayIdTiledDisplayTopology {
    /// Whether bevel information is present (capability bit 6).
    #[must_use]
    pub const fn has_bevel_info(&self) -> bool {
        self.caps & 0x40 != 0
    }

    /// Whether the tiled display is a single physical enclosure (capability bit 7).
    #[must_use]
    pub const fn single_enclosure(&self) -> bool {
        self.caps & 0x80 != 0
    }

    /// Behavior when this is the only visible tile (capability bits 0..2).
    #[must_use]
    pub const fn only_tile_behavior(&self) -> u8 {
        self.caps & 0x07
    }

    /// Behavior when more than one tile is visible but not all (capability bits 3..4).
    #[must_use]
    pub const fn multi_tile_behavior(&self) -> u8 {
        (self.caps >> 3) & 0x03
    }
}

/// DisplayID Type IX formula-based timing descriptor (part of tag 0x24).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayIdFormulaTiming {
    /// Horizontal active pixels (1..=65536).
    pub h_active: u16,
    /// Vertical active lines (1..=65536).
    pub v_active: u16,
    /// Vertical refresh rate in Hz (1..=256).
    pub v_refresh_hz: u16,
    /// Timing formula: 0 = CVT, 1 = CVT-RB, 2 = CVT-R2.
    pub formula: u8,
    /// Whether the NTSC refresh rate × (1000/1001) variant is supported.
    pub ntsc_refresh: bool,
    /// Stereoscopic 3D mode (bits 6..5): 0 = mono, 1 = 3D, 2 = user action.
    pub stereo_3d: u8,
}

/// A DisplayID Type VIII enumerated timing code (part of tag 0x23).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayIdEnumeratedTiming {
    /// Timing code type: 0 = DMT, 1 = CTA VIC, 2 = HDMI VIC.
    pub code_type: u8,
    /// Timing code value.
    pub code: u16,
}

/// Ordering policy used when re-ordering DisplayID data blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayIdOrdering {
    /// Keep the source order; the block bytes are left unchanged.
    PreserveSourceOrder,
    /// Deterministically sort data blocks by tag, revision, then payload.
    Canonical,
}

/// Typed read-only views for DisplayID data blocks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DisplayIdDataBlockView {
    /// DisplayID 1.x or 2.x Product Identification Data Block.
    ProductIdentification {
        /// Decoded product identification fields.
        product: DisplayIdProductIdentification,
    },
    /// DisplayID 1.x or 2.x Display Parameters Data Block.
    DisplayParameters {
        /// Decoded display parameters.
        parameters: DisplayIdDisplayParameters,
    },
    /// DisplayID Type I (1.x) or Type VII (2.x) detailed timings.
    DetailedTiming {
        /// Timing entries in source order.
        timings: Vec<DisplayIdDetailedTiming>,
    },
    /// DisplayID Display Interface Features (Tag 0x26 or Tag 0x0F).
    InterfaceFeatures {
        /// Decoded interface features.
        features: DisplayIdInterfaceFeatures,
    },
    /// DisplayID 2.0 Dynamic Video Timing Range Limits (Tag 0x25).
    DynamicVideoTimingRange {
        /// Decoded dynamic video timing range limits.
        range: DisplayIdDynamicVideoTimingRange,
    },
    /// DisplayID 1.x Video Timing Range Limits (Tag 0x09).
    VideoTimingRange1x {
        /// Decoded 1.x video timing range limits.
        range: DisplayIdVideoTimingRangeLimits,
    },
    /// DisplayID Type IX formula-based timings (Tag 0x24).
    FormulaTiming {
        /// Formula-based timing descriptors in source order.
        timings: Vec<DisplayIdFormulaTiming>,
    },
    /// DisplayID Type VIII enumerated timing codes (Tag 0x23).
    EnumeratedTiming {
        /// Timing code type (DMT / CTA VIC / HDMI VIC) shared by all codes.
        code_type: u8,
        /// Timing code size in bytes (1 or 2).
        code_size: u8,
        /// Timing code values in source order.
        codes: Vec<u16>,
    },
    /// DisplayID Tiled Display Topology (Tag 0x28 or Tag 0x12).
    TiledDisplayTopology {
        /// Decoded tiled display topology.
        topology: DisplayIdTiledDisplayTopology,
    },
    /// Embedded CTA data-block collection.
    Cta {
        /// Parsed CTA data blocks in source order.
        data_blocks: Vec<CtaDataBlock>,
        /// Original embedded CTA payload. Re-encoding the typed view uses the parsed
        /// blocks and may canonicalize padding or other unmodeled tail bytes.
        raw: Vec<u8>,
    },
    /// Any DisplayID tag not modeled by this version.
    Unknown {
        /// DisplayID data-block tag.
        tag: u8,
        /// Uninterpreted payload bytes.
        payload: Vec<u8>,
    },
}

/// Errors returned while encoding EDID extension structures.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExtensionWriteError {
    /// CTA data-block tag does not fit its three-bit field.
    InvalidCtaTag {
        /// Supplied tag.
        tag: u8,
    },
    /// CTA data-block payload exceeds its five-bit length field.
    CtaPayloadTooLong {
        /// Supplied length.
        length: usize,
        /// Maximum length.
        maximum: usize,
    },
    /// A CTA data block is shorter than the typed representation requires.
    CtaPayloadTooShort {
        /// CTA data-block tag.
        tag: u8,
        /// Supplied payload length.
        length: usize,
        /// Minimum representable payload length.
        minimum: usize,
    },
    /// A raw extended CTA payload has the wrong extended tag or is empty.
    InvalidCtaExtendedPayload {
        /// Expected extended tag.
        expected_tag: u8,
        /// Actual first byte, if present.
        actual_tag: Option<u8>,
        /// Supplied payload length.
        length: usize,
    },
    /// HDR luminance fields must be present as a contiguous prefix.
    InvalidCtaHdrLuminanceOrder,
    /// An HDR raw tail cannot be preserved after shortening its luminance prefix.
    InvalidCtaHdrRawTail,
    /// A CTA Video Identification Code cannot fit its seven-bit field.
    InvalidCtaVideoCode {
        /// Zero-based mode index.
        index: usize,
        /// Supplied VIC.
        vic: u8,
    },
    /// A CTA audio format cannot fit its four-bit field.
    InvalidCtaAudioFormat {
        /// Zero-based descriptor index.
        index: usize,
        /// Supplied format.
        format: u8,
    },
    /// A CTA audio channel count is outside 1..=8.
    InvalidCtaAudioChannels {
        /// Zero-based descriptor index.
        index: usize,
        /// Supplied channel count.
        channels: u8,
    },
    /// A CTA audio descriptor uses the reserved sample-rate bit.
    InvalidCtaAudioSampleRates {
        /// Zero-based descriptor index.
        index: usize,
        /// Supplied mask.
        sample_rates: u8,
    },
    /// A CTA bitfield value is outside its encoded range.
    InvalidCtaField {
        /// Field name.
        field: &'static str,
        /// Supplied value.
        value: u8,
        /// Maximum value.
        maximum: u8,
    },
    /// A known CTA vendor-specific block has a different OUI than its typed variant.
    InvalidCtaVendorOui {
        /// Expected OUI in little-endian byte order.
        expected: [u8; 3],
        /// OUI found in the raw payload.
        actual: [u8; 3],
    },
    /// A vendor-specific TMDS rate is not representable in its five-MHz byte field.
    InvalidCtaVendorRate {
        /// Typed field name.
        field: &'static str,
        /// Supplied rate in MHz.
        value: u16,
    },
    /// A typed vendor-specific field cannot be represented by the raw payload shape.
    InvalidCtaVendorField {
        /// Typed field name.
        field: &'static str,
    },
    /// Minimum refresh rate exceeds maximum refresh rate or is zero.
    InvalidRefreshRateRange {
        /// Minimum refresh rate in Hz.
        min_refresh_hz: u8,
        /// Maximum refresh rate in Hz.
        max_refresh_hz: u8,
    },
    /// The complete CTA data-block collection does not fit before byte 127.
    CtaDataBlocksTooLong {
        /// Supplied collection length.
        length: usize,
        /// Maximum collection length.
        maximum: usize,
    },
    /// The CTA DTD collection exceeds available slots.
    CtaDtdsTooLong {
        /// Supplied DTD count.
        count: usize,
        /// Maximum slot count.
        maximum: usize,
    },
    /// A CTA DTD cannot be represented.
    CtaDtdInvalid {
        /// Zero-based DTD index.
        index: usize,
        /// Underlying error.
        source: crate::error::DtdError,
    },
    /// CTA's DTD offset is outside the block or does not match its layout.
    InvalidDtdOffset {
        /// Raw CTA DTD offset byte.
        offset: usize,
    },
    /// The complete DisplayID payload exceeds the available section space.
    DisplayIdPayloadTooLong {
        /// Supplied payload length.
        length: usize,
        /// Maximum representable payload length.
        maximum: usize,
    },
    /// The DisplayID payload is shorter than a typed representation requires.
    DisplayIdPayloadTooShort {
        /// DisplayID data-block tag.
        tag: u8,
        /// Supplied payload length.
        length: usize,
        /// Minimum representable payload length.
        minimum: usize,
    },
    /// A DisplayID timing field cannot be represented after the encoded minus-one transform.
    InvalidDisplayIdTimingField {
        /// DisplayID data-block tag.
        tag: u8,
        /// Zero-based timing index.
        index: usize,
        /// Field name.
        field: &'static str,
        /// Supplied value.
        value: u32,
        /// Largest representable decoded value.
        maximum: u32,
    },
    /// A Type I DisplayID pixel clock is not an integral number of 10-kHz units.
    InvalidDisplayIdPixelClock {
        /// Zero-based timing index.
        index: usize,
        /// Supplied pixel clock in kHz.
        value: u32,
    },
    /// A DisplayID data-block tag cannot be emitted by the canonical typed encoder.
    InvalidDisplayIdTag {
        /// Supplied tag.
        tag: u8,
    },
    /// An embedded CTA block could not be decoded and re-encoded through its typed view.
    InvalidDisplayIdEmbeddedCta {
        /// Underlying CTA parsing error.
        source: ExtensionError,
    },
    /// DisplayID Dynamic Video Timing Range has an invalid range (min > max or out of range).
    InvalidDisplayIdDynamicRange {
        /// DisplayID data-block tag.
        tag: u8,
        /// Detail description of the invalid range constraint.
        reason: &'static str,
    },
    /// A DisplayID interface feature field is out of its representable range.
    InvalidDisplayIdFeatureField {
        /// Field name.
        field: &'static str,
        /// Supplied value.
        value: u8,
        /// Maximum representable value.
        maximum: u8,
    },
    /// A DisplayID product-identification field is out of its representable range.
    InvalidDisplayIdProductField {
        /// Field name.
        field: &'static str,
        /// Supplied value.
        value: u32,
        /// Maximum representable value.
        maximum: u32,
    },
    /// A DisplayID data-block layout cannot be re-read while re-ordering.
    InvalidDisplayIdLayout {
        /// Underlying read error.
        source: ExtensionError,
    },
    /// An HDR dynamic metadata entry cannot be represented.
    InvalidHdrDynamicMetadataEntry {
        /// Zero-based entry index.
        index: usize,
        /// Reason the entry is not representable.
        reason: &'static str,
    },
    /// The CTA extension has a malformed data-block collection or DTD layout.
    InvalidCtaLayout {
        /// Underlying structured CTA parsing error.
        source: ExtensionError,
    },
    /// The target block is not a CTA-861 extension.
    NotCta861,
    /// The target block is not a DisplayID extension.
    NotDisplayId,
}
impl std::fmt::Display for ExtensionWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCtaTag { tag } => {
                write!(f, "CTA data-block tag {tag} exceeds the three-bit field")
            }
            Self::CtaPayloadTooLong { length, maximum } => write!(
                f,
                "CTA data-block payload length {length} exceeds the {maximum}-byte maximum"
            ),
            Self::InvalidCtaVideoCode { index, vic } => write!(
                f,
                "CTA video code {vic} at index {index} is outside 1..=127"
            ),
            Self::InvalidCtaAudioFormat { index, format } => write!(
                f,
                "CTA audio format {format} at index {index} is outside 1..=15"
            ),
            Self::InvalidCtaAudioChannels { index, channels } => write!(
                f,
                "CTA audio channel count {channels} at index {index} is outside 1..=8"
            ),
            Self::InvalidCtaAudioSampleRates {
                index,
                sample_rates,
            } => write!(
                f,
                "CTA audio sample-rate mask 0x{sample_rates:02X} at index {index} uses a reserved bit"
            ),
            Self::CtaPayloadTooShort {
                tag,
                length,
                minimum,
            } => write!(
                f,
                "CTA data-block tag {tag} payload length {length} is below minimum {minimum}"
            ),
            Self::InvalidCtaExtendedPayload {
                expected_tag,
                actual_tag,
                length,
            } => write!(
                f,
                "CTA extended payload length {length} has tag {actual_tag:?}, expected {expected_tag}"
            ),
            Self::InvalidCtaVendorOui { expected, actual } => write!(
                f,
                "CTA vendor OUI {actual:02X?} does not match expected {expected:02X?}"
            ),
            Self::InvalidCtaVendorRate { field, value } => write!(
                f,
                "CTA vendor field {field} rate {value} MHz is not representable in 5-MHz units"
            ),
            Self::InvalidCtaVendorField { field } => {
                write!(
                    f,
                    "CTA vendor field {field} is not representable by the raw payload"
                )
            }
            Self::InvalidCtaHdrLuminanceOrder => {
                f.write_str("CTA HDR luminance fields must be a contiguous prefix")
            }
            Self::InvalidCtaHdrRawTail => f.write_str(
                "CTA HDR raw tail cannot be preserved after shortening the luminance prefix",
            ),
            Self::InvalidCtaField {
                field,
                value,
                maximum,
            } => write!(
                f,
                "CTA field {field} value {value} exceeds maximum {maximum}"
            ),
            Self::InvalidRefreshRateRange {
                min_refresh_hz,
                max_refresh_hz,
            } => write!(
                f,
                "invalid refresh rate range: {min_refresh_hz} Hz .. {max_refresh_hz} Hz"
            ),
            Self::CtaDataBlocksTooLong { length, maximum } => write!(
                f,
                "CTA data-block collection length {length} exceeds the {maximum}-byte maximum"
            ),
            Self::CtaDtdsTooLong { count, maximum } => write!(
                f,
                "CTA DTD count {count} exceeds the {maximum}-slot maximum"
            ),
            Self::CtaDtdInvalid { index, source } => {
                write!(f, "CTA DTD {index} is invalid: {source}")
            }
            Self::InvalidDtdOffset { offset } => {
                write!(f, "CTA DTD offset {offset} is outside the extension")
            }
            Self::DisplayIdPayloadTooLong { length, maximum } => write!(
                f,
                "DisplayID payload length {length} exceeds the {maximum}-byte maximum"
            ),
            Self::DisplayIdPayloadTooShort {
                tag,
                length,
                minimum,
            } => write!(
                f,
                "DisplayID tag 0x{tag:02X} payload length {length} is below minimum {minimum}"
            ),
            Self::InvalidDisplayIdTimingField {
                tag,
                index,
                field,
                value,
                maximum,
            } => write!(
                f,
                "DisplayID tag 0x{tag:02X} timing {index} field {field} value {value} exceeds maximum {maximum}"
            ),
            Self::InvalidDisplayIdPixelClock { index, value } => write!(
                f,
                "DisplayID Type I timing {index} pixel clock {value} kHz is not a multiple of 10"
            ),
            Self::InvalidDisplayIdTag { tag } => {
                write!(
                    f,
                    "DisplayID tag 0x{tag:02X} is not supported by this typed encoder"
                )
            }
            Self::InvalidDisplayIdDynamicRange { tag, reason } => write!(
                f,
                "DisplayID dynamic range data block 0x{tag:02X} has invalid range: {reason}"
            ),
            Self::InvalidDisplayIdFeatureField {
                field,
                value,
                maximum,
            } => write!(
                f,
                "DisplayID interface feature field {field} value {value} exceeds maximum {maximum}"
            ),
            Self::InvalidDisplayIdProductField {
                field,
                value,
                maximum,
            } => write!(
                f,
                "DisplayID product-identification field {field} value {value} exceeds maximum {maximum}"
            ),
            Self::InvalidDisplayIdLayout { source } => {
                write!(f, "DisplayID data-block layout is invalid: {source}")
            }
            Self::InvalidHdrDynamicMetadataEntry { index, reason } => write!(
                f,
                "CTA HDR dynamic metadata entry {index} is invalid: {reason}"
            ),
            Self::InvalidCtaLayout { source } => {
                write!(f, "CTA extension layout is invalid: {source}")
            }
            Self::InvalidDisplayIdEmbeddedCta { source } => {
                write!(f, "embedded CTA data block is invalid: {source}")
            }
            Self::NotCta861 => f.write_str("block is not a CTA-861 extension"),
            Self::NotDisplayId => f.write_str("block is not a DisplayID extension"),
        }
    }
}

impl DisplayIdDataBlock {
    /// Encode this DisplayID data block with its three-byte header.
    pub fn encode(&self) -> Result<Vec<u8>, ExtensionWriteError> {
        if self.payload.len() > u8::MAX as usize {
            return Err(ExtensionWriteError::DisplayIdPayloadTooLong {
                length: self.payload.len(),
                maximum: u8::MAX as usize,
            });
        }
        let mut encoded = Vec::with_capacity(self.payload.len() + 3);
        encoded.extend_from_slice(&[self.tag, self.revision, self.payload.len() as u8]);
        encoded.extend_from_slice(&self.payload);
        Ok(encoded)
    }
}

const MAX_DISPLAY_ID_PAYLOAD: usize = 121;

fn check_display_id_payload_length(length: usize) -> Result<(), ExtensionWriteError> {
    if length > MAX_DISPLAY_ID_PAYLOAD {
        return Err(ExtensionWriteError::DisplayIdPayloadTooLong {
            length,
            maximum: MAX_DISPLAY_ID_PAYLOAD,
        });
    }
    Ok(())
}

impl std::error::Error for ExtensionWriteError {}

impl DisplayIdDataBlock {
    /// Decode this data block into a typed read-only view.
    pub fn view(&self) -> Result<DisplayIdDataBlockView, ExtensionError> {
        match self.tag {
            0x00 | 0x20 => Ok(DisplayIdDataBlockView::ProductIdentification {
                product: decode_product_identification(self)?,
            }),
            0x01 | 0x21 => Ok(DisplayIdDataBlockView::DisplayParameters {
                parameters: decode_display_parameters(self)?,
            }),
            0x03 => Ok(DisplayIdDataBlockView::DetailedTiming {
                timings: decode_detailed_timings(self, true)?,
            }),
            0x22 => Ok(DisplayIdDataBlockView::DetailedTiming {
                timings: decode_detailed_timings(self, false)?,
            }),
            0x0F | 0x26 => Ok(DisplayIdDataBlockView::InterfaceFeatures {
                features: decode_interface_features(self)?,
            }),
            0x09 => Ok(DisplayIdDataBlockView::VideoTimingRange1x {
                range: decode_video_timing_range_limits(self)?,
            }),
            0x25 => Ok(DisplayIdDataBlockView::DynamicVideoTimingRange {
                range: decode_dynamic_video_timing_range(self)?,
            }),
            0x06 | 0x23 => Ok(DisplayIdDataBlockView::EnumeratedTiming {
                code_type: (self.revision & 0xC0) >> 6,
                code_size: if self.revision & 0x08 != 0 { 2 } else { 1 },
                codes: decode_enumerated_timing_codes(self)?,
            }),
            0x24 => Ok(DisplayIdDataBlockView::FormulaTiming {
                timings: decode_formula_timings(self)?,
            }),
            0x12 | 0x28 => Ok(DisplayIdDataBlockView::TiledDisplayTopology {
                topology: decode_tiled_display_topology(self)?,
            }),
            0x81 => {
                let raw = self.payload.clone();
                let data_blocks = parse_cta_data_blocks(&self.payload, 0, false)?;
                Ok(DisplayIdDataBlockView::Cta { data_blocks, raw })
            }
            tag => Ok(DisplayIdDataBlockView::Unknown {
                tag,
                payload: self.payload.clone(),
            }),
        }
    }
}

impl DisplayIdDataBlockView {
    /// Encode this typed view using its canonical DisplayID data-block tag.
    ///
    /// Use [`Self::to_data_block_with_tag`] when preserving the 1.x/2.x tag
    /// distinction matters for Product, Parameters, or Timing views.
    pub fn to_data_block(&self) -> Result<DisplayIdDataBlock, ExtensionWriteError> {
        let tag = match self {
            Self::ProductIdentification { .. } => 0x00,
            Self::DisplayParameters { .. } => 0x01,
            Self::DetailedTiming { timings } => {
                if timings.is_empty() {
                    0x22
                } else if timings
                    .iter()
                    .all(|timing| timing.pixel_clock_khz % 10 == 0)
                {
                    0x03
                } else {
                    0x22
                }
            }
            Self::InterfaceFeatures { .. } => 0x26,
            Self::FormulaTiming { .. } => 0x24,
            Self::EnumeratedTiming { .. } => 0x23,
            Self::DynamicVideoTimingRange { .. } => 0x25,
            Self::VideoTimingRange1x { .. } => 0x09,
            Self::TiledDisplayTopology { .. } => 0x28,
            Self::Cta { .. } => 0x81,
            Self::Unknown { tag, .. } => *tag,
        };
        self.to_data_block_with_tag(tag)
    }

    /// Encode this typed view while explicitly selecting its DisplayID tag.
    pub fn to_data_block_with_tag(
        &self,
        tag: u8,
    ) -> Result<DisplayIdDataBlock, ExtensionWriteError> {
        match self {
            Self::ProductIdentification { product } if matches!(tag, 0x00 | 0x20) => {
                if !(2000..=2255).contains(&product.year) {
                    return Err(ExtensionWriteError::InvalidDisplayIdProductField {
                        field: "year",
                        value: product.year as u32,
                        maximum: 2255,
                    });
                }
                if product.product_name.len() > u8::MAX as usize {
                    return Err(ExtensionWriteError::InvalidDisplayIdProductField {
                        field: "product_name",
                        value: product.product_name.len() as u32,
                        maximum: u8::MAX as u32,
                    });
                }
                let mut payload = Vec::with_capacity(12 + product.product_name.len());
                payload.extend_from_slice(&product.vendor_id);
                payload.extend_from_slice(&product.product_code.to_le_bytes());
                payload.extend_from_slice(&product.serial_number.to_le_bytes());
                payload.push(product.week_of_manufacture);
                payload.push((product.year - 2000) as u8);
                payload.push(product.product_name.len() as u8);
                payload.extend_from_slice(&product.product_name);
                check_display_id_payload_length(payload.len())?;
                Ok(DisplayIdDataBlock {
                    tag,
                    revision: 0,
                    payload,
                })
            }
            Self::DisplayParameters { parameters } if matches!(tag, 0x01 | 0x21) => {
                check_display_id_payload_length(parameters.raw.len())?;
                if parameters.raw.len() < 29 {
                    return Err(ExtensionWriteError::DisplayIdPayloadTooShort {
                        tag,
                        length: parameters.raw.len(),
                        minimum: 29,
                    });
                }
                let mut payload = parameters.raw.clone();
                payload[0..2].copy_from_slice(&parameters.horizontal_image_size_mm.to_le_bytes());
                payload[2..4].copy_from_slice(&parameters.vertical_image_size_mm.to_le_bytes());
                payload[4..6].copy_from_slice(&parameters.horizontal_pixel_count.to_le_bytes());
                payload[6..8].copy_from_slice(&parameters.vertical_pixel_count.to_le_bytes());
                payload[8] = parameters.features;
                payload[9..12].copy_from_slice(&parameters.primary_color_1);
                payload[12..15].copy_from_slice(&parameters.primary_color_2);
                payload[15..18].copy_from_slice(&parameters.primary_color_3);
                payload[18..21].copy_from_slice(&parameters.white_point);
                payload[21..23].copy_from_slice(&parameters.max_luminance_full.to_le_bytes());
                payload[23..25].copy_from_slice(&parameters.max_luminance_10_percent.to_le_bytes());
                payload[25..27].copy_from_slice(&parameters.min_luminance.to_le_bytes());
                payload[27] = parameters.color_depth_and_technology;
                payload[28] = parameters.gamma_eotf;
                Ok(DisplayIdDataBlock {
                    tag,
                    revision: 0,
                    payload,
                })
            }
            Self::DetailedTiming { timings } if matches!(tag, 0x03 | 0x22) => {
                if timings.is_empty() {
                    return Err(ExtensionWriteError::DisplayIdPayloadTooShort {
                        tag,
                        length: 0,
                        minimum: 20,
                    });
                }
                let type_one = tag == 0x03;
                let length = timings.len().checked_mul(20).ok_or(
                    ExtensionWriteError::DisplayIdPayloadTooLong {
                        length: usize::MAX,
                        maximum: MAX_DISPLAY_ID_PAYLOAD,
                    },
                )?;
                check_display_id_payload_length(length)?;
                let block_revision = if timings.iter().any(|t| t.ycbcr420) {
                    2
                } else {
                    0
                };
                let mut payload = Vec::with_capacity(length);
                for (index, timing) in timings.iter().enumerate() {
                    payload.extend_from_slice(&encode_display_id_timing(
                        timing,
                        index,
                        type_one,
                        tag,
                        block_revision,
                    )?);
                }
                Ok(DisplayIdDataBlock {
                    tag,
                    revision: block_revision,
                    payload,
                })
            }
            Self::Cta { data_blocks, .. } if tag == 0x81 => {
                let mut payload = Vec::new();
                for data_block in data_blocks {
                    let typed = data_block.view().map_err(|source| {
                        ExtensionWriteError::InvalidDisplayIdEmbeddedCta { source }
                    })?;
                    payload.extend_from_slice(&typed.to_data_block()?.encode()?);
                }
                check_display_id_payload_length(payload.len())?;
                Ok(DisplayIdDataBlock {
                    tag,
                    revision: 0,
                    payload,
                })
            }
            Self::Unknown {
                tag: original,
                payload,
            } if original == &tag => {
                check_display_id_payload_length(payload.len())?;
                Ok(DisplayIdDataBlock {
                    tag,
                    revision: 0,
                    payload: payload.clone(),
                })
            }
            Self::InterfaceFeatures { features } if matches!(tag, 0x0F | 0x26) => {
                if features.additional_colorspace_count > 7 {
                    return Err(ExtensionWriteError::InvalidDisplayIdFeatureField {
                        field: "additional_colorspace_count",
                        value: features.additional_colorspace_count,
                        maximum: 7,
                    });
                }
                let mut payload = if features.raw.len() >= 9 {
                    features.raw.clone()
                } else {
                    vec![0u8; 9]
                };
                check_display_id_payload_length(payload.len())?;
                payload[0] = features.color_depth_rgb;
                payload[1] = features.color_depth_ycbcr444;
                payload[2] = features.color_depth_ycbcr422;
                payload[3] = features.color_depth_ycbcr420;
                payload[4] = features.min_ycbcr420_pixel_rate;
                payload[5] = features.audio_flags;
                payload[6] = features.colorspace_eotf_1;
                payload[7] = features.colorspace_eotf_2;
                payload[8] = features.additional_colorspace_count;
                Ok(DisplayIdDataBlock {
                    tag,
                    revision: 0,
                    payload,
                })
            }
            Self::TiledDisplayTopology { topology } if matches!(tag, 0x12 | 0x28) => {
                if topology.tiles_h == 0
                    || topology.tiles_v == 0
                    || topology.tile_location_h >= topology.tiles_h
                    || topology.tile_location_v >= topology.tiles_v
                    || topology.tile_width == 0
                    || topology.tile_height == 0
                {
                    return Err(ExtensionWriteError::InvalidDisplayIdDynamicRange {
                        tag,
                        reason: "tile count, location, or size is invalid",
                    });
                }
                if !topology.has_bevel_info() && topology.pixel_multiplier != 0 {
                    return Err(ExtensionWriteError::InvalidDisplayIdDynamicRange {
                        tag,
                        reason: "bevel multiplier set without bevel info",
                    });
                }
                if topology.tiles_h > 64
                    || topology.tiles_v > 64
                    || topology.tile_location_h >= 64
                    || topology.tile_location_v >= 64
                {
                    return Err(ExtensionWriteError::InvalidDisplayIdDynamicRange {
                        tag,
                        reason: "tile count or location exceeds 6-bit field",
                    });
                }
                let mut payload = topology.raw.clone();
                if payload.len() < 22 {
                    payload.resize(22, 0);
                }
                check_display_id_payload_length(payload.len())?;
                let num_h_stored = topology.tiles_h - 1;
                let num_v_stored = topology.tiles_v - 1;
                payload[0] = topology.caps;
                payload[1] = ((num_h_stored & 0x0F) << 4) | (num_v_stored & 0x0F);
                payload[2] =
                    ((topology.tile_location_h & 0x0F) << 4) | (topology.tile_location_v & 0x0F);
                payload[3] = ((num_h_stored >> 4) & 0x03) << 6
                    | ((num_v_stored >> 4) & 0x03) << 4
                    | ((topology.tile_location_h >> 4) & 0x03) << 2
                    | ((topology.tile_location_v >> 4) & 0x03);
                payload[4..6].copy_from_slice(&(topology.tile_width - 1).to_le_bytes());
                payload[6..8].copy_from_slice(&(topology.tile_height - 1).to_le_bytes());
                payload[8] = topology.pixel_multiplier;
                if topology.has_bevel_info() {
                    payload[9] = topology.bevel_top.unwrap_or(0);
                    payload[10] = topology.bevel_bottom.unwrap_or(0);
                    payload[11] = topology.bevel_right.unwrap_or(0);
                    payload[12] = topology.bevel_left.unwrap_or(0);
                }
                payload[13..16].copy_from_slice(&topology.vendor_id);
                payload[16..18].copy_from_slice(&topology.product_code.to_le_bytes());
                payload[18..22].copy_from_slice(&topology.serial_number.to_le_bytes());
                Ok(DisplayIdDataBlock {
                    tag,
                    revision: 0,
                    payload,
                })
            }
            Self::EnumeratedTiming {
                code_type,
                code_size,
                codes,
            } if matches!(tag, 0x06 | 0x23) => {
                if *code_type > 2 {
                    return Err(ExtensionWriteError::InvalidDisplayIdFeatureField {
                        field: "code_type",
                        value: *code_type,
                        maximum: 2,
                    });
                }
                if *code_size != 1 && *code_size != 2 {
                    return Err(ExtensionWriteError::InvalidDisplayIdFeatureField {
                        field: "code_size",
                        value: *code_size,
                        maximum: 2,
                    });
                }
                let mut payload = Vec::with_capacity(codes.len() * (*code_size as usize));
                for &code in codes {
                    if code_size == &2 {
                        payload.extend_from_slice(&code.to_le_bytes());
                    } else {
                        if code > u8::MAX as u16 {
                            return Err(ExtensionWriteError::InvalidDisplayIdFeatureField {
                                field: "code",
                                value: code as u8,
                                maximum: u8::MAX,
                            });
                        }
                        payload.push(code as u8);
                    }
                }
                check_display_id_payload_length(payload.len())?;
                let revision = (*code_type << 6) | if *code_size == 2 { 0x08 } else { 0 };
                Ok(DisplayIdDataBlock {
                    tag,
                    revision,
                    payload,
                })
            }
            Self::FormulaTiming { timings } if tag == 0x24 => {
                if timings.is_empty() {
                    return Err(ExtensionWriteError::DisplayIdPayloadTooShort {
                        tag,
                        length: 0,
                        minimum: 6,
                    });
                }
                let mut payload = Vec::with_capacity(timings.len() * 6);
                for (index, timing) in timings.iter().enumerate() {
                    if timing.h_active == 0
                        || timing.v_active == 0
                        || timing.v_refresh_hz == 0
                        || timing.v_refresh_hz > 256
                    {
                        return Err(ExtensionWriteError::InvalidDisplayIdTimingField {
                            tag,
                            index,
                            field: "formula_timing",
                            value: timing.h_active as u32,
                            maximum: 65_536,
                        });
                    }
                    if timing.stereo_3d > 3 {
                        return Err(ExtensionWriteError::InvalidDisplayIdFeatureField {
                            field: "stereo_3d",
                            value: timing.stereo_3d,
                            maximum: 3,
                        });
                    }
                    let options = (timing.stereo_3d << 5)
                        | (u8::from(timing.ntsc_refresh) << 4)
                        | (timing.formula & 0x07);
                    payload.push(options);
                    payload.extend_from_slice(&(timing.h_active - 1).to_le_bytes());
                    payload.extend_from_slice(&(timing.v_active - 1).to_le_bytes());
                    payload.push((timing.v_refresh_hz - 1) as u8);
                }
                check_display_id_payload_length(payload.len())?;
                Ok(DisplayIdDataBlock {
                    tag,
                    revision: 0,
                    payload,
                })
            }
            Self::DynamicVideoTimingRange { range } if tag == 0x25 => {
                const MAX_PIXEL_KHZ: u32 = 0x0100_0000;
                if !(1..=MAX_PIXEL_KHZ).contains(&range.min_pixel_clock_khz)
                    || !(1..=MAX_PIXEL_KHZ).contains(&range.max_pixel_clock_khz)
                    || range.min_pixel_clock_khz > range.max_pixel_clock_khz
                {
                    return Err(ExtensionWriteError::InvalidDisplayIdDynamicRange {
                        tag,
                        reason: "pixel clock out of range or min exceeds max",
                    });
                }
                if range.min_vfreq_hz == 0
                    || range.max_vfreq_hz == 0
                    || range.max_vfreq_hz > 1023
                    || u16::from(range.min_vfreq_hz) > range.max_vfreq_hz
                {
                    return Err(ExtensionWriteError::InvalidDisplayIdDynamicRange {
                        tag,
                        reason: "refresh rate out of range or min exceeds max",
                    });
                }
                let mut payload = if range.raw.len() >= 9 {
                    range.raw.clone()
                } else {
                    vec![0u8; 9]
                };
                check_display_id_payload_length(payload.len())?;
                let min_clock = (range.min_pixel_clock_khz - 1).to_le_bytes();
                payload[0..3].copy_from_slice(&min_clock[..3]);
                let max_clock = (range.max_pixel_clock_khz - 1).to_le_bytes();
                payload[3..6].copy_from_slice(&max_clock[..3]);
                payload[6] = range.min_vfreq_hz;
                payload[7] = (range.max_vfreq_hz & 0xFF) as u8;
                let flags_base = payload[8] & 0x7C;
                let seamless_bit = u8::from(range.seamless_dynamic_video_timing) << 7;
                let upper_vfreq = ((range.max_vfreq_hz >> 8) & 0x03) as u8;
                payload[8] = flags_base | seamless_bit | upper_vfreq;
                Ok(DisplayIdDataBlock {
                    tag,
                    revision: 0,
                    payload,
                })
            }
            Self::VideoTimingRange1x { range } if tag == 0x09 => {
                const MAX_PIXEL_KHZ: u32 = 0x0100_0000;
                if !(1..=MAX_PIXEL_KHZ).contains(&range.min_pixel_clock_khz)
                    || !(1..=MAX_PIXEL_KHZ).contains(&range.max_pixel_clock_khz)
                    || range.min_pixel_clock_khz > range.max_pixel_clock_khz
                {
                    return Err(ExtensionWriteError::InvalidDisplayIdDynamicRange {
                        tag,
                        reason: "pixel clock out of range or min exceeds max",
                    });
                }
                if !range.min_pixel_clock_khz.is_multiple_of(10)
                    || !range.max_pixel_clock_khz.is_multiple_of(10)
                {
                    return Err(ExtensionWriteError::InvalidDisplayIdDynamicRange {
                        tag,
                        reason: "1.x pixel clock must be a multiple of 10 kHz",
                    });
                }
                if range.min_hfreq_khz > range.max_hfreq_khz {
                    return Err(ExtensionWriteError::InvalidDisplayIdDynamicRange {
                        tag,
                        reason: "horizontal frequency min exceeds max",
                    });
                }
                if range.min_vfreq_hz == 0
                    || range.max_vfreq_hz == 0
                    || range.min_vfreq_hz > range.max_vfreq_hz
                {
                    return Err(ExtensionWriteError::InvalidDisplayIdDynamicRange {
                        tag,
                        reason: "vertical refresh out of range or min exceeds max",
                    });
                }
                let mut payload = if range.raw.len() >= 15 {
                    range.raw.clone()
                } else {
                    vec![0u8; 15]
                };
                check_display_id_payload_length(payload.len())?;
                let min_clock = ((range.min_pixel_clock_khz / 10) - 1).to_le_bytes();
                payload[0..3].copy_from_slice(&min_clock[..3]);
                let max_clock = ((range.max_pixel_clock_khz / 10) - 1).to_le_bytes();
                payload[3..6].copy_from_slice(&max_clock[..3]);
                payload[6] = range.min_hfreq_khz;
                payload[7] = range.max_hfreq_khz;
                payload[8..10].copy_from_slice(&range.min_h_blanking.to_le_bytes());
                payload[10] = range.min_vfreq_hz;
                payload[11] = range.max_vfreq_hz;
                payload[12..14].copy_from_slice(&range.min_v_blanking.to_le_bytes());
                let mut flags = payload[14] & 0x0F;
                flags |= u8::from(range.supports_interlaced) << 7;
                flags |= u8::from(range.supports_cvt) << 6;
                flags |= u8::from(range.supports_cvt_reduced_blanking) << 5;
                flags |= u8::from(range.discrete_frequency) << 4;
                payload[14] = flags;
                Ok(DisplayIdDataBlock {
                    tag,
                    revision: 0,
                    payload,
                })
            }
            _ => Err(ExtensionWriteError::InvalidDisplayIdTag { tag }),
        }
    }
}

fn encode_display_id_timing(
    timing: &DisplayIdDetailedTiming,
    index: usize,
    type_one: bool,
    tag: u8,
    block_revision: u8,
) -> Result<[u8; 20], ExtensionWriteError> {
    let encode_field = |field: &'static str, value: u32, maximum: u32| {
        if !(1..=maximum).contains(&value) {
            return Err(ExtensionWriteError::InvalidDisplayIdTimingField {
                tag,
                index,
                field,
                value,
                maximum,
            });
        }
        Ok((value - 1) as u16)
    };
    let pixel_unit = if type_one {
        if timing.pixel_clock_khz == 0 || !timing.pixel_clock_khz.is_multiple_of(10) {
            return Err(ExtensionWriteError::InvalidDisplayIdPixelClock {
                index,
                value: timing.pixel_clock_khz,
            });
        }
        timing.pixel_clock_khz / 10
    } else {
        timing.pixel_clock_khz
    };
    const MAX_PIXEL_UNIT: u32 = 0x0100_0000;
    if !(1..=MAX_PIXEL_UNIT).contains(&pixel_unit) {
        return Err(ExtensionWriteError::InvalidDisplayIdTimingField {
            tag,
            index,
            field: "pixel_clock_khz",
            value: timing.pixel_clock_khz,
            maximum: if type_one {
                MAX_PIXEL_UNIT * 10
            } else {
                MAX_PIXEL_UNIT
            },
        });
    }
    let h_active = encode_field("h_active", timing.h_active, 65_536)?;
    let h_blank = encode_field("h_blank", timing.h_blank, 65_536)?;
    let h_offset = encode_field("h_sync_offset", timing.h_sync_offset, 32_768)?;
    let h_width = encode_field("h_sync_width", timing.h_sync_width, 65_536)?;
    let v_active = encode_field("v_active", timing.v_active, 65_536)?;
    let v_blank = encode_field("v_blank", timing.v_blank, 65_536)?;
    let v_offset = encode_field("v_sync_offset", timing.v_sync_offset, 32_768)?;
    let v_width = encode_field("v_sync_width", timing.v_sync_width, 65_536)?;
    let mut bytes = [0u8; 20];
    let clock = (pixel_unit - 1).to_le_bytes();
    bytes[0..3].copy_from_slice(&clock[..3]);
    let byte3 = timing.aspect_ratio.nibble()
        | (u8::from(timing.interlaced) << 4)
        | (timing.stereo_3d.bits() << 5)
        | (u8::from(if block_revision < 2 {
            timing.preferred
        } else {
            timing.ycbcr420
        }) << 7);
    bytes[3] = byte3;
    bytes[4..6].copy_from_slice(&h_active.to_le_bytes());
    bytes[6..8].copy_from_slice(&h_blank.to_le_bytes());
    bytes[8..10]
        .copy_from_slice(&(h_offset | (u16::from(timing.h_sync_positive) << 15)).to_le_bytes());
    bytes[10..12].copy_from_slice(&h_width.to_le_bytes());
    bytes[12..14].copy_from_slice(&v_active.to_le_bytes());
    bytes[14..16].copy_from_slice(&v_blank.to_le_bytes());
    bytes[16..18]
        .copy_from_slice(&(v_offset | (u16::from(timing.v_sync_positive) << 15)).to_le_bytes());
    bytes[18..20].copy_from_slice(&v_width.to_le_bytes());
    Ok(bytes)
}

fn decode_detailed_timings(
    block: &DisplayIdDataBlock,
    type_one: bool,
) -> Result<Vec<DisplayIdDetailedTiming>, ExtensionError> {
    if block.payload.is_empty() || !block.payload.len().is_multiple_of(20) {
        return Err(ExtensionError::InvalidDisplayIdDataBlockLength {
            tag: block.tag,
            length: block.payload.len(),
            minimum: 20,
            multiple: 20,
        });
    }
    let block_revision = block.revision & 0x07;
    let mut timings = Vec::with_capacity(block.payload.len() / 20);
    for bytes in block.payload.as_chunks::<20>().0 {
        let pixel_clock = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0]) + 1;
        let hsync = u16::from_le_bytes([bytes[8], bytes[9]]);
        let vsync = u16::from_le_bytes([bytes[16], bytes[17]]);
        let clock_multiplier = if type_one { 10 } else { 1 };
        let byte3 = bytes[3];
        let bit7 = byte3 & 0x80 != 0;
        timings.push(DisplayIdDetailedTiming {
            pixel_clock_khz: pixel_clock * clock_multiplier,
            h_active: u16::from_le_bytes([bytes[4], bytes[5]]) as u32 + 1,
            h_blank: u16::from_le_bytes([bytes[6], bytes[7]]) as u32 + 1,
            h_sync_offset: (hsync & 0x7FFF) as u32 + 1,
            h_sync_width: u16::from_le_bytes([bytes[10], bytes[11]]) as u32 + 1,
            v_active: u16::from_le_bytes([bytes[12], bytes[13]]) as u32 + 1,
            v_blank: u16::from_le_bytes([bytes[14], bytes[15]]) as u32 + 1,
            v_sync_offset: (vsync & 0x7FFF) as u32 + 1,
            v_sync_width: u16::from_le_bytes([bytes[18], bytes[19]]) as u32 + 1,
            h_sync_positive: hsync & 0x8000 != 0,
            v_sync_positive: vsync & 0x8000 != 0,
            aspect_ratio: DisplayIdAspectRatio::from_nibble(byte3 & 0x0F),
            interlaced: byte3 & 0x10 != 0,
            stereo_3d: DisplayIdStereo3d::from_bits((byte3 >> 5) & 0x03),
            preferred: block_revision < 2 && bit7,
            ycbcr420: block_revision >= 2 && bit7,
        });
    }
    Ok(timings)
}

fn decode_display_parameters(
    block: &DisplayIdDataBlock,
) -> Result<DisplayIdDisplayParameters, ExtensionError> {
    if block.payload.len() < 29 {
        return Err(ExtensionError::InvalidDisplayIdDataBlockLength {
            tag: block.tag,
            length: block.payload.len(),
            minimum: 29,
            multiple: 0,
        });
    }
    let bytes = &block.payload;
    Ok(DisplayIdDisplayParameters {
        horizontal_image_size_mm: u16::from_le_bytes([bytes[0], bytes[1]]),
        vertical_image_size_mm: u16::from_le_bytes([bytes[2], bytes[3]]),
        horizontal_pixel_count: u16::from_le_bytes([bytes[4], bytes[5]]),
        vertical_pixel_count: u16::from_le_bytes([bytes[6], bytes[7]]),
        features: bytes[8],
        primary_color_1: [bytes[9], bytes[10], bytes[11]],
        primary_color_2: [bytes[12], bytes[13], bytes[14]],
        primary_color_3: [bytes[15], bytes[16], bytes[17]],
        white_point: [bytes[18], bytes[19], bytes[20]],
        max_luminance_full: u16::from_le_bytes([bytes[21], bytes[22]]),
        max_luminance_10_percent: u16::from_le_bytes([bytes[23], bytes[24]]),
        min_luminance: u16::from_le_bytes([bytes[25], bytes[26]]),
        color_depth_and_technology: bytes[27],
        gamma_eotf: bytes[28],
        raw: bytes.to_vec(),
    })
}

fn decode_dynamic_video_timing_range(
    block: &DisplayIdDataBlock,
) -> Result<DisplayIdDynamicVideoTimingRange, ExtensionError> {
    if block.payload.len() < 9 {
        return Err(ExtensionError::InvalidDisplayIdDataBlockLength {
            tag: block.tag,
            length: block.payload.len(),
            minimum: 9,
            multiple: 0,
        });
    }
    let bytes = &block.payload;
    let min_pixel_clock_khz = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0]) + 1;
    let max_pixel_clock_khz = u32::from_le_bytes([bytes[3], bytes[4], bytes[5], 0]) + 1;
    let min_vfreq_hz = bytes[6];
    let max_vfreq_lower = bytes[7];
    let flags = bytes[8];
    let max_vfreq_upper = (flags & 0x03) as u16;
    let max_vfreq_hz = (max_vfreq_upper << 8) | u16::from(max_vfreq_lower);
    let seamless_dynamic_video_timing = (flags & 0x80) != 0;

    if min_pixel_clock_khz > max_pixel_clock_khz {
        return Err(ExtensionError::InvalidDisplayIdDynamicRange {
            tag: block.tag,
            reason: "min pixel clock exceeds max pixel clock",
        });
    }
    if min_vfreq_hz == 0 || max_vfreq_hz == 0 || u16::from(min_vfreq_hz) > max_vfreq_hz {
        return Err(ExtensionError::InvalidDisplayIdDynamicRange {
            tag: block.tag,
            reason: "min refresh rate exceeds max refresh rate or is zero",
        });
    }

    Ok(DisplayIdDynamicVideoTimingRange {
        min_pixel_clock_khz,
        max_pixel_clock_khz,
        min_vfreq_hz,
        max_vfreq_hz,
        seamless_dynamic_video_timing,
        raw: block.payload.clone(),
    })
}

fn decode_video_timing_range_limits(
    block: &DisplayIdDataBlock,
) -> Result<DisplayIdVideoTimingRangeLimits, ExtensionError> {
    if block.payload.len() != 15 {
        return Err(ExtensionError::InvalidDisplayIdDataBlockLength {
            tag: block.tag,
            length: block.payload.len(),
            minimum: 15,
            multiple: 15,
        });
    }
    let bytes = &block.payload;
    let min_pixel_clock_khz = (u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0]) + 1) * 10;
    let max_pixel_clock_khz = (u32::from_le_bytes([bytes[3], bytes[4], bytes[5], 0]) + 1) * 10;
    let min_hfreq_khz = bytes[6];
    let max_hfreq_khz = bytes[7];
    let min_h_blanking = u16::from_le_bytes([bytes[8], bytes[9]]);
    let min_vfreq_hz = bytes[10];
    let max_vfreq_hz = bytes[11];
    let min_v_blanking = u16::from_le_bytes([bytes[12], bytes[13]]);
    let flags = bytes[14];

    if min_pixel_clock_khz > max_pixel_clock_khz {
        return Err(ExtensionError::InvalidDisplayIdDynamicRange {
            tag: block.tag,
            reason: "min pixel clock exceeds max pixel clock",
        });
    }
    if min_hfreq_khz > max_hfreq_khz {
        return Err(ExtensionError::InvalidDisplayIdDynamicRange {
            tag: block.tag,
            reason: "min horizontal frequency exceeds max",
        });
    }
    if min_vfreq_hz == 0 || max_vfreq_hz == 0 || min_vfreq_hz > max_vfreq_hz {
        return Err(ExtensionError::InvalidDisplayIdDynamicRange {
            tag: block.tag,
            reason: "min refresh rate exceeds max refresh rate or is zero",
        });
    }

    Ok(DisplayIdVideoTimingRangeLimits {
        min_pixel_clock_khz,
        max_pixel_clock_khz,
        min_hfreq_khz,
        max_hfreq_khz,
        min_h_blanking,
        min_vfreq_hz,
        max_vfreq_hz,
        min_v_blanking,
        supports_interlaced: flags & 0x80 != 0,
        supports_cvt: flags & 0x40 != 0,
        supports_cvt_reduced_blanking: flags & 0x20 != 0,
        discrete_frequency: flags & 0x10 != 0,
        raw: block.payload.clone(),
    })
}

fn decode_interface_features(
    block: &DisplayIdDataBlock,
) -> Result<DisplayIdInterfaceFeatures, ExtensionError> {
    if block.payload.len() < 9 {
        return Err(ExtensionError::InvalidDisplayIdDataBlockLength {
            tag: block.tag,
            length: block.payload.len(),
            minimum: 9,
            multiple: 0,
        });
    }
    let bytes = &block.payload;
    if bytes[8] > 7 {
        return Err(ExtensionError::InvalidDisplayIdFeatureField {
            field: "additional_colorspace_count",
            value: bytes[8],
            maximum: 7,
        });
    }
    Ok(DisplayIdInterfaceFeatures {
        color_depth_rgb: bytes[0],
        color_depth_ycbcr444: bytes[1],
        color_depth_ycbcr422: bytes[2],
        color_depth_ycbcr420: bytes[3],
        min_ycbcr420_pixel_rate: bytes[4],
        audio_flags: bytes[5],
        colorspace_eotf_1: bytes[6],
        colorspace_eotf_2: bytes[7],
        additional_colorspace_count: bytes[8],
        raw: bytes.to_vec(),
    })
}

fn decode_product_identification(
    block: &DisplayIdDataBlock,
) -> Result<DisplayIdProductIdentification, ExtensionError> {
    if block.payload.len() < 12 {
        return Err(ExtensionError::InvalidDisplayIdDataBlockLength {
            tag: block.tag,
            length: block.payload.len(),
            minimum: 12,
            multiple: 0,
        });
    }
    let bytes = &block.payload;
    let name_len = bytes[11] as usize;
    if 12 + name_len > bytes.len() {
        return Err(ExtensionError::InvalidDisplayIdDataBlockLength {
            tag: block.tag,
            length: bytes.len(),
            minimum: 12 + name_len,
            multiple: 0,
        });
    }
    Ok(DisplayIdProductIdentification {
        vendor_id: [bytes[0], bytes[1], bytes[2]],
        vendor_id_is_oui: block.tag == 0x20,
        product_code: u16::from_le_bytes([bytes[3], bytes[4]]),
        serial_number: u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]),
        week_of_manufacture: bytes[9],
        year: 2000 + u16::from(bytes[10]),
        product_name: bytes[12..12 + name_len].to_vec(),
        raw: bytes.to_vec(),
    })
}

fn decode_tiled_display_topology(
    block: &DisplayIdDataBlock,
) -> Result<DisplayIdTiledDisplayTopology, ExtensionError> {
    if block.payload.len() < 22 {
        return Err(ExtensionError::InvalidDisplayIdDataBlockLength {
            tag: block.tag,
            length: block.payload.len(),
            minimum: 22,
            multiple: 0,
        });
    }
    let b = &block.payload;
    let caps = b[0];
    let num_v_stored = (b[1] & 0x0F) | ((b[3] & 0x30) >> 4);
    let num_h_stored = ((b[1] >> 4) & 0x0F) | ((b[3] & 0xC0) >> 4);
    let tile_v_location = (b[2] & 0x0F) | ((b[3] & 0x03) << 4);
    let tile_h_location = ((b[2] >> 4) & 0x0F) | ((b[3] & 0x0C) << 2);
    let tile_width = u16::from_le_bytes([b[4], b[5]]) + 1;
    let tile_height = u16::from_le_bytes([b[6], b[7]]) + 1;
    let pixel_multiplier = b[8];
    let has_bevel = caps & 0x40 != 0;
    let (bevel_top, bevel_bottom, bevel_right, bevel_left) = if has_bevel {
        (Some(b[9]), Some(b[10]), Some(b[11]), Some(b[12]))
    } else {
        (None, None, None, None)
    };
    let tiles_h = num_h_stored + 1;
    let tiles_v = num_v_stored + 1;
    if tile_h_location >= tiles_h || tile_v_location >= tiles_v {
        return Err(ExtensionError::InvalidDisplayIdDynamicRange {
            tag: block.tag,
            reason: "tile location exceeds tile count",
        });
    }
    if !has_bevel && pixel_multiplier != 0 {
        return Err(ExtensionError::InvalidDisplayIdDynamicRange {
            tag: block.tag,
            reason: "bevel multiplier set without bevel info",
        });
    }
    Ok(DisplayIdTiledDisplayTopology {
        caps,
        tiles_h,
        tiles_v,
        tile_location_h: tile_h_location,
        tile_location_v: tile_v_location,
        tile_width,
        tile_height,
        pixel_multiplier,
        bevel_top,
        bevel_bottom,
        bevel_right,
        bevel_left,
        vendor_id: [b[13], b[14], b[15]],
        vendor_id_is_oui: block.tag == 0x28,
        product_code: u16::from_le_bytes([b[16], b[17]]),
        serial_number: u32::from_le_bytes([b[18], b[19], b[20], b[21]]),
        raw: b.to_vec(),
    })
}

fn decode_formula_timings(
    block: &DisplayIdDataBlock,
) -> Result<Vec<DisplayIdFormulaTiming>, ExtensionError> {
    if block.payload.len() < 6 || !block.payload.len().is_multiple_of(6) {
        return Err(ExtensionError::InvalidDisplayIdDataBlockLength {
            tag: block.tag,
            length: block.payload.len(),
            minimum: 6,
            multiple: 6,
        });
    }
    let mut timings = Vec::with_capacity(block.payload.len() / 6);
    for bytes in block.payload.as_chunks::<6>().0 {
        let options = bytes[0];
        timings.push(DisplayIdFormulaTiming {
            h_active: u16::from_le_bytes([bytes[1], bytes[2]]) + 1,
            v_active: u16::from_le_bytes([bytes[3], bytes[4]]) + 1,
            v_refresh_hz: u16::from(bytes[5]) + 1,
            formula: options & 0x07,
            ntsc_refresh: options & 0x10 != 0,
            stereo_3d: (options >> 5) & 0x03,
        });
    }
    Ok(timings)
}

fn decode_enumerated_timing_codes(block: &DisplayIdDataBlock) -> Result<Vec<u16>, ExtensionError> {
    let two_byte = block.revision & 0x08 != 0;
    let bytes = &block.payload;
    if bytes.is_empty() {
        return Err(ExtensionError::InvalidDisplayIdDataBlockLength {
            tag: block.tag,
            length: 0,
            minimum: 1,
            multiple: 0,
        });
    }
    if two_byte {
        if !bytes.len().is_multiple_of(2) {
            return Err(ExtensionError::InvalidDisplayIdDataBlockLength {
                tag: block.tag,
                length: bytes.len(),
                minimum: 2,
                multiple: 2,
            });
        }
        Ok(bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect())
    } else {
        Ok(bytes.iter().map(|&b| u16::from(b)).collect())
    }
}

/// Errors returned while reading an extension's structured view.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExtensionError {
    /// The block is not a CTA-861 extension.
    NotCta861,
    /// The block is not a DisplayID extension.
    NotDisplayId,
    /// The DisplayID payload cannot fit inside one EDID extension block.
    InvalidDisplayIdLength {
        /// Declared number of data-block bytes.
        length: usize,
        /// Maximum representable number of data-block bytes.
        maximum: usize,
    },
    /// The DisplayID section checksum is invalid.
    InvalidDisplayIdChecksum {
        /// Section byte sum modulo 256.
        sum: u8,
    },
    /// A DisplayID data-block declaration exceeds the bounded section.
    TruncatedDisplayIdDataBlock {
        /// Offset of the data-block header within the EDID block.
        offset: usize,
        /// DisplayID data-block tag.
        tag: u8,
        /// Declared payload length.
        length: usize,
        /// Payload bytes available before the section checksum.
        available: usize,
    },
    /// A known DisplayID data block has an unsupported payload shape.
    InvalidDisplayIdDataBlockLength {
        /// DisplayID data-block tag.
        tag: u8,
        /// Actual payload length.
        length: usize,
        /// Minimum payload length for the typed view.
        minimum: usize,
        /// Required entry multiple, or zero when no multiple applies.
        multiple: usize,
    },
    /// A DisplayID section ends with fewer than three bytes for a block header.
    TruncatedDisplayIdDataBlockHeader {
        /// Offset where the incomplete header starts.
        offset: usize,
        /// Header bytes available before the section checksum.
        available: usize,
    },
    /// A CTA data-block header declares bytes beyond the data-block collection.
    TruncatedDataBlock {
        /// Offset of the data-block header within the extension.
        offset: usize,
        /// Declared payload length.
        length: usize,
    },
    /// CTA's DTD offset is outside the block.
    InvalidDtdOffset {
        /// Raw CTA DTD offset byte.
        offset: usize,
    },
    /// CTA data block payload has an invalid audio length.
    InvalidAudioDataBlockLength {
        /// Actual payload length.
        length: usize,
    },
    /// CTA video data block has no SVD entries.
    InvalidVideoDataBlockLength {
        /// Actual payload length.
        length: usize,
    },
    /// CTA video data block contains a zero VIC.
    InvalidVideoCode {
        /// Zero-based payload index.
        index: usize,
    },
    /// CTA extended data block payload is shorter than its known minimum.
    TruncatedExtendedDataBlock {
        /// Extended tag code.
        extended_tag: u8,
        /// Actual payload length including the extended tag.
        length: usize,
        /// Minimum payload length including the extended tag.
        minimum: usize,
    },
    /// CTA vendor-specific data block payload is shorter than 3-byte OUI.
    TruncatedVendorSpecificDataBlock {
        /// Actual payload length.
        length: usize,
    },
    /// CTA speaker allocation data block has an invalid length.
    InvalidSpeakerAllocationDataBlockLength {
        /// Actual payload length.
        length: usize,
    },
    /// A CTA YCbCr 4:2:0 Capability Map references an SVD index that does not exist in the collection.
    Y420CapabilityMapIndexOutOfRange {
        /// SVD index indicated by the capability map (zero-based).
        index: usize,
        /// Total number of SVD entries available in regular Video Data Blocks.
        available_svds: usize,
    },
    /// A CTA YCbCr 4:2:0 Capability Map is present but no Video Data Block exists.
    Y420CapabilityMapMissingVideoDataBlock,
    /// CTA Adaptive-Sync block has an invalid refresh rate range.
    InvalidRefreshRateRange {
        /// Minimum refresh rate in Hz.
        min_refresh_hz: u8,
        /// Maximum refresh rate in Hz.
        max_refresh_hz: u8,
    },
    /// DisplayID Dynamic Video Timing Range block has an invalid range (min > max or out of range).
    InvalidDisplayIdDynamicRange {
        /// DisplayID data-block tag.
        tag: u8,
        /// Detail description of the invalid range constraint.
        reason: &'static str,
    },
    /// A DisplayID interface feature field is out of its representable range.
    InvalidDisplayIdFeatureField {
        /// Field name.
        field: &'static str,
        /// Supplied value.
        value: u8,
        /// Maximum representable value.
        maximum: u8,
    },
    /// A DisplayID product-identification field is out of its representable range.
    InvalidDisplayIdProductField {
        /// Field name.
        field: &'static str,
        /// Supplied value.
        value: u32,
        /// Maximum representable value.
        maximum: u32,
    },
    /// A CTA HDR dynamic metadata entry has an invalid length (type_len < 2).
    InvalidDynamicHdrMetadataLength {
        /// Zero-based entry index.
        index: usize,
        /// Declared entry length byte.
        length: usize,
    },
    /// A CTA HDR dynamic metadata entry is truncated relative to the block payload.
    TruncatedDynamicHdrMetadataEntry {
        /// Zero-based entry index.
        index: usize,
        /// Available bytes.
        length: usize,
        /// Minimum required bytes.
        minimum: usize,
    },
}

impl std::fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotCta861 => f.write_str("EDID block is not a CTA-861 extension"),
            Self::NotDisplayId => f.write_str("EDID block is not a DisplayID extension"),
            Self::InvalidDisplayIdLength { length, maximum } => write!(
                f,
                "DisplayID data-block length {length} exceeds maximum {maximum}"
            ),
            Self::InvalidDisplayIdChecksum { sum } => {
                write!(f, "invalid DisplayID section checksum: byte sum is {sum}")
            }
            Self::TruncatedDisplayIdDataBlock {
                offset,
                tag,
                length,
                available,
            } => write!(
                f,
                "DisplayID data block tag 0x{tag:02X} at offset {offset} declares {length} bytes, only {available} available"
            ),
            Self::InvalidDisplayIdDataBlockLength {
                tag,
                length,
                minimum,
                multiple,
            } => write!(
                f,
                "DisplayID data block tag 0x{tag:02X} has length {length}, minimum {minimum}, multiple {multiple}"
            ),
            Self::TruncatedDisplayIdDataBlockHeader { offset, available } => write!(
                f,
                "DisplayID data-block header at offset {offset} has only {available} bytes"
            ),
            Self::InvalidDisplayIdDynamicRange { tag, reason } => write!(
                f,
                "DisplayID dynamic range data block 0x{tag:02X} has invalid range: {reason}"
            ),
            Self::InvalidDisplayIdFeatureField {
                field,
                value,
                maximum,
            } => write!(
                f,
                "DisplayID interface feature field {field} value {value} exceeds maximum {maximum}"
            ),
            Self::InvalidDisplayIdProductField {
                field,
                value,
                maximum,
            } => write!(
                f,
                "DisplayID product-identification field {field} value {value} exceeds maximum {maximum}"
            ),
            Self::InvalidDynamicHdrMetadataLength { index, length } => write!(
                f,
                "CTA HDR dynamic metadata entry {index} has invalid length {length}"
            ),
            Self::TruncatedDynamicHdrMetadataEntry {
                index,
                length,
                minimum,
            } => write!(
                f,
                "CTA HDR dynamic metadata entry {index} needs {minimum} bytes but only {length} available"
            ),
            Self::TruncatedDataBlock { offset, length } => write!(
                f,
                "CTA data block at offset {offset} declares {length} bytes beyond the collection"
            ),
            Self::InvalidDtdOffset { offset } => {
                write!(f, "CTA DTD offset {offset} is outside the extension")
            }
            Self::InvalidAudioDataBlockLength { length } => {
                write!(
                    f,
                    "CTA audio data block has invalid payload length {length}"
                )
            }
            Self::InvalidVideoDataBlockLength { length } => {
                write!(
                    f,
                    "CTA video data block has invalid payload length {length}"
                )
            }
            Self::InvalidVideoCode { index } => {
                write!(
                    f,
                    "CTA video data block has zero VIC at payload index {index}"
                )
            }
            Self::TruncatedExtendedDataBlock {
                extended_tag,
                length,
                minimum,
            } => write!(
                f,
                "CTA extended data block tag 0x{extended_tag:02X} has length {length}, minimum is {minimum}"
            ),
            Self::TruncatedVendorSpecificDataBlock { length } => write!(
                f,
                "CTA vendor-specific data block has payload length {length}, minimum is 3 (OUI)"
            ),
            Self::InvalidSpeakerAllocationDataBlockLength { length } => write!(
                f,
                "CTA speaker allocation data block has invalid payload length {length}"
            ),
            Self::Y420CapabilityMapIndexOutOfRange {
                index,
                available_svds,
            } => write!(
                f,
                "CTA Y420 capability map references SVD index {index} but only {available_svds} are available"
            ),
            Self::Y420CapabilityMapMissingVideoDataBlock => {
                f.write_str("CTA Y420 capability map is present without any Video Data Block")
            }
            Self::InvalidRefreshRateRange {
                min_refresh_hz,
                max_refresh_hz,
            } => write!(
                f,
                "invalid refresh rate range: {min_refresh_hz} Hz .. {max_refresh_hz} Hz"
            ),
        }
    }
}

impl std::error::Error for ExtensionError {}

fn map_cta_extension_write_error(error: ExtensionError) -> ExtensionWriteError {
    match error {
        ExtensionError::NotCta861 => ExtensionWriteError::NotCta861,
        ExtensionError::InvalidDtdOffset { offset } => {
            ExtensionWriteError::InvalidDtdOffset { offset }
        }
        source => ExtensionWriteError::InvalidCtaLayout { source },
    }
}

impl EdidBlock {
    /// Identify this block as CTA-861, DisplayID or unknown.
    #[must_use]
    pub fn extension_kind(&self) -> ExtensionKind {
        match self.raw[0] {
            0x02 => ExtensionKind::Cta861 {
                revision: self.raw[1],
            },
            0x70 => ExtensionKind::DisplayId {
                version: self.raw[1],
            },
            tag => ExtensionKind::Unknown { tag },
        }
    }
    /// Construct a CTA-861 extension containing data blocks and DTDs.
    pub fn from_cta_data_blocks_and_timings(
        revision: u8,
        blocks: &[CtaDataBlock],
        timings: &[crate::timing::DetailedTiming],
    ) -> Result<Self, ExtensionWriteError> {
        const DATA_BLOCK_OFFSET: usize = 4;
        const DTD_SIZE: usize = 18;
        const MAX_COLLECTION_LENGTH: usize = 123;

        let mut total_collection_length = 0usize;
        for block in blocks {
            if block.tag > 0x07 {
                return Err(ExtensionWriteError::InvalidCtaTag { tag: block.tag });
            }
            if block.payload.len() > 0x1F {
                return Err(ExtensionWriteError::CtaPayloadTooLong {
                    length: block.payload.len(),
                    maximum: 0x1F,
                });
            }
            total_collection_length =
                total_collection_length.saturating_add(1 + block.payload.len());
        }

        if total_collection_length > MAX_COLLECTION_LENGTH {
            return Err(ExtensionWriteError::CtaDataBlocksTooLong {
                length: total_collection_length,
                maximum: MAX_COLLECTION_LENGTH,
            });
        }

        let dtd_offset = DATA_BLOCK_OFFSET + total_collection_length;
        let maximum = (127usize.saturating_sub(dtd_offset)) / DTD_SIZE;
        if timings.len() > maximum {
            return Err(ExtensionWriteError::CtaDtdsTooLong {
                count: timings.len(),
                maximum,
            });
        }

        let mut collection = Vec::with_capacity(total_collection_length);
        for data_block in blocks {
            collection.extend_from_slice(&data_block.encode()?);
        }

        let mut block = Self {
            raw: [0; crate::edid::EDID_BLOCK_SIZE],
        };
        block.raw[0] = 0x02;
        block.raw[1] = revision;
        block.raw[2] = dtd_offset as u8;
        block.raw[3] = timings.len() as u8;
        block.raw[DATA_BLOCK_OFFSET..dtd_offset].copy_from_slice(&collection);

        for (index, timing) in timings.iter().enumerate() {
            let mut encoded = crate::edid::EdidBlock::new_default();
            encoded
                .write_detailed_checked(0, timing)
                .map_err(|source| ExtensionWriteError::CtaDtdInvalid { index, source })?;
            let start = dtd_offset + index * DTD_SIZE;
            block.raw[start..start + DTD_SIZE].copy_from_slice(&encoded.raw[54..54 + DTD_SIZE]);
        }
        block.update_checksum();
        Ok(block)
    }
    /// Replace the CTA data-block collection and clear existing CTA DTDs.
    ///
    /// The revision is preserved; all derived offsets, padding, native count,
    /// and checksum are regenerated by the checked constructor.
    pub fn replace_cta_data_blocks(
        &mut self,
        blocks: &[CtaDataBlock],
    ) -> Result<(), ExtensionWriteError> {
        if self.raw[0] != 0x02 {
            return Err(ExtensionWriteError::NotCta861);
        }
        let old_flags = self.raw[3] & 0xF0;
        let mut rebuilt = Self::from_cta_data_blocks(self.raw[1], blocks)?;
        rebuilt.raw[3] = old_flags;
        rebuilt.update_checksum();
        *self = rebuilt;
        Ok(())
    }

    /// Construct a DisplayID extension containing raw data blocks.
    pub fn from_display_id_data_blocks(
        revision: u8,
        product_type_or_primary_use: u8,
        extension_count: u8,
        blocks: &[DisplayIdDataBlock],
    ) -> Result<Self, ExtensionWriteError> {
        const DATA_OFFSET: usize = 5;

        let mut total_len = 0usize;
        for block in blocks {
            if block.payload.len() > u8::MAX as usize {
                return Err(ExtensionWriteError::DisplayIdPayloadTooLong {
                    length: block.payload.len(),
                    maximum: u8::MAX as usize,
                });
            }
            total_len = total_len.saturating_add(3 + block.payload.len());
        }
        check_display_id_payload_length(total_len)?;

        let mut payload = Vec::with_capacity(total_len);
        for data_block in blocks {
            payload.extend_from_slice(&data_block.encode()?);
        }

        let mut block = Self {
            raw: [0; crate::edid::EDID_BLOCK_SIZE],
        };
        block.raw[0] = 0x70;
        block.raw[1] = revision;
        block.raw[2] = payload.len() as u8;
        block.raw[3] = product_type_or_primary_use;
        block.raw[4] = extension_count;
        block.raw[DATA_OFFSET..DATA_OFFSET + payload.len()].copy_from_slice(&payload);
        let section_checksum = DATA_OFFSET + payload.len();
        let sum = block.raw[1..section_checksum]
            .iter()
            .fold(0u8, |sum, &byte| sum.wrapping_add(byte));
        block.raw[section_checksum] = 0u8.wrapping_sub(sum);
        block.update_checksum();
        Ok(block)
    }
    /// Replace DisplayID data blocks while preserving section header fields.
    pub fn replace_display_id_data_blocks(
        &mut self,
        blocks: &[DisplayIdDataBlock],
    ) -> Result<(), ExtensionWriteError> {
        if self.raw[0] != 0x70 {
            return Err(ExtensionWriteError::NotDisplayId);
        }
        let rebuilt =
            Self::from_display_id_data_blocks(self.raw[1], self.raw[3], self.raw[4], blocks)?;
        *self = rebuilt;
        Ok(())
    }

    /// Re-order DisplayID data blocks according to the ordering policy.
    ///
    /// `PreserveSourceOrder` is a no-op that keeps the block bytes unchanged;
    /// `Canonical` deterministically sorts the data blocks and is idempotent.
    pub fn reorder_display_id_data_blocks(
        &mut self,
        ordering: DisplayIdOrdering,
    ) -> Result<(), ExtensionWriteError> {
        if matches!(ordering, DisplayIdOrdering::PreserveSourceOrder) {
            return Ok(());
        }
        let mut blocks = self
            .display_id_data_blocks()
            .map_err(|source| ExtensionWriteError::InvalidDisplayIdLayout { source })?;
        blocks
            .sort_by(|a, b| (a.tag, a.revision, &a.payload).cmp(&(b.tag, b.revision, &b.payload)));
        self.replace_display_id_data_blocks(&blocks)
    }

    /// Construct a CTA-861 extension containing only a data-block collection.
    pub fn from_cta_data_blocks(
        revision: u8,
        blocks: &[CtaDataBlock],
    ) -> Result<Self, ExtensionWriteError> {
        Self::from_cta_data_blocks_and_timings(revision, blocks, &[])
    }

    /// Read and validate the DisplayID base-section header.
    pub fn display_id_header(&self) -> Result<DisplayIdHeader, ExtensionError> {
        if self.raw[0] != 0x70 {
            return Err(ExtensionError::NotDisplayId);
        }
        let payload_length = self.raw[2] as usize;
        const DISPLAY_ID_DATA_OFFSET: usize = 5;
        if payload_length > MAX_DISPLAY_ID_PAYLOAD {
            return Err(ExtensionError::InvalidDisplayIdLength {
                length: payload_length,
                maximum: MAX_DISPLAY_ID_PAYLOAD,
            });
        }
        let checksum_offset = DISPLAY_ID_DATA_OFFSET + payload_length;
        let sum = self.raw[1..=checksum_offset]
            .iter()
            .fold(0u8, |sum, &byte| sum.wrapping_add(byte));
        if sum != 0 {
            return Err(ExtensionError::InvalidDisplayIdChecksum { sum });
        }
        Ok(DisplayIdHeader {
            revision: self.raw[1],
            payload_length,
            product_type_or_primary_use: self.raw[3],
            extension_count: self.raw[4],
        })
    }

    /// Read bounded DisplayID data blocks without modifying the block.
    pub fn display_id_data_blocks(&self) -> Result<Vec<DisplayIdDataBlock>, ExtensionError> {
        let header = self.display_id_header()?;
        parse_display_id_data_blocks(&self.raw[5..5 + header.payload_length], 5)
    }

    /// Read all Dynamic Video Timing Range Limits from DisplayID extension blocks.
    pub fn display_id_dynamic_video_timing_ranges(
        &self,
    ) -> Result<Vec<DisplayIdDynamicVideoTimingRange>, ExtensionError> {
        let blocks = self.display_id_data_blocks()?;
        let mut ranges = Vec::new();
        for block in blocks {
            if let DisplayIdDataBlockView::DynamicVideoTimingRange { range } = block.view()? {
                ranges.push(range);
            }
        }
        Ok(ranges)
    }

    /// Read all 1.x Video Timing Range Limits from DisplayID extension blocks.
    pub fn display_id_video_timing_range_limits(
        &self,
    ) -> Result<Vec<DisplayIdVideoTimingRangeLimits>, ExtensionError> {
        let blocks = self.display_id_data_blocks()?;
        let mut ranges = Vec::new();
        for block in blocks {
            if let DisplayIdDataBlockView::VideoTimingRange1x { range } = block.view()? {
                ranges.push(range);
            }
        }
        Ok(ranges)
    }

    /// Read all Display Interface Features from DisplayID extension blocks.
    pub fn display_id_interface_features(
        &self,
    ) -> Result<Vec<DisplayIdInterfaceFeatures>, ExtensionError> {
        let blocks = self.display_id_data_blocks()?;
        let mut features = Vec::new();
        for block in blocks {
            if let DisplayIdDataBlockView::InterfaceFeatures { features: f } = block.view()? {
                features.push(f);
            }
        }
        Ok(features)
    }

    /// Read all Product Identification blocks from DisplayID extension blocks.
    pub fn display_id_product_identifications(
        &self,
    ) -> Result<Vec<DisplayIdProductIdentification>, ExtensionError> {
        let blocks = self.display_id_data_blocks()?;
        let mut products = Vec::new();
        for block in blocks {
            if let DisplayIdDataBlockView::ProductIdentification { product } = block.view()? {
                products.push(product);
            }
        }
        Ok(products)
    }

    /// Read all Tiled Display Topology blocks from DisplayID extension blocks.
    pub fn display_id_tiled_topologies(
        &self,
    ) -> Result<Vec<DisplayIdTiledDisplayTopology>, ExtensionError> {
        let blocks = self.display_id_data_blocks()?;
        let mut topologies = Vec::new();
        for block in blocks {
            if let DisplayIdDataBlockView::TiledDisplayTopology { topology } = block.view()? {
                topologies.push(topology);
            }
        }
        Ok(topologies)
    }

    /// Read all Type IX formula-based timings from DisplayID extension blocks.
    pub fn display_id_formula_timings(
        &self,
    ) -> Result<Vec<DisplayIdFormulaTiming>, ExtensionError> {
        let blocks = self.display_id_data_blocks()?;
        let mut timings = Vec::new();
        for block in blocks {
            if let DisplayIdDataBlockView::FormulaTiming { timings: t } = block.view()? {
                timings.extend(t);
            }
        }
        Ok(timings)
    }

    /// Read all Type VIII enumerated timing codes from DisplayID extension blocks.
    pub fn display_id_enumerated_timings(
        &self,
    ) -> Result<Vec<DisplayIdEnumeratedTiming>, ExtensionError> {
        let blocks = self.display_id_data_blocks()?;
        let mut timings = Vec::new();
        for block in blocks {
            if let DisplayIdDataBlockView::EnumeratedTiming {
                code_type, codes, ..
            } = block.view()?
            {
                timings.extend(
                    codes
                        .into_iter()
                        .map(|code| DisplayIdEnumeratedTiming { code_type, code }),
                );
            }
        }
        Ok(timings)
    }

    /// Read detailed timings from this DisplayID extension block.
    pub fn display_id_detailed_timings(
        &self,
    ) -> Result<Vec<DisplayIdDetailedTiming>, ExtensionError> {
        let blocks = self.display_id_data_blocks()?;
        let mut timings = Vec::new();
        for block in blocks {
            if let DisplayIdDataBlockView::DetailedTiming {
                timings: block_timings,
            } = block.view()?
            {
                timings.extend(block_timings);
            }
        }
        Ok(timings)
    }

    /// Read the CTA-861 data-block collection without modifying the block.
    pub fn cta_data_blocks(&self) -> Result<Vec<CtaDataBlock>, ExtensionError> {
        if self.raw[0] != 0x02 {
            return Err(ExtensionError::NotCta861);
        }
        let dtd_offset = self.raw[2] as usize;
        let end = if dtd_offset == 0 { 127 } else { dtd_offset };
        if !(4..=127).contains(&end) {
            return Err(ExtensionError::InvalidDtdOffset { offset: end });
        }
        parse_cta_data_blocks(&self.raw[4..end], 4, dtd_offset == 0)
    }

    /// Query resolved YCbCr 4:2:0 capabilities for this CTA-861 block.
    pub fn cta_y420_support(&self) -> Result<CtaY420Support, ExtensionError> {
        let blocks = self.cta_data_blocks()?;
        CtaY420Support::resolve_from_blocks(&blocks)
    }

    /// Read and validate the CTA-861 extension header and capability flags.
    pub fn cta_header(&self) -> Result<CtaHeader, ExtensionError> {
        if self.raw[0] != 0x02 {
            return Err(ExtensionError::NotCta861);
        }
        let dtd_offset = self.raw[2];
        if dtd_offset != 0 && !(4..=127).contains(&dtd_offset) {
            return Err(ExtensionError::InvalidDtdOffset {
                offset: dtd_offset as usize,
            });
        }
        let flags = self.raw[3];
        Ok(CtaHeader {
            revision: self.raw[1],
            dtd_offset,
            native_dtd_count: flags & 0x0F,
            underscan: flags & 0x80 != 0,
            basic_audio: flags & 0x40 != 0,
            ycbcr_444: flags & 0x20 != 0,
            ycbcr_422: flags & 0x10 != 0,
        })
    }

    /// Replace CTA capability flags and native DTD count without changing layout.
    pub fn set_cta_header(&mut self, header: CtaHeader) -> Result<(), ExtensionWriteError> {
        let current = self.cta_header().map_err(map_cta_extension_write_error)?;
        let current_offset = current.dtd_offset;
        if header.dtd_offset != current_offset {
            return Err(ExtensionWriteError::InvalidDtdOffset {
                offset: header.dtd_offset as usize,
            });
        }
        // Validate the existing collection before creating a candidate block.
        self.cta_data_blocks()
            .map_err(map_cta_extension_write_error)?;
        let populated_dtds = self
            .cta_detailed_timings_flagged()
            .map_err(map_cta_extension_write_error)?
            .len();
        let maximum_native = populated_dtds.min(0x0F);
        if usize::from(header.native_dtd_count) > maximum_native {
            return Err(ExtensionWriteError::CtaDtdsTooLong {
                count: usize::from(header.native_dtd_count),
                maximum: maximum_native,
            });
        }

        let mut candidate = self.clone();
        candidate.raw[1] = header.revision;
        candidate.raw[3] = u8::from(header.underscan) << 7
            | u8::from(header.basic_audio) << 6
            | u8::from(header.ycbcr_444) << 5
            | u8::from(header.ycbcr_422) << 4
            | header.native_dtd_count;
        candidate.update_checksum();
        *self = candidate;
        Ok(())
    }

    /// Set CTA capability flags while retaining the current revision, layout, and native count.
    pub fn set_cta_capabilities(
        &mut self,
        underscan: bool,
        basic_audio: bool,
        ycbcr_444: bool,
        ycbcr_422: bool,
    ) -> Result<(), ExtensionWriteError> {
        let header = self.cta_header().map_err(map_cta_extension_write_error)?;
        self.set_cta_header(CtaHeader {
            revision: header.revision,
            dtd_offset: header.dtd_offset,
            native_dtd_count: header.native_dtd_count,
            underscan,
            basic_audio,
            ycbcr_444,
            ycbcr_422,
        })
    }

    /// Replace the CTA DTD collection, rebuilding its offset, native count, layout, and checksum.
    pub fn replace_cta_detailed_timings(
        &mut self,
        timings: &[crate::timing::DetailedTiming],
    ) -> Result<(), ExtensionWriteError> {
        if self.raw[0] != 0x02 {
            return Err(ExtensionWriteError::NotCta861);
        }
        let blocks = self
            .cta_data_blocks()
            .map_err(map_cta_extension_write_error)?;
        let old_flags = self.raw[3] & 0xF0;
        let mut rebuilt = Self::from_cta_data_blocks_and_timings(self.raw[1], &blocks, timings)?;
        rebuilt.raw[3] = old_flags | rebuilt.raw[3] & 0x0F;
        rebuilt.update_checksum();
        *self = rebuilt;
        Ok(())
    }

    /// Read progressive Detailed Timing Descriptors from this CTA-861 extension block.
    pub fn cta_detailed_timings(
        &self,
    ) -> Result<Vec<crate::timing::DetailedTiming>, ExtensionError> {
        let flagged = self.cta_detailed_timings_flagged()?;
        Ok(flagged
            .into_iter()
            .filter_map(|dtd| (!dtd.flags.interlaced()).then_some(dtd.timing))
            .collect())
    }

    /// Read Detailed Timing Descriptors with flags from this CTA-861 extension block.
    pub fn cta_detailed_timings_flagged(
        &self,
    ) -> Result<Vec<crate::edid::DecodedDtd>, ExtensionError> {
        if self.raw[0] != 0x02 {
            return Err(ExtensionError::NotCta861);
        }
        let dtd_offset = self.raw[2] as usize;
        if dtd_offset == 0 {
            return Ok(Vec::new());
        }
        if !(4..=127).contains(&dtd_offset) {
            return Err(ExtensionError::InvalidDtdOffset { offset: dtd_offset });
        }
        let mut dtds = Vec::new();
        let mut offset = dtd_offset;
        while offset + 18 <= 127 {
            let slice = &self.raw[offset..offset + 18];
            if slice[0] == 0 && slice[1] == 0 {
                break;
            }
            if let Some(decoded) = crate::edid::decode_dtd_bytes(slice) {
                dtds.push(decoded);
            }
            offset += 18;
        }
        Ok(dtds)
    }
}

fn parse_display_id_data_blocks(
    data: &[u8],
    base_offset: usize,
) -> Result<Vec<DisplayIdDataBlock>, ExtensionError> {
    let mut blocks = Vec::new();
    let mut offset = 0;
    while offset < data.len() {
        let available_header = data.len() - offset;
        if available_header < 3 {
            return Err(ExtensionError::TruncatedDisplayIdDataBlockHeader {
                offset: base_offset + offset,
                available: available_header,
            });
        }
        let tag = data[offset];
        let revision = data[offset + 1];
        let length = data[offset + 2] as usize;
        let payload_start = offset + 3;
        let available = data.len() - payload_start;
        if length > available {
            return Err(ExtensionError::TruncatedDisplayIdDataBlock {
                offset: base_offset + offset,
                tag,
                length,
                available,
            });
        }
        blocks.push(DisplayIdDataBlock {
            tag,
            revision,
            payload: data[payload_start..payload_start + length].to_vec(),
        });
        offset = payload_start + length;
    }
    Ok(blocks)
}

fn parse_cta_data_blocks(
    data: &[u8],
    base_offset: usize,
    stop_at_zero_padding: bool,
) -> Result<Vec<CtaDataBlock>, ExtensionError> {
    let mut blocks = Vec::new();
    let mut offset = 0;
    while offset < data.len() {
        let header = data[offset];
        if header == 0 && stop_at_zero_padding {
            break;
        }
        let tag = header >> 5;
        let length = (header & 0x1F) as usize;
        let payload_start = offset + 1;
        let payload_end = payload_start + length;
        if payload_end > data.len() {
            return Err(ExtensionError::TruncatedDataBlock {
                offset: base_offset + offset,
                length,
            });
        }
        blocks.push(CtaDataBlock {
            tag,
            payload: data[payload_start..payload_end].to_vec(),
        });
        offset = payload_end;
    }
    Ok(blocks)
}

#[cfg(test)]
mod tests {
    use super::{
        CtaAdaptiveSync, CtaAudioDescriptor, CtaAudioFormat, CtaColorimetry, CtaDataBlock,
        CtaDataBlockView, CtaExtendedDataBlockView, CtaSpeakerAllocation, CtaVendorSpecificBlock,
        CtaVideoCapability, CtaVideoMode, CtaY420Support, DisplayIdAspectRatio, DisplayIdDataBlock,
        DisplayIdDataBlockView, DisplayIdDetailedTiming, DisplayIdDisplayParameters,
        DisplayIdDynamicVideoTimingRange, DisplayIdFormulaTiming, DisplayIdHeader,
        DisplayIdInterfaceFeatures, DisplayIdProductIdentification, DisplayIdStereo3d,
        DisplayIdTiledDisplayTopology, ExtensionError, ExtensionKind, ExtensionWriteError,
    };
    use crate::edid::EdidBlock;

    #[test]
    fn identifies_cta_and_reads_data_blocks() {
        let mut block = EdidBlock::new_default();
        block.raw[0] = 0x02;
        block.raw[1] = 3;
        block.raw[2] = 9;
        block.raw[4] = (2 << 5) | 3;
        block.raw[5..8].copy_from_slice(&[0x01, 0x02, 0x03]);
        block.update_checksum();

        assert_eq!(
            block.extension_kind(),
            ExtensionKind::Cta861 { revision: 3 }
        );
        assert_eq!(
            block.cta_data_blocks().unwrap()[0].payload,
            vec![0x01, 0x02, 0x03]
        );
    }

    #[test]
    fn rejects_truncated_cta_data_block() {
        let mut block = EdidBlock::new_default();
        block.raw[0] = 0x02;
        block.raw[2] = 6;
        block.raw[4] = (1 << 5) | 3;
        block.update_checksum();
        assert!(matches!(
            block.cta_data_blocks(),
            Err(ExtensionError::TruncatedDataBlock {
                offset: 4,
                length: 3
            })
        ));
    }

    #[test]
    fn identifies_displayid_and_unknown_extensions() {
        let mut display_id = EdidBlock::new_default();
        display_id.raw[0] = 0x70;
        display_id.raw[1] = 0x20;
        assert_eq!(
            display_id.extension_kind(),
            ExtensionKind::DisplayId { version: 0x20 }
        );

        let mut unknown = EdidBlock::new_default();
        unknown.raw[0] = 0x99;
        assert_eq!(
            unknown.extension_kind(),
            ExtensionKind::Unknown { tag: 0x99 }
        );
    }

    #[test]
    fn typed_cta_views_decode_video_audio_hdr_and_adaptive_sync() {
        let video = CtaDataBlock {
            tag: 2,
            payload: vec![0x80 | 16, 16],
        };
        assert_eq!(
            video.view().unwrap(),
            CtaDataBlockView::Video {
                modes: vec![
                    CtaVideoMode {
                        vic: 16,
                        native: true,
                    },
                    CtaVideoMode {
                        vic: 16,
                        native: false,
                    },
                ],
            }
        );

        let audio = CtaDataBlock {
            tag: 1,
            payload: vec![0x09, 0x07, 0x07, 0x15, 0x07, 0x0F],
        };
        match audio.view().unwrap() {
            CtaDataBlockView::Audio { descriptors } => {
                assert_eq!(descriptors.len(), 2);
                assert_eq!(descriptors[0].format, 1);
                assert_eq!(descriptors[0].channels, 2);
                assert_eq!(descriptors[0].sample_rates, 0x07);
                assert_eq!(descriptors[0].format_specific, 0x07);
            }
            other => panic!("unexpected CTA view: {other:?}"),
        }

        let hdr = CtaDataBlock {
            tag: 7,
            payload: vec![0x06, 0x07, 0x01, 0x40, 0x20, 0x01],
        };
        assert_eq!(
            hdr.view().unwrap(),
            CtaDataBlockView::Extended(CtaExtendedDataBlockView::HdrStaticMetadata {
                eotf_flags: 0x07,
                metadata_descriptor_flags: 0x01,
                max_luminance: Some(0x40),
                max_frame_average_luminance: Some(0x20),
                min_luminance: Some(0x01),
                raw: vec![0x06, 0x07, 0x01, 0x40, 0x20, 0x01],
            })
        );

        let adaptive_sync = CtaDataBlock {
            tag: 7,
            payload: vec![0x1A, 0x01, 48, 144],
        };
        assert_eq!(
            adaptive_sync.view().unwrap(),
            CtaDataBlockView::Extended(CtaExtendedDataBlockView::AdaptiveSync(CtaAdaptiveSync {
                flags: 0x01,
                min_refresh_hz: 48,
                max_refresh_hz: 144,
                raw: vec![0x1A, 0x01, 48, 144],
            }))
        );
    }

    #[test]
    fn cta_adaptive_sync_roundtrip_modification_and_boundary_rejection() {
        // Parse valid block with extra tail bytes
        let block = CtaDataBlock {
            tag: 7,
            payload: vec![0x1A, 0x05, 48, 144, 0xDE, 0xAD],
        };
        let view = block.view().unwrap();
        let CtaDataBlockView::Extended(CtaExtendedDataBlockView::AdaptiveSync(mut sync)) = view
        else {
            panic!("expected AdaptiveSync view");
        };
        assert_eq!(sync.flags, 0x05);
        assert_eq!(sync.min_refresh_hz, 48);
        assert_eq!(sync.max_refresh_hz, 144);
        assert_eq!(sync.raw, vec![0x1A, 0x05, 48, 144, 0xDE, 0xAD]);

        // Unmodified round-trip preserves raw tail
        let encoded =
            CtaDataBlockView::Extended(CtaExtendedDataBlockView::AdaptiveSync(sync.clone()))
                .to_data_block()
                .unwrap();
        assert_eq!(encoded, block);

        // Modifying fields updates payload while preserving tail
        sync.min_refresh_hz = 60;
        sync.max_refresh_hz = 240;
        sync.flags = 0x07;
        let modified_block =
            CtaDataBlockView::Extended(CtaExtendedDataBlockView::AdaptiveSync(sync.clone()))
                .to_data_block()
                .unwrap();
        assert_eq!(
            modified_block.payload,
            vec![0x1A, 0x07, 60, 240, 0xDE, 0xAD]
        );

        // Constructor creates valid block
        let created = CtaAdaptiveSync::new(1, 255, 0x01).unwrap();
        assert_eq!(created.min_refresh_hz, 1);
        assert_eq!(created.max_refresh_hz, 255);
        assert_eq!(created.raw, vec![0x1A, 0x01, 1, 255]);

        // Constructor rejects reverse range and zero
        assert!(matches!(
            CtaAdaptiveSync::new(144, 48, 0),
            Err(ExtensionWriteError::InvalidRefreshRateRange {
                min_refresh_hz: 144,
                max_refresh_hz: 48
            })
        ));
        assert!(matches!(
            CtaAdaptiveSync::new(0, 144, 0),
            Err(ExtensionWriteError::InvalidRefreshRateRange {
                min_refresh_hz: 0,
                max_refresh_hz: 144
            })
        ));

        // Parser rejects reverse range, zero, and truncated payloads
        let reverse_block = CtaDataBlock {
            tag: 7,
            payload: vec![0x1A, 0x01, 144, 48],
        };
        assert!(matches!(
            reverse_block.view(),
            Err(ExtensionError::InvalidRefreshRateRange {
                min_refresh_hz: 144,
                max_refresh_hz: 48
            })
        ));
        let zero_block = CtaDataBlock {
            tag: 7,
            payload: vec![0x1A, 0x01, 0, 144],
        };
        assert!(matches!(
            zero_block.view(),
            Err(ExtensionError::InvalidRefreshRateRange {
                min_refresh_hz: 0,
                max_refresh_hz: 144
            })
        ));
        let truncated_block = CtaDataBlock {
            tag: 7,
            payload: vec![0x1A, 0x01, 48],
        };
        assert!(matches!(
            truncated_block.view(),
            Err(ExtensionError::TruncatedExtendedDataBlock {
                extended_tag: 0x1A,
                length: 3,
                minimum: 4
            })
        ));
    }
    #[test]
    fn cta_audio_descriptor_semantic_accessors_and_roundtrip() {
        // LPCM: 2 channels, 48/44.1/32 kHz, 24/20/16-bit.
        let lpcm = CtaAudioDescriptor {
            format: 1,
            channels: 2,
            sample_rates: 0b000_0111,
            format_specific: 0b111,
        };
        assert_eq!(lpcm.format_kind(), CtaAudioFormat::Lpcm);
        assert!(lpcm.supports_sample_rate(48));
        assert!(lpcm.supports_sample_rate(44));
        assert!(lpcm.supports_sample_rate(32));
        assert!(!lpcm.supports_sample_rate(192));
        assert!(lpcm.lpcm_supports_sample_size(16));
        assert!(lpcm.lpcm_supports_sample_size(20));
        assert!(lpcm.lpcm_supports_sample_size(24));
        assert_eq!(lpcm.lpcm_sample_size_bits(), &[16, 20, 24]);

        // A descriptor supporting only 16+20-bit must not report 24-bit.
        let partial = CtaAudioDescriptor {
            format: 1,
            channels: 2,
            sample_rates: 0b000_0111,
            format_specific: 0b011,
        };
        assert_eq!(partial.lpcm_sample_size_bits(), &[16, 20]);
        assert!(partial.lpcm_supports_sample_size(16));
        assert!(partial.lpcm_supports_sample_size(20));
        assert!(!partial.lpcm_supports_sample_size(24));
        assert_eq!(lpcm.channels, 2);

        // Compressed format: AC-3 has no LPCM sample size.
        let ac3 = CtaAudioDescriptor {
            format: 2,
            channels: 6,
            sample_rates: 0b000_0111,
            format_specific: 0x40,
        };
        assert_eq!(ac3.format_kind(), CtaAudioFormat::Ac3);
        assert_eq!(ac3.lpcm_sample_size_bits(), &[]);
        assert!(!ac3.lpcm_supports_sample_size(16));

        // Extended format (format code 15) maps to Extended.
        let ext = CtaAudioDescriptor {
            format: 15,
            channels: 8,
            sample_rates: 0,
            format_specific: 0xD8,
        };
        assert_eq!(ext.format_kind(), CtaAudioFormat::Extended);

        // Unknown format code maps to Reserved.
        let unknown = CtaAudioDescriptor {
            format: 0,
            channels: 2,
            sample_rates: 0,
            format_specific: 0,
        };
        assert_eq!(unknown.format_kind(), CtaAudioFormat::Reserved);

        // Round-trip a mixed audio list (LPCM + compressed + unknown) preserves bytes.
        let block = CtaDataBlock {
            tag: 1,
            payload: vec![
                (1 << 3) | (2 - 1),
                0b000_0111,
                0b111, // LPCM
                (2 << 3) | (6 - 1),
                0b000_0111,
                0x40, // AC-3
                (7 << 3) | (5 - 1),
                0b000_0111,
                0x2A, // DTS
            ],
        };
        let view = block.view().unwrap();
        let CtaDataBlockView::Audio { descriptors } = &view else {
            panic!("expected Audio view");
        };
        assert_eq!(descriptors.len(), 3);
        assert_eq!(descriptors[0].format_kind(), CtaAudioFormat::Lpcm);
        assert_eq!(descriptors[1].format_kind(), CtaAudioFormat::Ac3);
        assert_eq!(descriptors[2].format_kind(), CtaAudioFormat::Dts);
        assert_eq!(view.to_data_block().unwrap(), block);

        // A reserved format code is rejected by the checked writer.
        let reserved = CtaDataBlockView::Audio {
            descriptors: vec![CtaAudioDescriptor {
                format: 0,
                channels: 2,
                sample_rates: 0,
                format_specific: 0,
            }],
        };
        assert!(matches!(
            reserved.to_data_block(),
            Err(ExtensionWriteError::InvalidCtaAudioFormat {
                index: 0,
                format: 0
            })
        ));

        // Reject channel count outside 1..=8.
        let bad_channels = CtaDataBlockView::Audio {
            descriptors: vec![CtaAudioDescriptor {
                format: 1,
                channels: 9,
                sample_rates: 0,
                format_specific: 0,
            }],
        };
        assert!(matches!(
            bad_channels.to_data_block(),
            Err(ExtensionWriteError::InvalidCtaAudioChannels {
                index: 0,
                channels: 9
            })
        ));

        // Reject format code outside 1..=15.
        let bad_format = CtaDataBlockView::Audio {
            descriptors: vec![CtaAudioDescriptor {
                format: 16,
                channels: 2,
                sample_rates: 0,
                format_specific: 0,
            }],
        };
        assert!(matches!(
            bad_format.to_data_block(),
            Err(ExtensionWriteError::InvalidCtaAudioFormat {
                index: 0,
                format: 16
            })
        ));
    }

    #[test]
    fn cta_y420_video_and_capability_map_roundtrip_and_query() {
        let vdb = CtaDataBlock {
            tag: 2,
            payload: vec![16, 97, 107], // SVD 0: VIC 16, SVD 1: VIC 97, SVD 2: VIC 107
        };
        let y420_vdb = CtaDataBlock {
            tag: 7,
            payload: vec![0x0E, 96], // VIC 96 (4:2:0 only)
        };
        let y420_cmdb = CtaDataBlock {
            tag: 7,
            payload: vec![0x0F, 0b0000_0010], // SVD 1 (VIC 97) is 4:2:0 capable
        };

        // Verify views
        let y420_vdb_view = y420_vdb.view().unwrap();
        assert_eq!(
            y420_vdb_view,
            CtaDataBlockView::Extended(CtaExtendedDataBlockView::Y420Video {
                modes: vec![CtaVideoMode {
                    vic: 96,
                    native: false
                }]
            })
        );
        assert_eq!(y420_vdb_view.to_data_block().unwrap(), y420_vdb);

        let y420_cmdb_view = y420_cmdb.view().unwrap();
        assert_eq!(
            y420_cmdb_view,
            CtaDataBlockView::Extended(CtaExtendedDataBlockView::Y420CapabilityMap {
                raw: vec![0x0F, 0b0000_0010],
            })
        );
        assert_eq!(y420_cmdb_view.to_data_block().unwrap(), y420_cmdb);

        // Test resolution
        let blocks = vec![vdb.clone(), y420_vdb.clone(), y420_cmdb.clone()];
        let support = CtaY420Support::resolve_from_blocks(&blocks).unwrap();
        assert!(!support.supports_vic(16));
        assert!(support.supports_vic(97));
        assert!(!support.is_420_only(97));
        assert!(support.supports_vic(96));
        assert!(support.is_420_only(96));
        assert_eq!(support.all_420_vics(), vec![97, 96]);

        // Test out of range error
        let invalid_cmdb = CtaDataBlock {
            tag: 7,
            payload: vec![0x0F, 0b0001_0000], // SVD 4 (out of range, only 3 exist)
        };
        let err_blocks = vec![vdb.clone(), invalid_cmdb];
        assert!(matches!(
            CtaY420Support::resolve_from_blocks(&err_blocks),
            Err(ExtensionError::Y420CapabilityMapIndexOutOfRange {
                index: 4,
                available_svds: 3
            })
        ));

        // Test missing VDB error
        let no_vdb_blocks = vec![y420_cmdb];
        assert!(matches!(
            CtaY420Support::resolve_from_blocks(&no_vdb_blocks),
            Err(ExtensionError::Y420CapabilityMapMissingVideoDataBlock)
        ));
    }

    #[test]
    fn display_id_dynamic_video_timing_range_roundtrip_query_and_rejection() {
        // 1. Decode valid Tag 0x25 block (451,310 kHz, 48-165 Hz, seamless = true)
        // 451,310 kHz -> minus 1 is 451,309 = 0x06E2ED -> bytes [0xED, 0xE2, 0x06]
        // 48 Hz min -> 0x30
        // 165 Hz max -> 165 = 0x00A5 -> lower 0xA5, upper 0x00
        // seamless flag -> bit 7 = 0x80
        let payload = vec![
            0xED, 0xE2, 0x06, // min pixel clock (451,310 kHz)
            0xED, 0xE2, 0x06, // max pixel clock (451,310 kHz)
            48,   // min vfreq
            165,  // max vfreq lower
            0x80, // seamless flag
        ];
        let block = DisplayIdDataBlock {
            tag: 0x25,
            revision: 0,
            payload: payload.clone(),
        };
        let view = block.view().unwrap();
        let DisplayIdDataBlockView::DynamicVideoTimingRange { range } = &view else {
            panic!("expected DynamicVideoTimingRange view");
        };
        assert_eq!(range.min_pixel_clock_khz, 451_310);
        assert_eq!(range.max_pixel_clock_khz, 451_310);
        assert_eq!(range.min_vfreq_hz, 48);
        assert_eq!(range.max_vfreq_hz, 165);
        assert!(range.seamless_dynamic_video_timing);
        assert_eq!(range.raw, payload);

        // Lossless round-trip
        let encoded = view.to_data_block().unwrap();
        assert_eq!(encoded, block);

        // Query from EdidBlock
        let edid_block =
            EdidBlock::from_display_id_data_blocks(0x20, 2, 0, std::slice::from_ref(&block))
                .unwrap();
        let ranges = edid_block.display_id_dynamic_video_timing_ranges().unwrap();
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].min_vfreq_hz, 48);
        assert_eq!(ranges[0].max_vfreq_hz, 165);
        // Mutation test
        let mut modified_range: DisplayIdDynamicVideoTimingRange = range.clone();
        modified_range.max_vfreq_hz = 240;
        modified_range.seamless_dynamic_video_timing = false;
        let modified_view = DisplayIdDataBlockView::DynamicVideoTimingRange {
            range: modified_range,
        };
        let modified_block = modified_view.to_data_block().unwrap();
        assert_eq!(modified_block.payload[7], 240);
        assert_eq!(modified_block.payload[8], 0x00);

        // Rejection tests: min clock > max clock
        let invalid_clock = DisplayIdDataBlock {
            tag: 0x25,
            revision: 0,
            payload: vec![0xFF, 0xE2, 0x06, 0x00, 0xE2, 0x06, 48, 165, 0x80],
        };
        assert!(matches!(
            invalid_clock.view(),
            Err(ExtensionError::InvalidDisplayIdDynamicRange { .. })
        ));

        // Rejection tests: min vfreq > max vfreq
        let invalid_vfreq = DisplayIdDataBlock {
            tag: 0x25,
            revision: 0,
            payload: vec![0xED, 0xE2, 0x06, 0xED, 0xE2, 0x06, 165, 48, 0x80],
        };
        assert!(matches!(
            invalid_vfreq.view(),
            Err(ExtensionError::InvalidDisplayIdDynamicRange { .. })
        ));
    }

    #[test]
    fn display_id_video_timing_range_limits_1x_roundtrip() {
        // Tag 0x09 (1.x) fixed 15-byte payload:
        // 148,500 kHz min/max (10 kHz units -> 14849 => 0x3A01), hfreq 30..100 kHz,
        // min h-blank 160, vfreq 48..165 Hz, min v-blank 3, flags 0xF0.
        let payload = vec![
            0x01, 0x3A, 0x00, // min pixel clock (10 kHz units)
            0x01, 0x3A, 0x00, // max pixel clock (10 kHz units)
            30,   // min hfreq
            100,  // max hfreq
            0xA0, 0x00, // min h blanking
            48,   // min vfreq
            165,  // max vfreq
            0x03, 0x00, // min v blanking
            0xF0, // flags: interlaced | CVT | CVT-RB | discrete
        ];
        let block = DisplayIdDataBlock {
            tag: 0x09,
            revision: 0,
            payload: payload.clone(),
        };
        let view = block.view().unwrap();
        let DisplayIdDataBlockView::VideoTimingRange1x { range } = &view else {
            panic!("expected VideoTimingRange1x view");
        };
        assert_eq!(range.min_pixel_clock_khz, 148_500);
        assert_eq!(range.max_pixel_clock_khz, 148_500);
        assert_eq!(range.min_hfreq_khz, 30);
        assert_eq!(range.max_hfreq_khz, 100);
        assert_eq!(range.min_h_blanking, 160);
        assert_eq!(range.min_vfreq_hz, 48);
        assert_eq!(range.max_vfreq_hz, 165);
        assert_eq!(range.min_v_blanking, 3);
        assert!(range.supports_interlaced);
        assert!(range.supports_cvt);
        assert!(range.supports_cvt_reduced_blanking);
        assert!(range.discrete_frequency);
        assert_eq!(range.raw, payload);

        // Lossless round-trip.
        let encoded = view.to_data_block().unwrap();
        assert_eq!(encoded.tag, 0x09);
        assert_eq!(encoded, block);

        // Reject non-multiple-of-10 kHz clock on encode.
        let mut invalid = range.clone();
        invalid.min_pixel_clock_khz = 148_505;
        let bad_view = DisplayIdDataBlockView::VideoTimingRange1x { range: invalid };
        assert!(matches!(
            bad_view.to_data_block_with_tag(0x09),
            Err(ExtensionWriteError::InvalidDisplayIdDynamicRange { .. })
        ));

        // Reject a payload that is not exactly 15 bytes.
        let short = DisplayIdDataBlock {
            tag: 0x09,
            revision: 0,
            payload: vec![0; 14],
        };
        assert!(matches!(
            short.view(),
            Err(ExtensionError::InvalidDisplayIdDataBlockLength { tag: 0x09, .. })
        ));
    }

    #[test]
    fn display_id_type_iv_enumerated_timing_roundtrip() {
        // Tag 0x06 (1.x Type IV) shares the DMT/VIC/HDMI-VIC structure of 0x23.
        // revision 0x40 = code_type 1 (CTA VIC), 1-byte codes.
        let block = DisplayIdDataBlock {
            tag: 0x06,
            revision: 0x40,
            payload: vec![16], // VIC 16 (1920x1080p60)
        };
        let view = block.view().unwrap();
        assert_eq!(
            view,
            DisplayIdDataBlockView::EnumeratedTiming {
                code_type: 1,
                code_size: 1,
                codes: vec![16],
            }
        );
        let encoded = view.to_data_block_with_tag(0x06).unwrap();
        assert_eq!(encoded.tag, 0x06);
        assert_eq!(encoded.revision, 0x40);
        assert_eq!(encoded.payload, vec![16]);
    }

    #[test]
    fn display_id_tiled_topology_roundtrip_geometry_and_rejection() {
        // Tag 0x28, 22-byte payload. 2x2 grid, tile at (1,0) (0-based), 1920x1080 tile,
        // vendor OUI, product 0x1234, serial 0xDEADBEEF.
        let payload = vec![
            0x00, // caps (multiple enclosures, no bevel)
            0x11, // num_h stored low nibble 1(->2), num_v stored low nibble 1(->2)
            0x10, // tile_h location high nibble 1, tile_v location low nibble 0
            0x00, // high bits all zero
            0x7F, 0x07, // width stored 1919 (0x077F) => 1920
            0x37, 0x04, // height stored 1079 => 1080
            0x00, // pixel multiplier
            0, 0, 0, 0, // bevels (not present, caps&0x40=0)
            0x03, 0x0C, 0x00, // vendor OUI
            0x34, 0x12, // product 0x1234
            0xEF, 0xBE, 0xAD, 0xDE, // serial 0xDEADBEEF
        ];
        assert_eq!(payload.len(), 22);
        let block = DisplayIdDataBlock {
            tag: 0x28,
            revision: 0,
            payload: payload.clone(),
        };
        let view = block.view().unwrap();
        let DisplayIdDataBlockView::TiledDisplayTopology { topology } = &view else {
            panic!("expected TiledDisplayTopology view");
        };
        let topology: &DisplayIdTiledDisplayTopology = topology;
        assert_eq!(topology.tiles_h, 2);
        assert_eq!(topology.tiles_v, 2);
        assert_eq!(topology.tile_location_h, 1);
        assert_eq!(topology.tile_location_v, 0);
        assert_eq!(topology.tile_width, 1920);
        assert_eq!(topology.tile_height, 1080);
        assert!(topology.vendor_id_is_oui);
        assert_eq!(topology.vendor_id, [0x03, 0x0C, 0x00]);
        assert_eq!(topology.product_code, 0x1234);
        assert_eq!(topology.serial_number, 0xDEADBEEF);
        assert!(!topology.has_bevel_info());
        assert!(!topology.single_enclosure());

        // Lossless round-trip.
        assert_eq!(view.to_data_block().unwrap(), block);

        // Query from EdidBlock.
        let edid_block =
            EdidBlock::from_display_id_data_blocks(0x20, 2, 0, std::slice::from_ref(&block))
                .unwrap();
        let topologies = edid_block.display_id_tiled_topologies().unwrap();
        assert_eq!(topologies.len(), 1);
        assert_eq!(topologies[0].tile_location_h, 1);

        // Rejection: tile location >= tile count.
        let bad_location = DisplayIdDataBlock {
            tag: 0x28,
            revision: 0,
            payload: {
                let mut p2 = payload.clone();
                p2[2] = 0x20; // tile_h location 2 (but 2 tiles)
                p2
            },
        };
        assert!(matches!(
            bad_location.view(),
            Err(ExtensionError::InvalidDisplayIdDynamicRange { .. })
        ));

        // Rejection: bevel multiplier set without bevel info.
        let bad_bevel = DisplayIdDataBlock {
            tag: 0x28,
            revision: 0,
            payload: {
                let mut p2 = payload.clone();
                p2[8] = 1; // pixel multiplier set, but caps&0x40=0
                p2
            },
        };
        assert!(matches!(
            bad_bevel.view(),
            Err(ExtensionError::InvalidDisplayIdDynamicRange { .. })
        ));
    }

    #[test]
    fn display_id_formula_and_enumerated_timing_roundtrip() {
        // Type IX formula timing (tag 0x24): 2 descriptors.
        let formula_payload = vec![
            0x02 | 0x10,
            0x7F,
            0x07,
            0x37,
            0x04,
            59, // CVT-RB, NTSC, 1920x1080 @60
            0x01 << 5,
            0xFF,
            0x0F,
            0x82,
            0x08,
            119, // CVT, 3D stereo, 3840x2160 @120
        ];
        let block = DisplayIdDataBlock {
            tag: 0x24,
            revision: 0,
            payload: formula_payload.clone(),
        };
        let view = block.view().unwrap();
        let DisplayIdDataBlockView::FormulaTiming { timings } = &view else {
            panic!("expected FormulaTiming view");
        };
        assert_eq!(timings.len(), 2);
        assert_eq!(timings[0].h_active, 1920);
        assert_eq!(timings[0].v_active, 1080);
        assert_eq!(timings[0].v_refresh_hz, 60);
        assert_eq!(timings[0].formula, 2);
        assert!(timings[0].ntsc_refresh);
        assert_eq!(timings[0].stereo_3d, 0);
        assert_eq!(timings[1].stereo_3d, 1);
        assert_eq!(timings[1].v_refresh_hz, 120);
        assert_eq!(view.to_data_block().unwrap(), block);

        // Type VIII enumerated timing (tag 0x23): 1-byte codes, CTA VIC (type 1).
        let enum_payload = vec![16, 97, 107];
        let enum_block = DisplayIdDataBlock {
            tag: 0x23,
            revision: 1 << 6,
            payload: enum_payload.clone(),
        };
        let enum_view = enum_block.view().unwrap();
        let DisplayIdDataBlockView::EnumeratedTiming {
            code_type,
            code_size,
            codes,
        } = &enum_view
        else {
            panic!("expected EnumeratedTiming view");
        };
        assert_eq!(*code_type, 1);
        assert_eq!(*code_size, 1);
        assert_eq!(*codes, vec![16, 97, 107]);
        assert_eq!(enum_view.to_data_block().unwrap(), enum_block);

        // Query from EdidBlock.
        let edid_block =
            EdidBlock::from_display_id_data_blocks(0x20, 2, 0, std::slice::from_ref(&block))
                .unwrap();
        let formulas = edid_block.display_id_formula_timings().unwrap();
        assert_eq!(formulas.len(), 2);
        assert_eq!(formulas[0].h_active, 1920);
        let enum_edid =
            EdidBlock::from_display_id_data_blocks(0x20, 2, 0, std::slice::from_ref(&enum_block))
                .unwrap();
        let enum_timings = enum_edid.display_id_enumerated_timings().unwrap();
        assert_eq!(enum_timings.len(), 3);
        assert_eq!(enum_timings[0].code_type, 1);
        assert_eq!(enum_timings[0].code, 16);

        // Rejection: formula timing with zero active region.
        let bad_formula = DisplayIdDataBlockView::FormulaTiming {
            timings: vec![DisplayIdFormulaTiming {
                h_active: 0,
                v_active: 1080,
                v_refresh_hz: 60,
                formula: 0,
                ntsc_refresh: false,
                stereo_3d: 0,
            }],
        };
        assert!(matches!(
            bad_formula.to_data_block_with_tag(0x24),
            Err(ExtensionWriteError::InvalidDisplayIdTimingField { .. })
        ));
    }

    #[test]
    fn cta_dynamic_hdr_metadata_roundtrip_and_rejection() {
        // Build an HDR Dynamic Metadata block: a versioned type 1, an SL-HDR type 2,
        // and an unknown type. Each entry is [type_len][type_lo][type_hi][data...].
        let payload = vec![
            0x07, // entry 1: type 1, type_len 3 => [3, 1,0, 0x13]
            3, 0x01, 0x00, 0x13, // entry 2: type 2, type_len 4 => [4, 2,0, 0x52, 0x00]
            4, 0x02, 0x00, 0x52, 0x00,
            // entry 3: unknown type 0xCDAB, type_len 4 => [4, 0xAB,0xCD, 0xDE,0xAD]
            4, 0xAB, 0xCD, 0xDE, 0xAD,
        ];
        let block = CtaDataBlock {
            tag: 7,
            payload: payload.clone(),
        };
        let view = block.view().unwrap();
        let CtaDataBlockView::Extended(CtaExtendedDataBlockView::HdrDynamicMetadata {
            entries,
            raw,
        }) = &view
        else {
            panic!("expected HdrDynamicMetadata view");
        };
        assert_eq!(raw, &payload);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].metadata_type, 1);
        assert_eq!(entries[0].version(), Some(3));
        assert!(!entries[0].sl_hdr1());
        assert_eq!(entries[1].metadata_type, 2);
        assert_eq!(entries[1].version(), Some(2));
        assert!(entries[1].sl_hdr1());
        assert!(!entries[1].sl_hdr2());
        assert!(entries[1].sl_hdr3());
        assert_eq!(entries[2].metadata_type, 0xCDAB);
        assert_eq!(entries[2].version(), None);
        assert_eq!(entries[2].data, vec![0xDE, 0xAD]);

        // Lossless round-trip.
        assert_eq!(view.to_data_block().unwrap(), block);

        // Rejection: truncated entry (declared length exceeds payload).
        let truncated = CtaDataBlock {
            tag: 7,
            payload: vec![0x07, 10, 0x01, 0x00, 0x11],
        };
        assert!(matches!(
            truncated.view(),
            Err(ExtensionError::TruncatedDynamicHdrMetadataEntry { index: 0, .. })
        ));

        // Rejection: type_len < 2.
        let bad_len = CtaDataBlock {
            tag: 7,
            payload: vec![0x07, 1, 0x01, 0x00],
        };
        assert!(matches!(
            bad_len.view(),
            Err(ExtensionError::InvalidDynamicHdrMetadataLength {
                index: 0,
                length: 1
            })
        ));
    }

    #[test]
    fn display_id_product_identification_roundtrip_fields_and_rejection() {
        // 2.x layout (tag 0x20): IEEE OUI vendor.
        let payload_2x = vec![
            0x03, 0x0C, 0x00, // OUI (little-endian 0x000C03)
            0xAA, 0xBB, // product code 0xBBAA
            0x78, 0x56, 0x34, 0x12, // serial 0x12345678
            42,   // week
            24,   // year stored = 2024 - 2000
            5,    // name len
            b'O', b'U', b'I', b'4', b'2',
        ];
        let block_2x = DisplayIdDataBlock {
            tag: 0x20,
            revision: 0,
            payload: payload_2x.clone(),
        };
        let view_2x = block_2x.view().unwrap();
        let DisplayIdDataBlockView::ProductIdentification { product } = &view_2x else {
            panic!("expected ProductIdentification view");
        };
        assert!(product.vendor_id_is_oui);
        assert_eq!(product.vendor_id, [0x03, 0x0C, 0x00]);
        assert_eq!(product.product_code, 0xBBAA);
        assert_eq!(product.serial_number, 0x12345678);
        assert_eq!(product.week_of_manufacture, 42);
        assert_eq!(product.year, 2024);
        assert_eq!(product.product_name, b"OUI42");
        assert!(!product.is_model_year());
        // Lossless round-trip
        assert_eq!(view_2x.to_data_block_with_tag(0x20).unwrap(), block_2x);

        // 1.x layout (tag 0x00): character vendor ID, no serial, empty name.
        let payload_1x = vec![
            b'A', b'U', b'P', // vendor chars
            0x05, 0x00, // product code 5
            0, 0, 0, 0,  // no serial
            0,  // no week
            20, // year stored = 2020
            0,  // empty name
        ];
        let block_1x = DisplayIdDataBlock {
            tag: 0x00,
            revision: 0,
            payload: payload_1x.clone(),
        };
        let view_1x = block_1x.view().unwrap();
        let DisplayIdDataBlockView::ProductIdentification { product } = &view_1x else {
            panic!("expected ProductIdentification view");
        };
        assert!(!product.vendor_id_is_oui);
        assert_eq!(product.vendor_id, [b'A', b'U', b'P']);
        assert_eq!(product.product_code, 5);
        assert_eq!(product.serial_number, 0);
        assert_eq!(product.year, 2020);
        assert!(product.product_name.is_empty());
        assert_eq!(view_1x.to_data_block_with_tag(0x00).unwrap(), block_1x);

        // Non-UTF-8 product name is preserved exactly.
        let non_utf8 = DisplayIdDataBlockView::ProductIdentification {
            product: DisplayIdProductIdentification {
                vendor_id: [0x01, 0x02, 0x03],
                vendor_id_is_oui: true,
                product_code: 7,
                serial_number: 0,
                week_of_manufacture: 0xFF,
                year: 2020,
                product_name: vec![0xFF, 0xFE, 0x00, 0x41],
                raw: vec![],
            },
        };
        let encoded = non_utf8.to_data_block().unwrap();
        let decoded = encoded.view().unwrap();
        let DisplayIdDataBlockView::ProductIdentification { product } = decoded else {
            panic!("expected ProductIdentification view");
        };
        assert_eq!(product.product_name, vec![0xFF, 0xFE, 0x00, 0x41]);
        assert!(product.product_name_str().is_none());
        assert!(product.is_model_year());

        // Query from EdidBlock.
        let edid_block =
            EdidBlock::from_display_id_data_blocks(0x20, 2, 0, std::slice::from_ref(&block_2x))
                .unwrap();
        let products = edid_block.display_id_product_identifications().unwrap();
        assert_eq!(products.len(), 1);
        assert_eq!(products[0].product_code, 0xBBAA);

        // Rejection: year outside 2000..=2255.
        let invalid_year = DisplayIdDataBlockView::ProductIdentification {
            product: DisplayIdProductIdentification {
                vendor_id: [0; 3],
                vendor_id_is_oui: false,
                product_code: 0,
                serial_number: 0,
                week_of_manufacture: 0,
                year: 1999,
                product_name: vec![],
                raw: vec![],
            },
        };
        assert!(matches!(
            invalid_year.to_data_block_with_tag(0x00),
            Err(ExtensionWriteError::InvalidDisplayIdProductField {
                field: "year",
                value: 1999,
                maximum: 2255
            })
        ));
    }

    #[test]
    fn display_id_interface_features_roundtrip_query_and_rejection() {
        // Tag 0x26 payload (9 bytes):
        // rgb: 8bpc + 10bpc (0b0011 -> bits 1,2)
        // ycbcr444: 8bpc (0b0001 -> bit 0)
        // ycbcr422: 10bpc (0b0010 -> bit 1)
        // ycbcr420: 12bpc (0b0100 -> bit 2)
        // min_ycbcr420_pixel_rate: 4 (74.25*4 = 297 MP/s)
        // audio_flags: 0xE0 (32/44.1/48 kHz)
        // colorspace_eotf_1: BT.2020 + ST2084 (bit 6) | BT.709 (bit 2) = 0x44
        // colorspace_eotf_2: reserved 0
        // additional_colorspace_count: 2
        let payload = vec![
            0b0000_0110,
            0b0000_0001,
            0b0000_0010,
            0b0000_0100,
            4,
            0xE0,
            0x44,
            0,
            2,
        ];
        let block = DisplayIdDataBlock {
            tag: 0x26,
            revision: 0,
            payload: payload.clone(),
        };
        let view = block.view().unwrap();
        let DisplayIdDataBlockView::InterfaceFeatures { features } = &view else {
            panic!("expected InterfaceFeatures view");
        };
        assert!(features.supports_rgb_bpc(8));
        assert!(features.supports_rgb_bpc(10));
        assert!(!features.supports_rgb_bpc(12));
        assert!(features.supports_ycbcr444_bpc(8));
        assert!(features.supports_ycbcr422_bpc(10));
        assert!(features.supports_ycbcr420_bpc(12));
        assert!(features.supports_bt2020_st2084());
        assert!(features.supports_bt709());
        assert_eq!(features.min_ycbcr420_pixel_rate, 4);
        assert_eq!(features.additional_colorspace_count, 2);
        assert_eq!(features.raw, payload);

        // Lossless round-trip
        let encoded = view.to_data_block().unwrap();
        assert_eq!(encoded, block);

        // Query from EdidBlock
        let edid_block =
            EdidBlock::from_display_id_data_blocks(0x20, 2, 0, std::slice::from_ref(&block))
                .unwrap();
        let features_list = edid_block.display_id_interface_features().unwrap();
        assert_eq!(features_list.len(), 1);
        assert!(features_list[0].supports_bt2020_st2084());

        // Mutation
        let mut features: DisplayIdInterfaceFeatures = features.clone();
        features.color_depth_rgb = 0b0111_1111; // all BPC
        let modified_view = DisplayIdDataBlockView::InterfaceFeatures { features };
        let modified_block = modified_view.to_data_block().unwrap();
        assert_eq!(modified_block.payload[0], 0b0111_1111);

        // Rejection: additional_colorspace_count > 7
        let invalid = DisplayIdDataBlock {
            tag: 0x26,
            revision: 0,
            payload: vec![0, 0, 0, 0, 0, 0, 0, 0, 8],
        };
        assert!(matches!(
            invalid.view(),
            Err(ExtensionError::InvalidDisplayIdFeatureField {
                field: "additional_colorspace_count",
                value: 8,
                maximum: 7
            })
        ));
    }

    #[test]
    fn typed_cta_views_reject_invalid_payload_shapes() {
        let invalid_audio = CtaDataBlock {
            tag: 1,
            payload: vec![0x09, 0x07],
        };
        assert!(matches!(
            invalid_audio.view(),
            Err(ExtensionError::InvalidAudioDataBlockLength { length: 2 })
        ));

        let invalid_video = CtaDataBlock {
            tag: 2,
            payload: vec![0x00],
        };
        assert!(matches!(
            invalid_video.view(),
            Err(ExtensionError::InvalidVideoCode { index: 0 })
        ));

        let invalid_hdr = CtaDataBlock {
            tag: 7,
            payload: vec![0x06, 0x07],
        };
        assert!(matches!(
            invalid_hdr.view(),
            Err(ExtensionError::TruncatedExtendedDataBlock {
                extended_tag: 0x06,
                length: 2,
                minimum: 3,
            })
        ));

        let empty_audio = CtaDataBlock {
            tag: 1,
            payload: Vec::new(),
        };
        assert!(matches!(
            empty_audio.view(),
            Err(ExtensionError::InvalidAudioDataBlockLength { length: 0 })
        ));

        let empty_video = CtaDataBlock {
            tag: 2,
            payload: Vec::new(),
        };
        assert!(matches!(
            empty_video.view(),
            Err(ExtensionError::InvalidVideoDataBlockLength { length: 0 })
        ));
    }

    #[test]
    fn rejects_empty_extended_data_block_payload() {
        let block = CtaDataBlock {
            tag: 7,
            payload: Vec::new(),
        };
        assert!(matches!(
            block.view(),
            Err(ExtensionError::TruncatedExtendedDataBlock {
                extended_tag: 0,
                length: 0,
                minimum: 1,
            })
        ));
    }

    #[test]
    fn reads_displayid_header_and_typed_detailed_timing() {
        let mut block = EdidBlock::new_default();
        block.raw[0] = 0x70;
        block.raw[1] = 0x20;
        block.raw[2] = 23;
        block.raw[3] = 2;
        block.raw[4] = 0;
        block.raw[5..8].copy_from_slice(&[0x22, 1, 20]);
        let timing = &mut block.raw[8..28];
        timing[0..3].copy_from_slice(&59_999u32.to_le_bytes()[..3]);
        timing[3] = 0x80;
        timing[4..6].copy_from_slice(&1919u16.to_le_bytes());
        timing[6..8].copy_from_slice(&279u16.to_le_bytes());
        timing[8..10].copy_from_slice(&(87u16 | 0x8000).to_le_bytes());
        timing[10..12].copy_from_slice(&43u16.to_le_bytes());
        timing[12..14].copy_from_slice(&1079u16.to_le_bytes());
        timing[14..16].copy_from_slice(&44u16.to_le_bytes());
        timing[16..18].copy_from_slice(&(4u16 | 0x8000).to_le_bytes());
        timing[18..20].copy_from_slice(&5u16.to_le_bytes());
        block.raw[28] = block.raw[1..28]
            .iter()
            .fold(0u8, |sum, &byte| sum.wrapping_sub(byte));
        block.update_checksum();

        assert_eq!(
            block.display_id_header().unwrap(),
            DisplayIdHeader {
                revision: 0x20,
                payload_length: 23,
                product_type_or_primary_use: 2,
                extension_count: 0,
            }
        );
        let blocks = block.display_id_data_blocks().unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0].view().unwrap(),
            DisplayIdDataBlockView::DetailedTiming {
                timings: vec![DisplayIdDetailedTiming {
                    pixel_clock_khz: 60_000,
                    h_active: 1920,
                    h_blank: 280,
                    h_sync_offset: 88,
                    h_sync_width: 44,
                    v_active: 1080,
                    v_blank: 45,
                    v_sync_offset: 5,
                    v_sync_width: 6,
                    h_sync_positive: true,
                    v_sync_positive: true,
                    aspect_ratio: DisplayIdAspectRatio::OneToOne,
                    interlaced: false,
                    stereo_3d: DisplayIdStereo3d::Mono,
                    preferred: true,
                    ycbcr420: false,
                }]
            }
        );
    }

    #[test]
    fn reads_displayid_parameters_and_preserves_unknown_blocks() {
        let mut block = EdidBlock::new_default();
        block.raw[0] = 0x70;
        block.raw[1] = 1;
        block.raw[2] = 3 + 29 + 3 + 2;
        block.raw[3] = 3;
        block.raw[5..8].copy_from_slice(&[0x21, 2, 29]);
        block.raw[8..10].copy_from_slice(&600u16.to_le_bytes());
        block.raw[10..12].copy_from_slice(&340u16.to_le_bytes());
        block.raw[12..14].copy_from_slice(&1920u16.to_le_bytes());
        block.raw[14..16].copy_from_slice(&1080u16.to_le_bytes());
        block.raw[16] = 0x80;
        block.raw[17..29].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        block.raw[29..31].copy_from_slice(&1000u16.to_le_bytes());
        block.raw[31..33].copy_from_slice(&800u16.to_le_bytes());
        block.raw[33..35].copy_from_slice(&2u16.to_le_bytes());
        block.raw[35] = 0x15;
        block.raw[36] = 0x20;
        block.raw[37..40].copy_from_slice(&[0x55, 7, 2]);
        block.raw[40..42].copy_from_slice(&[0xAA, 0xBB]);
        block.raw[42] = block.raw[1..42]
            .iter()
            .fold(0u8, |sum, &byte| sum.wrapping_sub(byte));
        block.update_checksum();

        assert_eq!(
            block.display_id_data_blocks().unwrap()[0].view().unwrap(),
            DisplayIdDataBlockView::DisplayParameters {
                parameters: DisplayIdDisplayParameters {
                    horizontal_image_size_mm: 600,
                    vertical_image_size_mm: 340,
                    horizontal_pixel_count: 1920,
                    vertical_pixel_count: 1080,
                    features: 0x80,
                    primary_color_1: [1, 2, 3],
                    primary_color_2: [4, 5, 6],
                    primary_color_3: [7, 8, 9],
                    white_point: [10, 11, 12],
                    max_luminance_full: 1000,
                    max_luminance_10_percent: 800,
                    min_luminance: 2,
                    color_depth_and_technology: 0x15,
                    gamma_eotf: 0x20,
                    raw: block.raw[8..37].to_vec(),
                }
            }
        );
        assert_eq!(
            block.display_id_data_blocks().unwrap()[1].view().unwrap(),
            DisplayIdDataBlockView::Unknown {
                tag: 0x55,
                payload: vec![0xAA, 0xBB],
            }
        );
    }

    #[test]
    fn decodes_displayid_type_one_timing_clock_units() {
        let block = super::DisplayIdDataBlock {
            tag: 0x03,
            revision: 1,
            payload: vec![0; 20],
        };
        assert_eq!(
            block.view().unwrap(),
            DisplayIdDataBlockView::DetailedTiming {
                timings: vec![DisplayIdDetailedTiming {
                    pixel_clock_khz: 10,
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
                    aspect_ratio: DisplayIdAspectRatio::OneToOne,
                    interlaced: false,
                    stereo_3d: DisplayIdStereo3d::Mono,
                    preferred: false,
                    ycbcr420: false,
                }]
            }
        );
    }

    #[test]
    fn type_1_7_timing_preserves_byte3_field_semantics() {
        // Tag 0x22, revision 1: byte 3 bit 7 means "preferred".
        // byte3 = 0xB6 = aspect 6 (64:27) | interlaced (0x10) | 3D stereo (0x20) | preferred (0x80).
        let block = DisplayIdDataBlock {
            tag: 0x22,
            revision: 1,
            payload: {
                let mut p = vec![0u8; 20];
                p[3] = 0xB6;
                p
            },
        };
        let view = block.view().unwrap();
        let DisplayIdDataBlockView::DetailedTiming { timings } = &view else {
            panic!("expected detailed timing view");
        };
        assert_eq!(timings.len(), 1);
        assert_eq!(
            timings[0].aspect_ratio,
            DisplayIdAspectRatio::SixtyFourToTwentySeven
        );
        assert!(timings[0].interlaced);
        assert_eq!(timings[0].stereo_3d, DisplayIdStereo3d::Stereo3d);
        assert!(timings[0].preferred);
        assert!(!timings[0].ycbcr420);

        // Re-encoding a revision<2 block reproduces byte 3 exactly.
        let encoded = view.to_data_block_with_tag(0x22).unwrap();
        assert_eq!(encoded.payload[3], 0xB6);

        // Same payload in a revision>=2 block: bit 7 means YCbCr 4:2:0, not preferred.
        let block_r2 = DisplayIdDataBlock {
            tag: 0x22,
            revision: 2,
            payload: {
                let mut p = vec![0u8; 20];
                p[3] = 0xB6;
                p
            },
        };
        let view_r2 = block_r2.view().unwrap();
        let DisplayIdDataBlockView::DetailedTiming { timings } = &view_r2 else {
            panic!("expected detailed timing view");
        };
        assert!(!timings[0].preferred);
        assert!(timings[0].ycbcr420);
        let encoded_r2 = view_r2.to_data_block_with_tag(0x22).unwrap();
        assert_eq!(encoded_r2.payload[3], 0xB6);
    }

    #[test]
    fn type_1_7_timing_preserves_reserved_aspect_and_stereo_values() {
        // Reserved aspect (0xA = 10) and reserved stereo (0x3) round-trip verbatim.
        let block = DisplayIdDataBlock {
            tag: 0x03,
            revision: 0,
            payload: {
                let mut p = vec![0u8; 20];
                // aspect=0xA, stereo=0x3 (bit 6:5 => 0x60)
                p[3] = 0x60 | 0x0A;
                p
            },
        };
        let view = block.view().unwrap();
        let DisplayIdDataBlockView::DetailedTiming { timings } = &view else {
            panic!("expected detailed timing view");
        };
        assert_eq!(timings[0].aspect_ratio, DisplayIdAspectRatio::Reserved(10));
        assert_eq!(timings[0].stereo_3d, DisplayIdStereo3d::Reserved);
        let encoded = view.to_data_block_with_tag(0x03).unwrap();
        assert_eq!(encoded.payload[3], 0x6A);
    }

    #[test]
    fn decodes_displayid_v1_display_parameters_tag() {
        let block = super::DisplayIdDataBlock {
            tag: 0x01,
            revision: 1,
            payload: vec![0; 29],
        };
        assert!(matches!(
            block.view().unwrap(),
            DisplayIdDataBlockView::DisplayParameters { .. }
        ));
    }

    #[test]
    fn rejects_incomplete_displayid_data_block_header() {
        let mut block = EdidBlock::new_default();
        block.raw[0] = 0x70;
        block.raw[1] = 0x20;
        block.raw[2] = 1;
        block.raw[5] = 0x55;
        block.raw[6] = block.raw[1..6]
            .iter()
            .fold(0u8, |sum, &byte| sum.wrapping_sub(byte));
        block.update_checksum();
        assert!(matches!(
            block.display_id_data_blocks(),
            Err(ExtensionError::TruncatedDisplayIdDataBlockHeader {
                offset: 5,
                available: 1
            })
        ));
    }

    #[test]
    fn rejects_invalid_displayid_lengths_and_checksum() {
        let mut block = EdidBlock::new_default();
        block.raw[0] = 0x70;
        block.raw[1] = 0x20;
        block.raw[2] = 4;
        block.raw[5..8].copy_from_slice(&[0x22, 1, 20]);
        block.raw[8] = 0xAA;
        block.raw[9] = block.raw[1..9]
            .iter()
            .fold(0u8, |sum, &byte| sum.wrapping_sub(byte));
        block.update_checksum();
        assert!(matches!(
            block.display_id_data_blocks(),
            Err(ExtensionError::TruncatedDisplayIdDataBlock {
                offset: 5,
                tag: 0x22,
                length: 20,
                available: 1
            })
        ));

        block.raw[2] = 0;
        block.raw[5] = 1;
        block.update_checksum();
        assert!(matches!(
            block.display_id_header(),
            Err(ExtensionError::InvalidDisplayIdChecksum { .. })
        ));
    }

    #[test]
    fn cta_vendor_specific_data_blocks_decode_correctly() {
        use super::CtaVendorSpecificBlock;

        // HDMI 1.4b VSDB
        let hdmi14b_payload = vec![0x03, 0x0C, 0x00, 0x10, 0x00, 0x38, 0x3C, 0x20];
        let block = CtaDataBlock {
            tag: 3,
            payload: hdmi14b_payload.clone(),
        };
        match block.view().unwrap() {
            CtaDataBlockView::VendorSpecific(CtaVendorSpecificBlock::Hdmi14b {
                physical_address,
                max_tmds_clock_mhz,
                deep_color_flags,
                feature_flags,
                raw,
            }) => {
                assert_eq!(physical_address, [0x10, 0x00]);
                assert_eq!(max_tmds_clock_mhz, Some(300)); // 0x3C * 5 = 60 * 5 = 300
                assert_eq!(deep_color_flags, 0x38);
                assert_eq!(feature_flags, 0x20);
                assert_eq!(raw, hdmi14b_payload);
            }
            other => panic!("expected Hdmi14b, got {other:?}"),
        }

        // HDMI Forum VSDB
        let hf_payload = vec![0xD8, 0x5D, 0xC4, 0x01, 0x78, 0xDC, 0x00, 0x01];
        let block = CtaDataBlock {
            tag: 3,
            payload: hf_payload.clone(),
        };
        match block.view().unwrap() {
            CtaDataBlockView::VendorSpecific(CtaVendorSpecificBlock::HdmiForum {
                version,
                max_tmds_character_rate_mhz,
                scdc_flags,
                deep_color_420_flags,
                raw,
                ..
            }) => {
                assert_eq!(version, 1);
                assert_eq!(max_tmds_character_rate_mhz, Some(600)); // 0x78 * 5 = 120 * 5 = 600
                assert_eq!(scdc_flags, 0xDC);
                assert_eq!(deep_color_420_flags, 0x01);
                assert_eq!(raw, hf_payload);
            }
            other => panic!("expected HdmiForum, got {other:?}"),
        }

        // Other VSDB (e.g. unknown vendor OUI [0xAA, 0xBB, 0xCC])
        let unknown_payload = vec![0xAA, 0xBB, 0xCC, 0x11, 0x22, 0x33];
        let block = CtaDataBlock {
            tag: 3,
            payload: unknown_payload.clone(),
        };
        match block.view().unwrap() {
            CtaDataBlockView::VendorSpecific(CtaVendorSpecificBlock::Other { oui, payload }) => {
                assert_eq!(oui, [0xAA, 0xBB, 0xCC]);
                assert_eq!(payload, vec![0x11, 0x22, 0x33]);
            }
            other => panic!("expected Other, got {other:?}"),
        }

        // Truncated VSDB (< 3 bytes)
        let truncated = CtaDataBlock {
            tag: 3,
            payload: vec![0x03, 0x0C],
        };
        assert!(matches!(
            truncated.view(),
            Err(ExtensionError::TruncatedVendorSpecificDataBlock { length: 2 })
        ));
    }

    #[test]
    fn hdmi_forum_modern_features_and_short_vsdb_roundtrip() {
        // 1. Short HDMI 2.0 HF-VSDB (8 bytes)
        let short_payload = vec![0xD8, 0x5D, 0xC4, 1, 120, 0xDC, 0x00, 0x01];
        let short_block = CtaDataBlock {
            tag: 3,
            payload: short_payload.clone(),
        };
        let short_view = short_block.view().unwrap();
        let CtaDataBlockView::VendorSpecific(CtaVendorSpecificBlock::HdmiForum {
            version,
            max_tmds_character_rate_mhz,
            scdc_flags,
            max_frl_rate,
            deep_color_420_flags,
            vrr_flags,
            vrr_min_hz,
            vrr_max_hz,
            dsc_flags,
            dsc_max_slices,
            dsc_total_chunk_kbytes,
            raw,
        }) = &short_view
        else {
            panic!("expected HdmiForum view");
        };
        assert_eq!(*version, 1);
        assert_eq!(*max_tmds_character_rate_mhz, Some(600));
        assert_eq!(*scdc_flags, 0xDC);
        assert_eq!(*max_frl_rate, Some(0));
        assert_eq!(*deep_color_420_flags, 0x01);
        assert_eq!(*vrr_flags, None);
        assert_eq!(*vrr_min_hz, None);
        assert_eq!(*vrr_max_hz, None);
        assert_eq!(*dsc_flags, None);
        assert_eq!(*dsc_max_slices, None);
        assert_eq!(*dsc_total_chunk_kbytes, None);
        assert_eq!(*raw, short_payload);

        // Lossless round-trip of short HF-VSDB
        let encoded_short = short_view.to_data_block().unwrap();
        assert_eq!(encoded_short.payload, short_payload);

        // 2. Full HDMI 2.1 HF-VSDB (14 bytes)
        // Byte 0..2: 0xD8, 0x5D, 0xC4
        // Byte 3: version 1
        // Byte 4: TMDS character rate 600 MHz (0x78 = 120)
        // Byte 5: SCDC 0xDC
        // Byte 6: FRL 6 (48Gbps: 0x60)
        // Byte 7: DC 4:2:0 0x07 (30, 36, 48 bit)
        // Byte 8: ALLM + FVA + Cinema VRR (0b0001_0110: ALLM bit 1, FVA bit 2, Cinema VRR bit 4)
        // Byte 9: VRR min 48 Hz (0x30) + VRR max upper 0 (0x30)
        // Byte 10: VRR max lower 144 (0x90)
        // Byte 11: DSC 1.2 flags (0x87)
        // Byte 12: DSC max slices (0x44: 4 slices, FRL 4)
        // Byte 13: DSC total chunk kbytes (0x10: 16 KB)
        let full_payload = vec![
            0xD8,
            0x5D,
            0xC4,
            1,
            120,
            0xDC,
            0x60,
            0x07,
            0b0001_0110,
            0x30,
            0x90,
            0x87,
            0x44,
            0x10,
        ];
        let full_block = CtaDataBlock {
            tag: 3,
            payload: full_payload.clone(),
        };
        let full_view = full_block.view().unwrap();
        let CtaDataBlockView::VendorSpecific(full_vsdb) = full_view.clone() else {
            panic!("expected HdmiForum view");
        };
        assert!(full_vsdb.is_allm_supported());
        assert!(full_vsdb.is_fva_supported());
        assert!(full_vsdb.is_cinema_vrr_supported());
        if let CtaVendorSpecificBlock::HdmiForum {
            max_frl_rate,
            vrr_min_hz,
            vrr_max_hz,
            dsc_flags,
            ..
        } = &full_vsdb
        {
            assert_eq!(*max_frl_rate, Some(6));
            assert_eq!(*vrr_min_hz, Some(48));
            assert_eq!(*vrr_max_hz, Some(144));
            assert_eq!(*dsc_flags, Some(0x87));
        }

        // Lossless round-trip of full HF-VSDB
        let encoded_full = full_view.to_data_block().unwrap();
        assert_eq!(encoded_full.payload, full_payload);

        // Modifying fields updates encoded bytes
        let mut modified = full_vsdb.clone();
        if let CtaVendorSpecificBlock::HdmiForum {
            max_frl_rate,
            vrr_max_hz,
            ..
        } = &mut modified
        {
            *max_frl_rate = Some(5);
            *vrr_max_hz = Some(240);
        }
        let modified_encoded = CtaDataBlockView::VendorSpecific(modified)
            .to_data_block()
            .unwrap();
        assert_eq!(modified_encoded.payload[6], 0x50);
        assert_eq!(modified_encoded.payload[9], 0x30); // upper 2 bits of 240 is 0
        assert_eq!(modified_encoded.payload[10], 240); // 0xF0

        // Rejection tests: FRL > 6
        let mut invalid_frl = full_vsdb.clone();
        if let CtaVendorSpecificBlock::HdmiForum { max_frl_rate, .. } = &mut invalid_frl {
            *max_frl_rate = Some(7);
        }
        assert!(matches!(
            CtaDataBlockView::VendorSpecific(invalid_frl).to_data_block(),
            Err(ExtensionWriteError::InvalidCtaField {
                field: "max_frl_rate",
                value: 7,
                maximum: 6
            })
        ));

        // Rejection tests: VRR min > max
        let mut invalid_vrr = full_vsdb.clone();
        if let CtaVendorSpecificBlock::HdmiForum {
            vrr_min_hz,
            vrr_max_hz,
            ..
        } = &mut invalid_vrr
        {
            *vrr_min_hz = Some(60);
            *vrr_max_hz = Some(48);
        }
        assert!(matches!(
            CtaDataBlockView::VendorSpecific(invalid_vrr).to_data_block(),
            Err(ExtensionWriteError::InvalidRefreshRateRange {
                min_refresh_hz: 60,
                max_refresh_hz: 48
            })
        ));
    }

    #[test]
    fn cta_header_and_dtds_decode_correctly() {
        let mut raw = [0u8; 128];
        raw[0] = 0x02; // CTA tag
        raw[1] = 0x03; // Revision 3
        raw[2] = 0x06; // DTD offset at byte 6
        raw[3] = 0xF2; // underscan=1, basic_audio=1, ycbcr_444=1, ycbcr_422=1, native_dtd_count=2

        // Data block collection at 4..6: e.g. 1-byte Video block (tag 2, length 1 -> header 0x41)
        raw[4] = 0x41;
        raw[5] = 0x10; // VIC 16 (1080p60)

        // DTD at byte 6..24: 1080p60 (1920x1080 @ 60Hz)
        // 148.5 MHz pixel clock -> 14850 = 0x3A02
        raw[6] = 0x02;
        raw[7] = 0x3A;
        raw[8] = 0x80; // HActive[7:0] = 1920 & 0xFF = 0x80
        raw[9] = 0x18; // HBlank[7:0] = 280 & 0xFF = 0x18
        raw[10] = 0x71; // HActive[11:8]=0x7, HBlank[11:8]=0x1
        raw[11] = 0x38; // VActive[7:0] = 1080 & 0xFF = 0x38
        raw[12] = 0x2D; // VBlank[7:0] = 45 & 0xFF = 0x2D
        raw[13] = 0x40; // VActive[11:8]=0x4, VBlank[11:8]=0x0
        raw[14] = 0x58; // HFront=88
        raw[15] = 0x2C; // HSync=44
        raw[16] = 0x45; // VFront=4, VSync=5
        raw[17] = 0x00; // high bits for porches
        raw[18] = 0x00;
        raw[19] = 0x00;
        raw[20] = 0x00;
        raw[21] = 0x00;
        raw[22] = 0x00;
        raw[23] = 0x1E; // separate sync +H +V

        let block = EdidBlock { raw };
        let header = block.cta_header().unwrap();
        assert_eq!(header.revision, 3);
        assert_eq!(header.dtd_offset, 6);
        assert_eq!(header.native_dtd_count, 2);
        assert!(header.underscan);
        assert!(header.basic_audio);
        assert!(header.ycbcr_444);
        assert!(header.ycbcr_422);

        let timings = block.cta_detailed_timings().unwrap();
        assert_eq!(timings.len(), 1);
        assert_eq!(timings[0].h_active, 1920);
        assert_eq!(timings[0].v_active, 1080);
        assert_eq!(timings[0].pixel_clock_khz, 148500);
    }

    #[test]
    fn cta_header_rejects_non_cta_block() {
        let raw = [0u8; 128];
        let block = EdidBlock { raw };
        assert!(matches!(block.cta_header(), Err(ExtensionError::NotCta861)));
        assert!(matches!(
            block.cta_detailed_timings(),
            Err(ExtensionError::NotCta861)
        ));
    }

    #[test]
    fn cta_speaker_allocation_view_decodes_and_rejects_empty() {
        let block = CtaDataBlock {
            tag: 4,
            payload: vec![0x07, 0x05, 0x00], // FL/FR (bit 0), LFE (bit 1), FC (bit 2), FLW/FRW (byte 1 bit 0), TC (byte 1 bit 2)
        };
        match block.view().unwrap() {
            CtaDataBlockView::SpeakerAllocation(spk) => {
                assert!(spk.front_left_right());
                assert!(spk.lfe());
                assert!(spk.front_center());
                assert!(!spk.rear_left_right());
                assert!(spk.front_left_right_wide());
                assert!(spk.top_center());
                assert!(!spk.front_left_right_high());
            }
            other => panic!("expected SpeakerAllocation, got {other:?}"),
        }

        let empty = CtaDataBlock {
            tag: 4,
            payload: vec![],
        };
        assert!(matches!(
            empty.view(),
            Err(ExtensionError::InvalidSpeakerAllocationDataBlockLength { length: 0 })
        ));
    }

    #[test]
    fn cta_colorimetry_and_video_capability_views() {
        // Colorimetry (ext tag 0x05)
        let color_block = CtaDataBlock {
            tag: 7,
            payload: vec![0x05, 0x85, 0x01], // xvYCC601 (bit 0), sYCC601 (bit 2), BT2020RGB (bit 7), md_flags = 1
        };
        match color_block.view().unwrap() {
            CtaDataBlockView::Extended(CtaExtendedDataBlockView::Colorimetry(c)) => {
                assert!(c.xvycc601);
                assert!(!c.xvycc709);
                assert!(c.sycc601);
                assert!(!c.adobe_rgb);
                assert!(c.bt2020_rgb);
                assert_eq!(c.md_flags, 1);
            }
            other => panic!("expected Colorimetry, got {other:?}"),
        }

        // Video Capability (ext tag 0x00)
        let vcap_block = CtaDataBlock {
            tag: 7,
            payload: vec![0x00, 0xE4], // Q=1 (bit 7), QS=1 (bit 6), PT=2 (bits 5..4 = 10), IT=1 (bits 3..2 = 01), CE=0 (bits 1..0 = 00)
        };
        match vcap_block.view().unwrap() {
            CtaDataBlockView::Extended(CtaExtendedDataBlockView::VideoCapability(v)) => {
                assert!(v.selectable_quantization_range_rgb);
                assert!(v.selectable_quantization_range_ycc);
                assert_eq!(v.pt_behavior, 2);
                assert_eq!(v.it_behavior, 1);
                assert_eq!(v.ce_behavior, 0);
            }
            other => panic!("expected VideoCapability, got {other:?}"),
        }

        // Truncated extended blocks
        let short_color = CtaDataBlock {
            tag: 7,
            payload: vec![0x05, 0x01], // len 2 < 3
        };
        assert!(matches!(
            short_color.view(),
            Err(ExtensionError::TruncatedExtendedDataBlock {
                extended_tag: 0x05,
                length: 2,
                minimum: 3
            })
        ));
    }

    #[test]
    fn cta_freesync_and_dolby_vision_vsdb_views() {
        // FreeSync OUI: 0x00001A -> [0x1A, 0x00, 0x00]
        let fs_payload = vec![0x1A, 0x00, 0x00, 0x01, 48, 144, 0x01];
        let fs_block = CtaDataBlock {
            tag: 3,
            payload: fs_payload,
        };
        match fs_block.view().unwrap() {
            CtaDataBlockView::VendorSpecific(CtaVendorSpecificBlock::AmdFreeSync {
                version,
                min_refresh_hz,
                max_refresh_hz,
                flags,
                ..
            }) => {
                assert_eq!(version, 1);
                assert_eq!(min_refresh_hz, Some(48));
                assert_eq!(max_refresh_hz, Some(144));
                assert_eq!(flags, 1);
            }
            other => panic!("expected AmdFreeSync, got {other:?}"),
        }

        // Dolby Vision OUI: 0x00D046 -> [0x46, 0xD0, 0x00]
        let dv_payload = vec![0x46, 0xD0, 0x00, 0x02, 0x11, 0x22];
        let dv_block = CtaDataBlock {
            tag: 3,
            payload: dv_payload,
        };
        match dv_block.view().unwrap() {
            CtaDataBlockView::VendorSpecific(CtaVendorSpecificBlock::DolbyVision {
                version,
                ..
            }) => {
                assert_eq!(version, 2);
            }
            other => panic!("expected DolbyVision, got {other:?}"),
        }
    }

    #[test]
    fn constructs_cta_block_from_data_blocks() {
        let blocks = [CtaDataBlock {
            tag: 2,
            payload: vec![16, 31],
        }];
        let block = EdidBlock::from_cta_data_blocks(3, &blocks).unwrap();
        assert_eq!(block.raw[0], 0x02);
        assert_eq!(block.raw[1], 3);
        assert_eq!(block.raw[2], 7);
        assert_eq!(block.cta_data_blocks().unwrap(), blocks);
        assert_eq!(block.validate(), Ok(()));
    }

    #[test]
    fn cta_constructor_zeroes_padding_and_dtd_area() {
        let block = EdidBlock::from_cta_data_blocks(3, &[]).unwrap();
        assert_eq!(&block.raw[4..127], &[0; 123]);
        assert!(block.cta_detailed_timings_flagged().unwrap().is_empty());
    }

    #[test]
    fn constructs_cta_block_with_detailed_timings_and_native_count() {
        let timing = crate::timing::all_presets()[0].clone();
        let block = EdidBlock::from_cta_data_blocks_and_timings(3, &[], &[timing]).unwrap();
        assert_eq!(block.raw[2], 4);
        assert_eq!(block.raw[3] & 0x0F, 1);
        assert_eq!(block.cta_detailed_timings().unwrap().len(), 1);
        assert_eq!(block.validate(), Ok(()));
    }

    #[test]
    fn rejects_cta_dtds_without_space() {
        let timings = (0..7)
            .map(|_| crate::timing::all_presets()[0].clone())
            .collect::<Vec<_>>();
        assert!(matches!(
            EdidBlock::from_cta_data_blocks_and_timings(3, &[], &timings),
            Err(ExtensionWriteError::CtaDtdsTooLong {
                count: 7,
                maximum: 6
            })
        ));
    }

    #[test]
    fn rejects_cta_data_collection_that_cannot_fit() {
        let blocks = vec![
            CtaDataBlock {
                tag: 2,
                payload: vec![0; 31],
            };
            4
        ];
        assert!(matches!(
            EdidBlock::from_cta_data_blocks(3, &blocks),
            Err(ExtensionWriteError::CtaDataBlocksTooLong {
                length: 128,
                maximum: 123
            })
        ));
    }

    #[test]
    fn replaces_cta_data_blocks_and_recomputes_layout() {
        let mut block = EdidBlock::from_cta_data_blocks(
            3,
            &[CtaDataBlock {
                tag: 2,
                payload: vec![16],
            }],
        )
        .unwrap();
        block
            .replace_cta_data_blocks(&[CtaDataBlock {
                tag: 1,
                payload: vec![0x09, 0x07, 0x07],
            }])
            .unwrap();
        assert_eq!(block.raw[2], 8);
        assert_eq!(block.cta_data_blocks().unwrap()[0].tag, 1);
        assert_eq!(block.validate(), Ok(()));
    }
    #[test]
    fn encodes_raw_cta_data_block_header_and_payload() {
        let block = CtaDataBlock {
            tag: 2,
            payload: vec![16, 31],
        };
        assert_eq!(block.encode().unwrap(), vec![0x42, 16, 31]);
    }
    #[test]
    fn encodes_lossless_cta_typed_views() {
        let video = CtaDataBlockView::Video {
            modes: vec![
                CtaVideoMode {
                    vic: 16,
                    native: true,
                },
                CtaVideoMode {
                    vic: 31,
                    native: false,
                },
            ],
        };
        assert_eq!(
            video.to_data_block().unwrap(),
            CtaDataBlock {
                tag: 2,
                payload: vec![0x90, 31]
            }
        );

        let audio = CtaDataBlockView::Audio {
            descriptors: vec![CtaAudioDescriptor {
                format: 1,
                channels: 2,
                sample_rates: 0x07,
                format_specific: 0x10,
            }],
        };
        assert_eq!(
            audio.to_data_block().unwrap(),
            CtaDataBlock {
                tag: 1,
                payload: vec![0x09, 0x07, 0x10]
            }
        );

        let unknown = CtaDataBlockView::Unknown {
            tag: 5,
            payload: vec![1, 2, 3],
        };
        assert_eq!(
            unknown.to_data_block().unwrap(),
            CtaDataBlock {
                tag: 5,
                payload: vec![1, 2, 3]
            }
        );

        let vendor = CtaDataBlockView::VendorSpecific(CtaVendorSpecificBlock::Other {
            oui: [0xAA, 0xBB, 0xCC],
            payload: vec![0x11, 0x22],
        });
        assert_eq!(
            vendor.to_data_block().unwrap(),
            CtaDataBlock {
                tag: 3,
                payload: vec![0xAA, 0xBB, 0xCC, 0x11, 0x22],
            }
        );

        let extended = CtaDataBlockView::Extended(CtaExtendedDataBlockView::Unknown {
            extended_tag: 0x1A,
            payload: vec![0x01, 0x02],
        });
        assert_eq!(
            extended.to_data_block().unwrap(),
            CtaDataBlock {
                tag: 7,
                payload: vec![0x1A, 0x01, 0x02]
            }
        );
    }
    #[test]
    fn encodes_cta_speaker_colorimetry_and_video_capability_views() {
        let speaker = CtaDataBlockView::SpeakerAllocation(CtaSpeakerAllocation {
            raw_mask: [0x07, 0x05, 0x01],
        });
        assert_eq!(
            speaker.to_data_block().unwrap(),
            CtaDataBlock {
                tag: 4,
                payload: vec![0x07, 0x05, 0x01],
            }
        );

        let colorimetry =
            CtaDataBlockView::Extended(CtaExtendedDataBlockView::Colorimetry(CtaColorimetry {
                xvycc601: true,
                xvycc709: false,
                sycc601: true,
                adobe_ycc601: false,
                adobe_rgb: true,
                bt2020_cycc: false,
                bt2020_ycc: true,
                bt2020_rgb: true,
                md_flags: 2,
            }));
        assert_eq!(
            colorimetry.to_data_block().unwrap(),
            CtaDataBlock {
                tag: 7,
                payload: vec![0x05, 0xD5, 0x02],
            }
        );

        let capability = CtaDataBlockView::Extended(CtaExtendedDataBlockView::VideoCapability(
            CtaVideoCapability {
                selectable_quantization_range_rgb: true,
                selectable_quantization_range_ycc: false,
                pt_behavior: 2,
                it_behavior: 1,
                ce_behavior: 3,
            },
        ));
        assert_eq!(
            capability.to_data_block().unwrap(),
            CtaDataBlock {
                tag: 7,
                payload: vec![0x00, 0xA7],
            }
        );
    }
    #[test]
    fn encodes_hdr_static_metadata_and_preserves_extra_raw_bytes() {
        let hdr = CtaDataBlockView::Extended(CtaExtendedDataBlockView::HdrStaticMetadata {
            eotf_flags: 0x07,
            metadata_descriptor_flags: 0x01,
            max_luminance: Some(0x40),
            max_frame_average_luminance: Some(0x20),
            min_luminance: Some(0x01),
            raw: vec![0x06, 0x07, 0x01, 0x40, 0x20, 0x01, 0xAA],
        });
        assert_eq!(
            hdr.to_data_block().unwrap(),
            CtaDataBlock {
                tag: 7,
                payload: vec![0x06, 0x07, 0x01, 0x40, 0x20, 0x01, 0xAA],
            }
        );
    }

    #[test]
    fn typed_cta_encoders_roundtrip_through_views() {
        let views = [
            CtaDataBlockView::Video {
                modes: vec![CtaVideoMode {
                    vic: 16,
                    native: true,
                }],
            },
            CtaDataBlockView::Audio {
                descriptors: vec![CtaAudioDescriptor {
                    format: 1,
                    channels: 2,
                    sample_rates: 0x07,
                    format_specific: 0x10,
                }],
            },
            CtaDataBlockView::SpeakerAllocation(CtaSpeakerAllocation {
                raw_mask: [0x07, 0x05, 0x01],
            }),
            CtaDataBlockView::Extended(CtaExtendedDataBlockView::Colorimetry(CtaColorimetry {
                xvycc601: true,
                xvycc709: false,
                sycc601: true,
                adobe_ycc601: false,
                adobe_rgb: true,
                bt2020_cycc: false,
                bt2020_ycc: true,
                bt2020_rgb: true,
                md_flags: 2,
            })),
        ];
        for view in views {
            let encoded = view.to_data_block().unwrap();
            assert_eq!(encoded.view().unwrap(), view);
        }
    }
    #[test]
    fn encodes_known_vendor_views_losslessly_and_writes_typed_fields() {
        let source_blocks = [
            CtaDataBlock {
                tag: 3,
                payload: vec![0x03, 0x0C, 0x00, 0x10, 0x00, 0x38, 0x3C, 0x20, 0xAA],
            },
            CtaDataBlock {
                tag: 3,
                payload: vec![0xD8, 0x5D, 0xC4, 1, 0x78, 0xDC, 0x00, 0x01, 0xBB],
            },
            CtaDataBlock {
                tag: 3,
                payload: vec![0x1A, 0x00, 0x00, 1, 48, 144, 1, 0xCC],
            },
            CtaDataBlock {
                tag: 3,
                payload: vec![0x46, 0xD0, 0x00, 2, 0x11, 0x22, 0xDD],
            },
        ];
        for source in &source_blocks {
            let view = source.view().unwrap();
            assert_eq!(view.to_data_block().unwrap(), source.clone());
        }

        let mut hdmi14b = match source_blocks[0].view().unwrap() {
            CtaDataBlockView::VendorSpecific(block) => block,
            other => panic!("unexpected view: {other:?}"),
        };
        if let CtaVendorSpecificBlock::Hdmi14b {
            physical_address,
            max_tmds_clock_mhz,
            deep_color_flags,
            feature_flags,
            ..
        } = &mut hdmi14b
        {
            *physical_address = [0x20, 0x01];
            *max_tmds_clock_mhz = Some(400);
            *deep_color_flags = 0x70;
            *feature_flags = 0x40;
        }
        let encoded = CtaDataBlockView::VendorSpecific(hdmi14b)
            .to_data_block()
            .unwrap();
        assert_eq!(&encoded.payload[3..8], &[0x20, 0x01, 0x70, 0x50, 0x40]);

        let mut forum = match source_blocks[1].view().unwrap() {
            CtaDataBlockView::VendorSpecific(block) => block,
            other => panic!("unexpected view: {other:?}"),
        };
        if let CtaVendorSpecificBlock::HdmiForum {
            version,
            max_tmds_character_rate_mhz,
            scdc_flags,
            deep_color_420_flags,
            ..
        } = &mut forum
        {
            *version = 2;
            *max_tmds_character_rate_mhz = Some(700);
            *scdc_flags = 0xAA;
            *deep_color_420_flags = 0x02;
        }
        let encoded = CtaDataBlockView::VendorSpecific(forum)
            .to_data_block()
            .unwrap();
        assert_eq!(&encoded.payload[3..8], &[2, 0x8C, 0xAA, 0x00, 0x02]);

        let mut freesync = match source_blocks[2].view().unwrap() {
            CtaDataBlockView::VendorSpecific(block) => block,
            other => panic!("unexpected view: {other:?}"),
        };
        if let CtaVendorSpecificBlock::AmdFreeSync {
            version,
            min_refresh_hz,
            max_refresh_hz,
            flags,
            ..
        } = &mut freesync
        {
            *version = 2;
            *min_refresh_hz = Some(50);
            *max_refresh_hz = Some(165);
            *flags = 2;
        }
        let encoded = CtaDataBlockView::VendorSpecific(freesync)
            .to_data_block()
            .unwrap();
        assert_eq!(&encoded.payload[3..7], &[2, 50, 165, 2]);

        let mut dolby = match source_blocks[3].view().unwrap() {
            CtaDataBlockView::VendorSpecific(block) => block,
            other => panic!("unexpected view: {other:?}"),
        };
        if let CtaVendorSpecificBlock::DolbyVision { version, .. } = &mut dolby {
            *version = 3;
        }
        let encoded = CtaDataBlockView::VendorSpecific(dolby)
            .to_data_block()
            .unwrap();
        assert_eq!(encoded.payload, vec![0x46, 0xD0, 0x00, 3, 0x11, 0x22, 0xDD]);
    }
    #[test]
    fn validates_hdr_raw_prefix_and_drops_removed_luminance_bytes() {
        let malformed = CtaDataBlockView::Extended(CtaExtendedDataBlockView::HdrStaticMetadata {
            eotf_flags: 0,
            metadata_descriptor_flags: 0,
            max_luminance: None,
            max_frame_average_luminance: None,
            min_luminance: None,
            raw: vec![0x05, 0x01, 0x02],
        });
        assert!(matches!(
            malformed.to_data_block(),
            Err(ExtensionWriteError::InvalidCtaExtendedPayload {
                expected_tag: 0x06,
                actual_tag: Some(0x05),
                length: 3
            })
        ));

        let removed = CtaDataBlockView::Extended(CtaExtendedDataBlockView::HdrStaticMetadata {
            eotf_flags: 0x07,
            metadata_descriptor_flags: 0x01,
            max_luminance: None,
            max_frame_average_luminance: Some(0x20),
            min_luminance: Some(0x01),
            raw: vec![0x06, 0x07, 0x01, 0x40, 0x20, 0x01, 0xAA],
        });
        assert!(matches!(
            removed.to_data_block(),
            Err(ExtensionWriteError::InvalidCtaHdrLuminanceOrder)
        ));

        let retained = CtaDataBlockView::Extended(CtaExtendedDataBlockView::HdrStaticMetadata {
            eotf_flags: 0x07,
            metadata_descriptor_flags: 0x01,
            max_luminance: None,
            max_frame_average_luminance: None,
            min_luminance: None,
            raw: vec![0x06, 0x07, 0x01, 0x40, 0x20, 0x01, 0xAA],
        });
        assert!(matches!(
            retained.to_data_block(),
            Err(ExtensionWriteError::InvalidCtaHdrRawTail)
        ));
    }

    #[test]
    fn rejects_malformed_known_vendor_encoder_inputs() {
        let short = CtaDataBlockView::VendorSpecific(CtaVendorSpecificBlock::Hdmi14b {
            physical_address: [1, 0],
            max_tmds_clock_mhz: None,
            deep_color_flags: 0,
            feature_flags: 0,
            raw: vec![0x03, 0x0C, 0x00],
        });
        assert!(matches!(
            short.to_data_block(),
            Err(ExtensionWriteError::CtaPayloadTooShort {
                tag: 3,
                length: 3,
                minimum: 5
            })
        ));
        let default_valued_tail =
            CtaDataBlockView::VendorSpecific(CtaVendorSpecificBlock::Hdmi14b {
                physical_address: [0x10, 0],
                max_tmds_clock_mhz: None,
                deep_color_flags: 0,
                feature_flags: 0,
                raw: vec![0x03, 0x0C, 0x00, 0x10],
            });
        assert!(matches!(
            default_valued_tail.to_data_block(),
            Err(ExtensionWriteError::CtaPayloadTooShort {
                tag: 3,
                length: 4,
                minimum: 5
            })
        ));

        let wrong_oui = CtaDataBlockView::VendorSpecific(CtaVendorSpecificBlock::DolbyVision {
            version: 2,
            raw: vec![0x03, 0x0C, 0x00, 2],
        });
        assert!(matches!(
            wrong_oui.to_data_block(),
            Err(ExtensionWriteError::InvalidCtaVendorOui { .. })
        ));

        let invalid_rate = CtaDataBlockView::VendorSpecific(CtaVendorSpecificBlock::HdmiForum {
            version: 1,
            max_tmds_character_rate_mhz: Some(701),
            scdc_flags: 0,
            max_frl_rate: None,
            deep_color_420_flags: 0,
            vrr_flags: None,
            vrr_min_hz: None,
            vrr_max_hz: None,
            dsc_flags: None,
            dsc_max_slices: None,
            dsc_total_chunk_kbytes: None,
            raw: vec![0xD8, 0x5D, 0xC4, 1, 0, 0, 0, 0],
        });
        assert!(matches!(
            invalid_rate.to_data_block(),
            Err(ExtensionWriteError::InvalidCtaVendorRate {
                field: "max_tmds_character_rate_mhz",
                value: 701
            })
        ));
    }

    #[test]
    fn rejects_unrepresentable_cta_typed_view_fields() {
        assert!(matches!(
            (CtaDataBlockView::Video {
                modes: vec![CtaVideoMode {
                    vic: 128,
                    native: false
                }],
            })
            .to_data_block(),
            Err(ExtensionWriteError::InvalidCtaVideoCode { index: 0, vic: 128 })
        ));
        assert!(matches!(
            (CtaDataBlockView::Audio {
                descriptors: vec![CtaAudioDescriptor {
                    format: 32,
                    channels: 2,
                    sample_rates: 0,
                    format_specific: 0,
                }],
            })
            .to_data_block(),
            Err(ExtensionWriteError::InvalidCtaAudioFormat {
                index: 0,
                format: 32
            })
        ));
        assert!(matches!(
            (CtaDataBlockView::Audio {
                descriptors: vec![CtaAudioDescriptor {
                    format: 1,
                    channels: 0,
                    sample_rates: 0,
                    format_specific: 0,
                }],
            })
            .to_data_block(),
            Err(ExtensionWriteError::InvalidCtaAudioChannels {
                index: 0,
                channels: 0
            })
        ));
    }
    #[test]
    fn rejects_empty_and_malformed_cta_typed_payloads() {
        assert!(matches!(
            (CtaDataBlockView::Video { modes: vec![] }).to_data_block(),
            Err(ExtensionWriteError::CtaPayloadTooShort {
                tag: 2,
                length: 0,
                minimum: 1
            })
        ));
        assert!(matches!(
            (CtaDataBlockView::Audio {
                descriptors: vec![]
            })
            .to_data_block(),
            Err(ExtensionWriteError::CtaPayloadTooShort {
                tag: 1,
                length: 0,
                minimum: 3
            })
        ));
        assert!(matches!(
            CtaDataBlockView::Extended(CtaExtendedDataBlockView::AdaptiveSync(CtaAdaptiveSync {
                flags: 0,
                min_refresh_hz: 48,
                max_refresh_hz: 144,
                raw: vec![],
            }))
            .to_data_block(),
            Err(ExtensionWriteError::InvalidCtaExtendedPayload {
                expected_tag: 0x1A,
                actual_tag: None,
                length: 0
            })
        ));
        assert!(matches!(
            CtaDataBlockView::Extended(CtaExtendedDataBlockView::AdaptiveSync(CtaAdaptiveSync {
                flags: 0,
                min_refresh_hz: 48,
                max_refresh_hz: 144,
                raw: vec![0x05, 0x01],
            }))
            .to_data_block(),
            Err(ExtensionWriteError::InvalidCtaExtendedPayload {
                expected_tag: 0x1A,
                actual_tag: Some(0x05),
                length: 2
            })
        ));
        assert!(matches!(
            CtaDataBlockView::Extended(CtaExtendedDataBlockView::HdrStaticMetadata {
                eotf_flags: 0,
                metadata_descriptor_flags: 0,
                max_luminance: None,
                max_frame_average_luminance: Some(0x20),
                min_luminance: None,
                raw: vec![],
            })
            .to_data_block(),
            Err(ExtensionWriteError::InvalidCtaExtendedPayload {
                expected_tag: 0x06,
                actual_tag: None,
                length: 0
            })
        ));
    }

    #[test]
    fn rejects_unrepresentable_raw_cta_data_block() {
        let invalid_tag = CtaDataBlock {
            tag: 8,
            payload: Vec::new(),
        };
        assert!(matches!(
            invalid_tag.encode(),
            Err(ExtensionWriteError::InvalidCtaTag { tag: 8 })
        ));

        let oversized = CtaDataBlock {
            tag: 2,
            payload: vec![0; 32],
        };
        assert!(matches!(
            oversized.encode(),
            Err(ExtensionWriteError::CtaPayloadTooLong {
                length: 32,
                maximum: 31
            })
        ));
    }

    #[test]
    fn rejects_displayid_detailed_timing_payload_over_121_bytes() {
        let timing = DisplayIdDetailedTiming {
            pixel_clock_khz: 60_000,
            h_active: 1920,
            h_blank: 280,
            h_sync_offset: 88,
            h_sync_width: 44,
            v_active: 1080,
            v_blank: 45,
            v_sync_offset: 5,
            v_sync_width: 6,
            h_sync_positive: true,
            v_sync_positive: true,
            aspect_ratio: DisplayIdAspectRatio::OneToOne,
            interlaced: false,
            stereo_3d: DisplayIdStereo3d::Mono,
            preferred: false,
            ycbcr420: false,
        };
        let view = DisplayIdDataBlockView::DetailedTiming {
            timings: vec![timing; 7],
        };

        assert!(matches!(
            view.to_data_block_with_tag(0x03),
            Err(ExtensionWriteError::DisplayIdPayloadTooLong {
                length: 140,
                maximum: 121
            })
        ));
    }

    #[test]
    fn constructs_display_id_from_data_blocks() {
        let blocks = [DisplayIdDataBlock {
            tag: 0x55,
            revision: 1,
            payload: vec![0xAA, 0xBB],
        }];
        let block = EdidBlock::from_display_id_data_blocks(0x20, 2, 0, &blocks).unwrap();
        assert_eq!(block.raw[0], 0x70);
        assert_eq!(block.raw[1], 0x20);
        assert_eq!(block.display_id_data_blocks().unwrap(), blocks);
        assert_eq!(block.validate(), Ok(()));
    }

    #[test]
    fn rejects_display_id_payload_over_121_bytes() {
        let blocks = [DisplayIdDataBlock {
            tag: 1,
            revision: 0,
            payload: vec![0; 119],
        }];
        assert!(matches!(
            EdidBlock::from_display_id_data_blocks(0x20, 2, 0, &blocks),
            Err(ExtensionWriteError::DisplayIdPayloadTooLong {
                length: 122,
                maximum: 121
            })
        ));
    }

    #[test]
    fn replaces_display_id_data_blocks_and_preserves_header_fields() {
        let mut block = EdidBlock::from_display_id_data_blocks(
            0x20,
            2,
            3,
            &[DisplayIdDataBlock {
                tag: 0x55,
                revision: 1,
                payload: vec![0xAA],
            }],
        )
        .unwrap();
        block
            .replace_display_id_data_blocks(&[DisplayIdDataBlock {
                tag: 0x56,
                revision: 2,
                payload: vec![0xBB, 0xCC],
            }])
            .unwrap();
        assert_eq!(block.raw[1], 0x20);
        assert_eq!(block.raw[3], 2);
        assert_eq!(block.raw[4], 3);
        assert_eq!(block.display_id_data_blocks().unwrap()[0].tag, 0x56);
        assert_eq!(block.validate(), Ok(()));
    }
}
