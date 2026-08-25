//! Read-only views for EDID extension blocks, including selected CTA data blocks.

use crate::edid::EdidBlock;

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

/// Parsed CTA-861 extension block header and capability flags (Bytes 1-3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CtaHeader {
    /// CTA-861 extension revision byte (typically 3).
    pub revision: u8,
    /// Byte offset where 18-byte Detailed Timing Descriptors start, or 0 if no DTDs.
    pub dtd_offset: u8,
    /// Total number of Native DTDs in this block (from byte 3 bits 0..3).
    pub native_dtd_count: u8,
    /// IT video formats are underscanned by default (byte 3 bit 7).
    pub underscan: bool,
    /// Basic Audio is supported (byte 3 bit 6).
    pub basic_audio: bool,
    /// YCbCr 4:4:4 is supported (byte 3 bit 5).
    pub ycbcr_444: bool,
    /// YCbCr 4:2:2 is supported (byte 3 bit 4).
    pub ycbcr_422: bool,
}

/// A CTA Speaker Allocation Data Block (Tag 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CtaSpeakerAllocation {
    /// Raw 3-byte speaker allocation payload bitmask.
    pub raw_mask: [u8; 3],
}

impl CtaSpeakerAllocation {
    /// Front Left / Front Right (FL/FR, byte 0 bit 0).
    #[must_use]
    pub const fn front_left_right(self) -> bool {
        self.raw_mask[0] & 0x01 != 0
    }

    /// Low Frequency Effects / Subwoofer (LFE, byte 0 bit 1).
    #[must_use]
    pub const fn lfe(self) -> bool {
        self.raw_mask[0] & 0x02 != 0
    }

    /// Front Center (FC, byte 0 bit 2).
    #[must_use]
    pub const fn front_center(self) -> bool {
        self.raw_mask[0] & 0x04 != 0
    }

    /// Rear Left / Rear Right (RL/RR, byte 0 bit 3).
    #[must_use]
    pub const fn rear_left_right(self) -> bool {
        self.raw_mask[0] & 0x08 != 0
    }

    /// Rear Center (RC, byte 0 bit 4).
    #[must_use]
    pub const fn rear_center(self) -> bool {
        self.raw_mask[0] & 0x10 != 0
    }

    /// Front Left Center / Front Right Center (FLC/FRC, byte 0 bit 5).
    #[must_use]
    pub const fn front_left_right_center(self) -> bool {
        self.raw_mask[0] & 0x20 != 0
    }

    /// Rear Left Center / Rear Right Center (RLC/RRC, byte 0 bit 6).
    #[must_use]
    pub const fn rear_left_right_center(self) -> bool {
        self.raw_mask[0] & 0x40 != 0
    }

    /// Front Left Wide / Front Right Wide (FLW/FRW, byte 1 bit 0).
    #[must_use]
    pub const fn front_left_right_wide(self) -> bool {
        self.raw_mask[1] & 0x01 != 0
    }

    /// Front Left High / Front Right High (FLH/FRH, byte 1 bit 1).
    #[must_use]
    pub const fn front_left_right_high(self) -> bool {
        self.raw_mask[1] & 0x02 != 0
    }

    /// Top Center (TC, byte 1 bit 2).
    #[must_use]
    pub const fn top_center(self) -> bool {
        self.raw_mask[1] & 0x04 != 0
    }

    /// Front Center High (FCH, byte 1 bit 3).
    #[must_use]
    pub const fn front_center_high(self) -> bool {
        self.raw_mask[1] & 0x08 != 0
    }
}

/// A CTA Colorimetry Data Block (Extended Tag 0x05).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CtaColorimetry {
    /// xvYCC601 color space support (byte 1 bit 0).
    pub xvycc601: bool,
    /// xvYCC709 color space support (byte 1 bit 1).
    pub xvycc709: bool,
    /// sYCC601 color space support (byte 1 bit 2).
    pub sycc601: bool,
    /// AdobeYCC601 color space support (byte 1 bit 3).
    pub adobe_ycc601: bool,
    /// AdobeRGB color space support (byte 1 bit 4).
    pub adobe_rgb: bool,
    /// BT2020CYCC color space support (byte 1 bit 5).
    pub bt2020_cycc: bool,
    /// BT2020YCC color space support (byte 1 bit 6).
    pub bt2020_ycc: bool,
    /// BT2020RGB color space support (byte 1 bit 7).
    pub bt2020_rgb: bool,
    /// Gamut boundary metadata profile support flags (byte 2 bits 0..1).
    pub md_flags: u8,
}

/// A CTA Video Capability Data Block (Extended Tag 0x00).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CtaVideoCapability {
    /// Selectable RGB Quantization Range (Q, bit 7).
    pub selectable_quantization_range_rgb: bool,
    /// Selectable YCC Quantization Range (QS, bit 6).
    pub selectable_quantization_range_ycc: bool,
    /// Preferred Timing overscan/underscan behavior (PT, bits 5..4).
    pub pt_behavior: u8,
    /// IT Video overscan/underscan behavior (IT, bits 3..2).
    pub it_behavior: u8,
    /// CE Video overscan/underscan behavior (CE, bits 1..0).
    pub ce_behavior: u8,
}

/// A CTA data block with its three-bit tag and raw payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CtaDataBlock {
    /// CTA data-block tag.
    pub tag: u8,
    /// Data-block payload without the header byte.
    pub payload: Vec<u8>,
}

/// A CTA Short Audio Descriptor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CtaAudioDescriptor {
    /// CTA audio coding format, 1 = LPCM.
    pub format: u8,
    /// Maximum channel count.
    pub channels: u8,
    /// Supported sample-rate bit mask.
    pub sample_rates: u8,
    /// Format-specific third byte.
    pub format_specific: u8,
}

/// A CTA Video Data Block entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CtaVideoMode {
    /// CEA/CTA Video Identification Code without the native bit.
    pub vic: u8,
    /// Whether this entry is marked native.
    pub native: bool,
}

/// A CTA Vendor-Specific Data Block (Tag 3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CtaVendorSpecificBlock {
    /// HDMI 1.4b VSDB (IEEE OUI 0x000C03).
    Hdmi14b {
        /// Source Physical Address (CEC A.B.C.D).
        physical_address: [u8; 2],
        /// Maximum TMDS clock in MHz, if specified.
        max_tmds_clock_mhz: Option<u16>,
        /// Deep Color support flags (byte 5).
        deep_color_flags: u8,
        /// Latency and video feature flags (byte 7).
        feature_flags: u8,
        /// Original payload including the 3-byte OUI.
        raw: Vec<u8>,
    },
    /// HDMI Forum (HDMI 2.0+) VSDB (IEEE OUI 0xC45DD8).
    HdmiForum {
        /// HF-VSDB version (usually 1).
        version: u8,
        /// Maximum TMDS character rate in MHz, if specified (5 MHz units).
        max_tmds_character_rate_mhz: Option<u16>,
        /// SCDC and scrambling capability flags (byte 5).
        scdc_flags: u8,
        /// Deep Color 4:2:0 support flags (byte 7).
        deep_color_420_flags: u8,
        /// Original payload including the 3-byte OUI.
        raw: Vec<u8>,
    },
    /// AMD FreeSync VSDB (IEEE OUI 0x00001A).
    AmdFreeSync {
        /// FreeSync block version.
        version: u8,
        /// Minimum supported refresh rate in Hz, if specified.
        min_refresh_hz: Option<u8>,
        /// Maximum supported refresh rate in Hz, if specified.
        max_refresh_hz: Option<u8>,
        /// FreeSync capability flags (LFC, Native Color Space, etc.).
        flags: u8,
        /// Original payload including the 3-byte OUI.
        raw: Vec<u8>,
    },
    /// Dolby Vision VSDB (IEEE OUI 0x00D046).
    DolbyVision {
        /// Dolby Vision version byte.
        version: u8,
        /// Original payload including the 3-byte OUI.
        raw: Vec<u8>,
    },
    /// Any other Vendor-Specific Data Block.
    Other {
        /// 24-bit IEEE Organizationally Unique Identifier in Little-Endian byte order.
        oui: [u8; 3],
        /// Payload bytes after the 3-byte OUI.
        payload: Vec<u8>,
    },
}

/// Typed read-only views for CTA data blocks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CtaDataBlockView {
    /// Video Data Block entries.
    Video {
        /// Video Identification Code entries.
        modes: Vec<CtaVideoMode>,
    },
    /// Short Audio Descriptor entries.
    Audio {
        /// Audio descriptors in source order.
        descriptors: Vec<CtaAudioDescriptor>,
    },
    /// Speaker Allocation Data Block (Tag 4).
    SpeakerAllocation(CtaSpeakerAllocation),
    /// Vendor-Specific Data Block (Tag 3).
    VendorSpecific(CtaVendorSpecificBlock),
    /// Extended CTA data block.
    Extended(CtaExtendedDataBlockView),
    /// Any CTA tag not modeled by this version.
    Unknown {
        /// Three-bit CTA data-block tag.
        tag: u8,
        /// Uninterpreted payload bytes.
        payload: Vec<u8>,
    },
}
/// Typed views for selected CTA extended data blocks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CtaExtendedDataBlockView {
    /// Video Capability Data Block, extended tag 0x00.
    VideoCapability(CtaVideoCapability),
    /// Colorimetry Data Block, extended tag 0x05.
    Colorimetry(CtaColorimetry),
    /// CTA-861 HDR Static Metadata Data Block, extended tag 0x06.
    HdrStaticMetadata {
        /// Supported EOTF flags.
        eotf_flags: u8,
        /// Supported static metadata descriptor flags.
        metadata_descriptor_flags: u8,
        /// Optional desired content maximum luminance byte.
        max_luminance: Option<u8>,
        /// Optional frame-average maximum luminance byte.
        max_frame_average_luminance: Option<u8>,
        /// Optional minimum luminance byte.
        min_luminance: Option<u8>,
        /// Original extended-tag-prefixed payload.
        raw: Vec<u8>,
    },
    /// CTA Adaptive-Sync Data Block, extended tag 0x1A; raw fields retained.
    AdaptiveSync {
        /// Original extended-tag-prefixed payload.
        raw: Vec<u8>,
    },
    /// An unsupported CTA extended data block.
    Unknown {
        /// Extended tag code.
        extended_tag: u8,
        /// Payload bytes after the extended tag.
        payload: Vec<u8>,
    },
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
        /// Original embedded CTA payload.
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
        /// Supplied tag value.
        tag: u8,
    },
    /// CTA data-block payload exceeds its five-bit length field.
    CtaPayloadTooLong {
        /// Supplied payload length.
        length: usize,
        /// Maximum representable payload length.
        maximum: usize,
    },
    /// The complete CTA data-block collection does not fit before byte 127.
    CtaDataBlocksTooLong {
        /// Supplied collection length.
        length: usize,
        /// Maximum collection length.
        maximum: usize,
    },
    /// The CTA DTD collection exceeds the available 18-byte slots.
    CtaDtdsTooLong {
        /// Supplied DTD count.
        count: usize,
        /// Maximum DTD count for the collection.
        maximum: usize,
    },
    /// A CTA DTD cannot be represented by the EDID DTD fields.
    CtaDtdInvalid {
        /// Zero-based DTD index.
        index: usize,
        /// Underlying DTD validation error.
        source: crate::error::DtdError,
    },
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
            Self::CtaDataBlocksTooLong { length, maximum } => write!(
                f,
                "CTA data-block collection length {length} exceeds the {maximum}-byte maximum"
            ),
            Self::CtaDtdsTooLong { count, maximum } => {
                write!(
                    f,
                    "CTA DTD count {count} exceeds the {maximum}-slot maximum"
                )
            }
            Self::CtaDtdInvalid { index, source } => {
                write!(f, "CTA DTD {index} is invalid: {source}")
            }
        }
    }
}

impl std::error::Error for ExtensionWriteError {}

impl CtaDataBlock {
    /// Encode this block as a CTA data-block header followed by its payload.
    pub fn encode(&self) -> Result<Vec<u8>, ExtensionWriteError> {
        const MAX_PAYLOAD_LENGTH: usize = 0x1F;

        if self.tag > 0x07 {
            return Err(ExtensionWriteError::InvalidCtaTag { tag: self.tag });
        }
        if self.payload.len() > MAX_PAYLOAD_LENGTH {
            return Err(ExtensionWriteError::CtaPayloadTooLong {
                length: self.payload.len(),
                maximum: MAX_PAYLOAD_LENGTH,
            });
        }

        let mut encoded = Vec::with_capacity(self.payload.len() + 1);
        encoded.push((self.tag << 5) | self.payload.len() as u8);
        encoded.extend_from_slice(&self.payload);
        Ok(encoded)
    }

    /// Decode this data block into a typed read-only view.
    pub fn view(&self) -> Result<CtaDataBlockView, ExtensionError> {
        match self.tag {
            1 => {
                if self.payload.is_empty() || !self.payload.len().is_multiple_of(3) {
                    return Err(ExtensionError::InvalidAudioDataBlockLength {
                        length: self.payload.len(),
                    });
                }
                let descriptors = self
                    .payload
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .map(|bytes| CtaAudioDescriptor {
                        format: bytes[0] >> 3,
                        channels: (bytes[0] & 0x07) + 1,
                        sample_rates: bytes[1],
                        format_specific: bytes[2],
                    })
                    .collect();
                Ok(CtaDataBlockView::Audio { descriptors })
            }
            2 => {
                if self.payload.is_empty() {
                    return Err(ExtensionError::InvalidVideoDataBlockLength { length: 0 });
                }
                let mut modes = Vec::with_capacity(self.payload.len());
                for (index, &code) in self.payload.iter().enumerate() {
                    let vic = code & 0x7F;
                    if vic == 0 {
                        return Err(ExtensionError::InvalidVideoCode { index });
                    }
                    modes.push(CtaVideoMode {
                        vic,
                        native: code & 0x80 != 0,
                    });
                }
                Ok(CtaDataBlockView::Video { modes })
            }
            3 => self.vendor_specific_view(),
            4 => {
                if self.payload.is_empty() {
                    return Err(ExtensionError::InvalidSpeakerAllocationDataBlockLength {
                        length: 0,
                    });
                }
                let mut raw_mask = [0u8; 3];
                let len = self.payload.len().min(3);
                raw_mask[..len].copy_from_slice(&self.payload[..len]);
                Ok(CtaDataBlockView::SpeakerAllocation(CtaSpeakerAllocation {
                    raw_mask,
                }))
            }
            7 => self.extended_view(),
            tag => Ok(CtaDataBlockView::Unknown {
                tag,
                payload: self.payload.clone(),
            }),
        }
    }

    fn extended_view(&self) -> Result<CtaDataBlockView, ExtensionError> {
        let Some(&extended_tag) = self.payload.first() else {
            return Err(ExtensionError::TruncatedExtendedDataBlock {
                extended_tag: 0,
                length: 0,
                minimum: 1,
            });
        };
        match extended_tag {
            0x00 => {
                if self.payload.len() < 2 {
                    return Err(ExtensionError::TruncatedExtendedDataBlock {
                        extended_tag,
                        length: self.payload.len(),
                        minimum: 2,
                    });
                }
                let b = self.payload[1];
                Ok(CtaDataBlockView::Extended(
                    CtaExtendedDataBlockView::VideoCapability(CtaVideoCapability {
                        selectable_quantization_range_rgb: b & 0x80 != 0,
                        selectable_quantization_range_ycc: b & 0x40 != 0,
                        pt_behavior: (b >> 4) & 0x03,
                        it_behavior: (b >> 2) & 0x03,
                        ce_behavior: b & 0x03,
                    }),
                ))
            }
            0x05 => {
                if self.payload.len() < 3 {
                    return Err(ExtensionError::TruncatedExtendedDataBlock {
                        extended_tag,
                        length: self.payload.len(),
                        minimum: 3,
                    });
                }
                let b1 = self.payload[1];
                let b2 = self.payload[2];
                Ok(CtaDataBlockView::Extended(
                    CtaExtendedDataBlockView::Colorimetry(CtaColorimetry {
                        xvycc601: b1 & 0x01 != 0,
                        xvycc709: b1 & 0x02 != 0,
                        sycc601: b1 & 0x04 != 0,
                        adobe_ycc601: b1 & 0x08 != 0,
                        adobe_rgb: b1 & 0x10 != 0,
                        bt2020_cycc: b1 & 0x20 != 0,
                        bt2020_ycc: b1 & 0x40 != 0,
                        bt2020_rgb: b1 & 0x80 != 0,
                        md_flags: b2 & 0x03,
                    }),
                ))
            }
            0x06 => {
                if self.payload.len() < 3 {
                    return Err(ExtensionError::TruncatedExtendedDataBlock {
                        extended_tag,
                        length: self.payload.len(),
                        minimum: 3,
                    });
                }
                Ok(CtaDataBlockView::Extended(
                    CtaExtendedDataBlockView::HdrStaticMetadata {
                        eotf_flags: self.payload[1],
                        metadata_descriptor_flags: self.payload[2],
                        max_luminance: self.payload.get(3).copied(),
                        max_frame_average_luminance: self.payload.get(4).copied(),
                        min_luminance: self.payload.get(5).copied(),
                        raw: self.payload.clone(),
                    },
                ))
            }
            0x1A => Ok(CtaDataBlockView::Extended(
                CtaExtendedDataBlockView::AdaptiveSync {
                    raw: self.payload.clone(),
                },
            )),
            extended_tag => Ok(CtaDataBlockView::Extended(
                CtaExtendedDataBlockView::Unknown {
                    extended_tag,
                    payload: self.payload[1..].to_vec(),
                },
            )),
        }
    }

    fn vendor_specific_view(&self) -> Result<CtaDataBlockView, ExtensionError> {
        if self.payload.len() < 3 {
            return Err(ExtensionError::TruncatedVendorSpecificDataBlock {
                length: self.payload.len(),
            });
        }
        let oui = [self.payload[0], self.payload[1], self.payload[2]];
        match oui {
            // HDMI 1.4b OUI: 0x000C03 (Little-Endian: 0x03, 0x0C, 0x00)
            [0x03, 0x0C, 0x00] => {
                let physical_address = if self.payload.len() >= 5 {
                    [self.payload[3], self.payload[4]]
                } else {
                    [0, 0]
                };
                let deep_color_flags = self.payload.get(5).copied().unwrap_or(0);
                let max_tmds_clock_mhz = self
                    .payload
                    .get(6)
                    .copied()
                    .filter(|&b| b != 0)
                    .map(|b| b as u16 * 5);
                let feature_flags = self.payload.get(7).copied().unwrap_or(0);
                Ok(CtaDataBlockView::VendorSpecific(
                    CtaVendorSpecificBlock::Hdmi14b {
                        physical_address,
                        max_tmds_clock_mhz,
                        deep_color_flags,
                        feature_flags,
                        raw: self.payload.clone(),
                    },
                ))
            }
            // HDMI Forum OUI: 0xC45DD8 (Little-Endian: 0xD8, 0x5D, 0xC4)
            [0xD8, 0x5D, 0xC4] => {
                let version = self.payload.get(3).copied().unwrap_or(1);
                let max_tmds_character_rate_mhz = self
                    .payload
                    .get(4)
                    .copied()
                    .filter(|&b| b != 0)
                    .map(|b| b as u16 * 5);
                let scdc_flags = self.payload.get(5).copied().unwrap_or(0);
                let deep_color_420_flags = self.payload.get(7).copied().unwrap_or(0);
                Ok(CtaDataBlockView::VendorSpecific(
                    CtaVendorSpecificBlock::HdmiForum {
                        version,
                        max_tmds_character_rate_mhz,
                        scdc_flags,
                        deep_color_420_flags,
                        raw: self.payload.clone(),
                    },
                ))
            }
            // AMD FreeSync OUI: 0x00001A (Little-Endian: 0x1A, 0x00, 0x00)
            [0x1A, 0x00, 0x00] => {
                let version = self.payload.get(3).copied().unwrap_or(1);
                let min_refresh_hz = self.payload.get(4).copied();
                let max_refresh_hz = self.payload.get(5).copied();
                let flags = self.payload.get(6).copied().unwrap_or(0);
                Ok(CtaDataBlockView::VendorSpecific(
                    CtaVendorSpecificBlock::AmdFreeSync {
                        version,
                        min_refresh_hz,
                        max_refresh_hz,
                        flags,
                        raw: self.payload.clone(),
                    },
                ))
            }
            // Dolby Vision OUI: 0x00D046 (Little-Endian: 0x46, 0xD0, 0x00)
            [0x46, 0xD0, 0x00] => {
                let version = self.payload.get(3).copied().unwrap_or(0);
                Ok(CtaDataBlockView::VendorSpecific(
                    CtaVendorSpecificBlock::DolbyVision {
                        version,
                        raw: self.payload.clone(),
                    },
                ))
            }
            _ => Ok(CtaDataBlockView::VendorSpecific(
                CtaVendorSpecificBlock::Other {
                    oui,
                    payload: self.payload[3..].to_vec(),
                },
            )),
        }
    }
}

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
                let data_blocks = parse_cta_data_blocks(&self.payload, 0)?;
                Ok(DisplayIdDataBlockView::Cta { data_blocks, raw })
            }
            tag => Ok(DisplayIdDataBlockView::Unknown {
                tag,
                payload: self.payload.clone(),
            }),
        }
    }
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
        parse_cta_data_blocks(&self.raw[4..end], 4)
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
) -> Result<Vec<CtaDataBlock>, ExtensionError> {
    let mut blocks = Vec::new();
    let mut offset = 0;
    while offset < data.len() {
        let header = data[offset];
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
        CtaDataBlock, CtaDataBlockView, CtaExtendedDataBlockView, CtaVendorSpecificBlock,
        CtaVideoMode, DisplayIdDataBlockView, DisplayIdDetailedTiming, DisplayIdDisplayParameters,
        DisplayIdHeader, ExtensionError, ExtensionKind, ExtensionWriteError,
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
    fn encodes_raw_cta_data_block_header_and_payload() {
        let block = CtaDataBlock {
            tag: 2,
            payload: vec![16, 31],
        };
        assert_eq!(block.encode().unwrap(), vec![0x42, 16, 31]);
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
}
