//! Structured errors for strict EDID parsing and validation.

use std::fmt;

/// Errors returned when an EDID block cannot be accepted by the strict parser.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EdidError {
    /// The input is not exactly one complete EDID block.
    InvalidLength {
        /// Required EDID block length.
        expected: usize,
        /// Actual input length.
        actual: usize,
    },
    /// A base block does not begin with the required EDID header.
    InvalidHeader,
    /// The block bytes do not sum to zero modulo 256.
    InvalidChecksum {
        /// Sum of all block bytes modulo 256.
        sum: u8,
    },
    /// The input is not a whole sequence of EDID blocks.
    InvalidBlockSequenceLength {
        /// Actual input length in bytes.
        actual: usize,
    },
    /// The base block's extension count disagrees with the supplied blocks.
    ExtensionCountMismatch {
        /// Count declared by the base block.
        declared: usize,
        /// Number of extension blocks supplied.
        actual: usize,
    },
    /// The base block uses an unsupported EDID version.
    UnsupportedVersion {
        /// EDID major version.
        major: u8,
        /// EDID minor version.
        minor: u8,
    },
}

/// Errors returned while decoding or encoding base-block metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetadataError {
    /// The block does not contain the EDID base-block header.
    NotBaseBlock,
    /// Manufacturer ID contains a value outside A-Z.
    InvalidManufacturerId,
    /// Manufacturer ID contains a non-uppercase ASCII character.
    InvalidManufacturerCharacter {
        /// Character position in the three-letter ID.
        index: usize,
        /// Invalid character value.
        character: char,
    },
    /// Manufacture year cannot be represented by the EDID year offset.
    InvalidManufactureYear {
        /// Supplied absolute year.
        year: u16,
    },
    /// Manufacture week uses a reserved EDID encoding.
    InvalidManufactureWeek {
        /// Supplied EDID manufacture-week byte.
        week: u8,
    },
    /// Gamma is outside the EDID representable range, in hundredths.
    InvalidGamma {
        /// Supplied gamma multiplied by 100.
        value: u16,
    },
}
impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotBaseBlock => f.write_str("block is not an EDID base block"),
            Self::InvalidManufacturerId => f.write_str("invalid EDID manufacturer ID"),
            Self::InvalidManufacturerCharacter { index, character } => {
                write!(
                    f,
                    "invalid manufacturer character {character:?} at index {index}"
                )
            }
            Self::InvalidManufactureYear { year } => {
                write!(f, "manufacture year {year} is outside the EDID range")
            }
            Self::InvalidManufactureWeek { week } => {
                write!(f, "manufacture week {week} uses a reserved EDID encoding")
            }
            Self::InvalidGamma { value } => {
                write!(f, "gamma value {value} is outside the EDID range")
            }
        }
    }
}

impl std::error::Error for MetadataError {}

/// Errors returned while encoding the newly added base-block metadata views.
///
/// This is separate from [`MetadataError`] so existing exhaustive matches on
/// that public 0.1 enum remain source-compatible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetadataWriteError {
    /// The block does not contain the EDID base-block header.
    NotBaseBlock,
    /// A chromaticity coordinate exceeds its 10-bit EDID representation.
    InvalidChromaticityValue {
        /// Supplied encoded coordinate value.
        value: u16,
    },
    /// A standard timing cannot be represented by its two-byte encoding.
    InvalidStandardTiming {
        /// Supplied horizontal active size.
        horizontal_pixels: u16,
        /// Supplied refresh rate.
        refresh_rate_hz: u8,
    },
    /// A standard timing collides with the EDID unused-slot encoding.
    ReservedStandardTimingEncoding,
    /// A reserved standard-timing entry contains a valid mode encoding.
    InvalidReservedStandardTimingEncoding {
        /// Original two-byte encoding.
        raw: [u8; 2],
    },
}

impl fmt::Display for MetadataWriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotBaseBlock => f.write_str("block is not an EDID base block"),
            Self::InvalidChromaticityValue { value } => {
                write!(
                    f,
                    "chromaticity value {value} exceeds the 10-bit EDID range"
                )
            }
            Self::InvalidStandardTiming {
                horizontal_pixels,
                refresh_rate_hz,
            } => write!(
                f,
                "standard timing {horizontal_pixels} pixels at {refresh_rate_hz} Hz is not representable"
            ),
            Self::ReservedStandardTimingEncoding => {
                f.write_str("standard timing collides with the EDID unused-slot encoding")
            }
            Self::InvalidReservedStandardTimingEncoding { raw } => write!(
                f,
                "reserved standard timing encoding {raw:02X?} is a valid mode encoding"
            ),
        }
    }
}

impl std::error::Error for MetadataWriteError {}

/// Errors returned while decoding or encoding monitor descriptors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DescriptorError {
    /// The requested descriptor slot does not exist.
    SlotOutOfRange {
        /// Requested slot index.
        slot: usize,
        /// Number of descriptor slots.
        slots: usize,
    },
    /// The block does not contain the EDID base-block header.
    NotBaseBlock,
    /// Text is longer than the 13-byte descriptor payload.
    TextTooLong {
        /// Maximum text length in bytes.
        max: usize,
        /// Supplied text length in bytes.
        actual: usize,
    },
    /// Text contains a character that cannot be encoded in an EDID descriptor.
    NonAsciiText,
    /// A range-limit field is outside its one-byte EDID representation.
    RangeOutOfBounds,
    /// Chromaticity coordinate exceeds 10-bit range (0..=1023).
    InvalidChromaticityCoordinate {
        /// Coordinate value.
        value: u16,
    },
    /// Gamma is outside the EDID representable range, in hundredths.
    InvalidGamma {
        /// Supplied gamma multiplied by 100.
        value: u16,
    },
    /// Standard timing error in descriptor 0xFA.
    StandardTimingError(MetadataWriteError),
    /// Invalid descriptor revision number.
    InvalidRevision {
        /// Descriptor tag byte.
        tag: u8,
        /// Found revision byte.
        revision: u8,
    },
    /// Invalid Range Limits extension definition code or parameters.
    InvalidRangeExtension,
    /// Invalid CVT 3-byte timing code entry.
    InvalidCvtTiming {
        /// CVT slot index (0..4).
        slot: usize,
        /// Detailed reason.
        reason: &'static str,
    },
}
impl fmt::Display for DescriptorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SlotOutOfRange { slot, slots } => {
                write!(
                    f,
                    "descriptor slot {slot} is out of range ({} slots)",
                    slots
                )
            }
            Self::NotBaseBlock => f.write_str("block is not an EDID base block"),
            Self::TextTooLong { max, actual } => {
                write!(f, "descriptor text is {actual} bytes, maximum is {max}")
            }
            Self::NonAsciiText => f.write_str("descriptor text must be ASCII"),
            Self::RangeOutOfBounds => f.write_str("descriptor range field is out of bounds"),
            Self::InvalidChromaticityCoordinate { value } => {
                write!(
                    f,
                    "descriptor chromaticity coordinate {value} exceeds 10 bits"
                )
            }
            Self::InvalidGamma { value } => {
                write!(
                    f,
                    "descriptor gamma value {value} is outside the EDID range"
                )
            }
            Self::StandardTimingError(err) => write!(f, "descriptor standard timing error: {err}"),
            Self::InvalidRevision { tag, revision } => {
                write!(
                    f,
                    "invalid revision {revision:#04X} for descriptor tag {tag:#04X}"
                )
            }
            Self::InvalidRangeExtension => f.write_str("invalid range limits extension parameters"),
            Self::InvalidCvtTiming { slot, reason } => {
                write!(f, "invalid CVT 3-byte timing at slot {slot}: {reason}")
            }
        }
    }
}

impl std::error::Error for DescriptorError {}

/// DTD field that failed a representability check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DtdField {
    /// Horizontal active pixels.
    HorizontalActive,
    /// Vertical active lines.
    VerticalActive,
    /// Horizontal blanking pixels.
    HorizontalBlanking,
    /// Vertical blanking lines.
    VerticalBlanking,
    /// Horizontal front porch pixels.
    HorizontalFrontPorch,
    /// Horizontal sync width in pixels.
    HorizontalSync,
    /// Vertical front porch lines.
    VerticalFrontPorch,
    /// Vertical sync width in lines.
    VerticalSync,
    /// Horizontal border pixels.
    HorizontalBorder,
    /// Vertical border lines.
    VerticalBorder,
    /// Pixel clock in kHz.
    PixelClockKHz,
}

/// Errors returned when a timing cannot be represented by an EDID DTD.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DtdError {
    /// The requested DTD slot does not exist.
    SlotOutOfRange {
        /// Requested slot index.
        slot: usize,
        /// Number of available DTD slots.
        slots: usize,
    },
    /// A field has an invalid value even though it fits its bit width.
    InvalidField {
        /// Field with the invalid value.
        field: DtdField,
        /// Supplied field value.
        value: u32,
    },
    /// A field exceeds its EDID bit-field limit.
    FieldOutOfRange {
        /// Field that exceeded its limit.
        field: DtdField,
        /// Supplied field value.
        value: u32,
        /// Largest representable field value.
        max: u32,
    },
    /// An intermediate timing calculation overflowed.
    ArithmeticOverflow,
    /// The block has fewer reusable DTD slots than requested timings.
    NoAvailableSlot {
        /// Number of requested timings.
        requested: usize,
        /// Number of reusable slots.
        available: usize,
    },
}
impl fmt::Display for DtdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SlotOutOfRange { slot, slots } => {
                write!(f, "DTD slot {slot} is out of range ({} slots)", slots)
            }
            Self::InvalidField { field, value } => {
                write!(f, "DTD field {field:?} has invalid value {value}")
            }
            Self::FieldOutOfRange { field, value, max } => {
                write!(f, "DTD field {field:?} value {value} exceeds {max}")
            }
            Self::NoAvailableSlot {
                requested,
                available,
            } => write!(
                f,
                "requested {requested} timings but only {available} DTD slots are available"
            ),
            Self::ArithmeticOverflow => f.write_str("DTD timing arithmetic overflowed"),
        }
    }
}

impl std::error::Error for DtdError {}

/// Errors returned by the strict resolution-to-EDID serializer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SerializeError {
    /// Existing input is not an integral sequence of EDID blocks.
    InvalidExistingLength {
        /// Actual input length in bytes.
        actual: usize,
    },
    /// An existing block failed strict validation.
    InvalidExistingBlock {
        /// Zero-based block index.
        index: usize,
        /// Validation failure from that block.
        source: EdidError,
    },
    /// The base block's extension count disagrees with the supplied blocks.
    ExtensionCountMismatch {
        /// Count declared by the base block.
        declared: usize,
        /// Number of extension blocks supplied.
        actual: usize,
    },
    /// A requested resolution could not produce a timing.
    TimingUnavailable {
        /// Zero-based resolution index.
        index: usize,
    },
    /// A computed timing exceeds DTD field limits.
    TimingDoesNotFit {
        /// Zero-based resolution index.
        index: usize,
    },
    /// No reusable DTD slot remains for a requested timing.
    NoDtdSlot {
        /// Zero-based resolution index that could not be written.
        index: usize,
    },
    /// A manually supplied timing failed DTD validation.
    InvalidTiming {
        /// Zero-based timing index.
        index: usize,
        /// DTD validation failure.
        source: DtdError,
    },
    /// The extension count cannot be represented by the base block byte.
    TooManyExtensions {
        /// Number of extension blocks supplied.
        count: usize,
    },
}

impl fmt::Display for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidExistingLength { actual } => {
                write!(
                    f,
                    "existing EDID length {actual} is not a whole block sequence"
                )
            }
            Self::InvalidExistingBlock { index, source } => {
                write!(f, "existing EDID block {index} is invalid: {source}")
            }
            Self::ExtensionCountMismatch { declared, actual } => write!(
                f,
                "EDID declares {declared} extension blocks but supplied {actual}"
            ),
            Self::TooManyExtensions { count } => {
                write!(f, "EDID contains too many extension blocks: {count}")
            }
            Self::TimingUnavailable { index } => {
                write!(f, "resolution {index} could not produce a timing")
            }
            Self::TimingDoesNotFit { index } => {
                write!(f, "resolution {index} does not fit an EDID DTD")
            }
            Self::NoDtdSlot { index } => {
                write!(f, "no DTD slot is available for resolution {index}")
            }
            Self::InvalidTiming { index, source } => {
                write!(f, "timing {index} is invalid: {source}")
            }
        }
    }
}
impl std::error::Error for SerializeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidExistingBlock { source, .. } => Some(source),
            Self::InvalidTiming { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl fmt::Display for EdidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength { expected, actual } => {
                write!(
                    f,
                    "invalid EDID block length: expected {expected}, got {actual}"
                )
            }
            Self::InvalidBlockSequenceLength { actual } => {
                write!(f, "invalid EDID block sequence length: got {actual} bytes")
            }
            Self::InvalidHeader => f.write_str("invalid EDID base-block header"),
            Self::InvalidChecksum { sum } => {
                write!(f, "invalid EDID checksum: byte sum modulo 256 is {sum}")
            }
            Self::ExtensionCountMismatch { declared, actual } => write!(
                f,
                "EDID declares {declared} extension blocks but supplied {actual}"
            ),
            Self::UnsupportedVersion { major, minor } => {
                write!(f, "unsupported EDID version {major}.{minor}")
            }
        }
    }
}

impl std::error::Error for EdidError {}

/// Errors returned while parsing an X11 / xrandr Modeline string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelineError {
    /// Modeline string is empty or contains insufficient tokens.
    InsufficientTokens,
    /// A numeric parameter failed to parse.
    InvalidNumber {
        /// The invalid token.
        token: String,
    },
    /// Invalid geometry (e.g. sync start < active, sync end < sync start, total < sync end).
    InvalidGeometry(&'static str),
    /// Pixel clock is zero or mathematically invalid.
    InvalidPixelClock,
}

impl fmt::Display for ModelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientTokens => f.write_str("Modeline string has insufficient tokens"),
            Self::InvalidNumber { token } => {
                write!(f, "invalid numeric token in Modeline: {token:?}")
            }
            Self::InvalidGeometry(reason) => write!(f, "invalid Modeline geometry: {reason}"),
            Self::InvalidPixelClock => {
                f.write_str("Modeline pixel clock must be positive and non-zero")
            }
        }
    }
}

impl std::error::Error for ModelineError {}

/// Errors returned while parsing hexadecimal EDID strings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HexError {
    /// String contains a non-hexadecimal character.
    InvalidHexCharacter {
        /// Character byte offset.
        offset: usize,
        /// Invalid character.
        character: char,
    },
    /// Hex string has an odd number of hexadecimal digits.
    OddLength {
        /// Total number of hex digits found.
        length: usize,
    },
    /// Decoded byte count is not a multiple of EDID_BLOCK_SIZE (128 bytes).
    InvalidLength {
        /// Number of decoded bytes.
        bytes: usize,
    },
    /// EDID block or sequence validation failed.
    EdidError(EdidError),
}

impl fmt::Display for HexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHexCharacter { offset, character } => {
                write!(f, "invalid hex character {character:?} at index {offset}")
            }
            Self::OddLength { length } => {
                write!(f, "hex string has odd length ({length} nibbles)")
            }
            Self::InvalidLength { bytes } => {
                write!(
                    f,
                    "decoded {bytes} bytes, expected a multiple of 128 (EDID block size)"
                )
            }
            Self::EdidError(err) => write!(f, "EDID validation error: {err}"),
        }
    }
}

impl std::error::Error for HexError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EdidError(err) => Some(err),
            _ => None,
        }
    }
}
