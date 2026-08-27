//! Read-only views for EDID extension blocks, including selected CTA data blocks.

use crate::edid::EdidBlock;

mod cta;

pub use cta::{
    CtaAudioDescriptor, CtaColorimetry, CtaDataBlock, CtaDataBlockView, CtaExtendedDataBlockView,
    CtaHeader, CtaSpeakerAllocation, CtaVendorSpecificBlock, CtaVideoCapability, CtaVideoMode,
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
    /// Whether this entry is marked as preferred.
    pub preferred: bool,
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

/// Typed read-only views for DisplayID data blocks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DisplayIdDataBlockView {
    /// DisplayID 1.x or 2.x Product Identification Data Block.
    ProductIdentification {
        /// Original product-identification payload.
        raw: Vec<u8>,
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
                raw: self.payload.clone(),
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
            Self::ProductIdentification { raw } if matches!(tag, 0x00 | 0x20) => {
                check_display_id_payload_length(raw.len())?;
                Ok(DisplayIdDataBlock {
                    tag,
                    revision: 0,
                    payload: raw.clone(),
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
                        maximum: u8::MAX as usize,
                    },
                )?;
                if length > u8::MAX as usize {
                    return Err(ExtensionWriteError::DisplayIdPayloadTooLong {
                        length,
                        maximum: u8::MAX as usize,
                    });
                }
                let mut payload = Vec::with_capacity(length);
                for (index, timing) in timings.iter().enumerate() {
                    payload.extend_from_slice(&encode_display_id_timing(
                        timing, index, type_one, tag,
                    )?);
                }
                Ok(DisplayIdDataBlock {
                    tag,
                    revision: 0,
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
            _ => Err(ExtensionWriteError::InvalidDisplayIdTag { tag }),
        }
    }
}

fn encode_display_id_timing(
    timing: &DisplayIdDetailedTiming,
    index: usize,
    type_one: bool,
    tag: u8,
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
    bytes[3] = u8::from(timing.preferred) << 7;
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
    let mut timings = Vec::with_capacity(block.payload.len() / 20);
    for bytes in block.payload.as_chunks::<20>().0 {
        let pixel_clock = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0]) + 1;
        let hsync = u16::from_le_bytes([bytes[8], bytes[9]]);
        let vsync = u16::from_le_bytes([bytes[16], bytes[17]]);
        let clock_multiplier = if type_one { 10 } else { 1 };
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
            preferred: bytes[3] & 0x80 != 0,
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

        let mut collection = Vec::new();
        for data_block in blocks {
            collection.extend_from_slice(&data_block.encode()?);
        }
        const MAX_COLLECTION_LENGTH: usize = 123;
        if collection.len() > MAX_COLLECTION_LENGTH {
            return Err(ExtensionWriteError::CtaDataBlocksTooLong {
                length: collection.len(),
                maximum: MAX_COLLECTION_LENGTH,
            });
        }
        let dtd_offset = DATA_BLOCK_OFFSET + collection.len();
        let maximum = (127usize.saturating_sub(dtd_offset)) / DTD_SIZE;
        if timings.len() > maximum {
            return Err(ExtensionWriteError::CtaDtdsTooLong {
                count: timings.len(),
                maximum,
            });
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
        const MAX_PAYLOAD: usize = 121;

        let mut payload = Vec::new();
        for data_block in blocks {
            payload.extend_from_slice(&data_block.encode()?);
        }
        if payload.len() > MAX_PAYLOAD {
            return Err(ExtensionWriteError::DisplayIdPayloadTooLong {
                length: payload.len(),
                maximum: MAX_PAYLOAD,
            });
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
        const DISPLAY_ID_MAX_PAYLOAD: usize = 121;
        if payload_length > DISPLAY_ID_MAX_PAYLOAD {
            return Err(ExtensionError::InvalidDisplayIdLength {
                length: payload_length,
                maximum: DISPLAY_ID_MAX_PAYLOAD,
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
        // Only an offset-zero collection has an unknown boundary where a
        // zero-filled remainder can be treated as padding.
        if stop_at_zero_padding && header == 0 && data[offset..].iter().all(|&byte| byte == 0) {
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
        CtaAudioDescriptor, CtaColorimetry, CtaDataBlock, CtaDataBlockView,
        CtaExtendedDataBlockView, CtaSpeakerAllocation, CtaVendorSpecificBlock, CtaVideoCapability,
        CtaVideoMode, DisplayIdDataBlock, DisplayIdDataBlockView, DisplayIdDetailedTiming,
        DisplayIdDisplayParameters, DisplayIdHeader, ExtensionError, ExtensionKind,
        ExtensionWriteError,
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
            payload: vec![0x1A, 0x01, 0x02],
        };
        assert_eq!(
            adaptive_sync.view().unwrap(),
            CtaDataBlockView::Extended(CtaExtendedDataBlockView::AdaptiveSync {
                raw: vec![0x1A, 0x01, 0x02],
            })
        );
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
                    preferred: true,
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
                    preferred: false,
                }]
            }
        );
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
            deep_color_420_flags: 0,
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
            CtaDataBlockView::Extended(CtaExtendedDataBlockView::AdaptiveSync { raw: vec![] })
                .to_data_block(),
            Err(ExtensionWriteError::InvalidCtaExtendedPayload {
                expected_tag: 0x1A,
                actual_tag: None,
                length: 0
            })
        ));
        assert!(matches!(
            CtaDataBlockView::Extended(CtaExtendedDataBlockView::AdaptiveSync {
                raw: vec![0x05, 0x01]
            })
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
