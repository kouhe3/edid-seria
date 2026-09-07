use super::{ExtensionError, ExtensionWriteError};

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

/// CTA-861 audio coding format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CtaAudioFormat {
    /// Linear PCM (code 1).
    Lpcm,
    /// AC-3 (code 2).
    Ac3,
    /// MPEG-1, layers 1 & 2 (code 3).
    Mpeg1,
    /// MPEG-1 layer 3 / MP3 (code 4).
    Mp3,
    /// MPEG-2 multichannel (code 5).
    Mpeg2,
    /// AAC LC (code 6).
    AacLc,
    /// DTS (code 7).
    Dts,
    /// ATRAC (code 8).
    Atrac,
    /// One Bit Audio (code 9).
    OneBitAudio,
    /// Enhanced AC-3 / DD+ (code 10).
    EnhancedAc3,
    /// DTS-HD (code 11).
    DtsHd,
    /// MAT / MLP (code 12).
    MatMlp,
    /// DST (code 13).
    Dst,
    /// WMA Pro (code 14).
    WmaPro,
    /// Extended audio format (code 15); the format code is in the third byte.
    Extended,
    /// Reserved / unknown code (0 or > 15).
    Reserved,
}

/// A CTA Short Audio Descriptor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CtaAudioDescriptor {
    /// CTA audio coding format, 1 = LPCM.
    pub format: u8,
    /// Maximum channel count.
    pub channels: u8,
    /// Supported sample-rate bit mask (bit0=32k, bit1=44.1k, bit2=48k,
    /// bit3=88.2k, bit4=96k, bit5=176.4k, bit6=192k).
    pub sample_rates: u8,
    /// Format-specific third byte.
    pub format_specific: u8,
}

impl CtaAudioDescriptor {
    /// Map the raw format code to a typed [`CtaAudioFormat`].
    #[must_use]
    pub const fn format_kind(&self) -> CtaAudioFormat {
        match self.format {
            1 => CtaAudioFormat::Lpcm,
            2 => CtaAudioFormat::Ac3,
            3 => CtaAudioFormat::Mpeg1,
            4 => CtaAudioFormat::Mp3,
            5 => CtaAudioFormat::Mpeg2,
            6 => CtaAudioFormat::AacLc,
            7 => CtaAudioFormat::Dts,
            8 => CtaAudioFormat::Atrac,
            9 => CtaAudioFormat::OneBitAudio,
            10 => CtaAudioFormat::EnhancedAc3,
            11 => CtaAudioFormat::DtsHd,
            12 => CtaAudioFormat::MatMlp,
            13 => CtaAudioFormat::Dst,
            14 => CtaAudioFormat::WmaPro,
            15 => CtaAudioFormat::Extended,
            _ => CtaAudioFormat::Reserved,
        }
    }

    /// Return whether the given nominal sample rate in kHz is supported.
    ///
    /// Valid `khz` values are 32, 44, 48, 88, 96, 176, and 192 (44 / 88 / 176
    /// denote the 44.1 / 88.2 / 176.4 kHz rates).
    #[must_use]
    pub const fn supports_sample_rate(&self, khz: u16) -> bool {
        let bit = match khz {
            32 => 0,
            44 => 1,
            48 => 2,
            88 => 3,
            96 => 4,
            176 => 5,
            192 => 6,
            _ => return false,
        };
        self.sample_rates & (1 << bit) != 0
    }

    /// Return the LPCM sample sizes in bits (a subset of 16, 20, 24), or empty
    /// when this descriptor is not LPCM.
    #[must_use]
    pub const fn lpcm_sample_size_bits(&self) -> &'static [u8] {
        if self.format != 1 {
            return &[];
        }
        // LPCM: byte2 bit0=16, bit1=20, bit2=24 (bits 7..3 reserved).
        match self.format_specific & 0x07 {
            0b000 => &[],
            0b001 => &[16],
            0b010 => &[20],
            0b100 => &[24],
            // Bit patterns are a combination; the common cases cover single sizes.
            _ => &[16, 20, 24],
        }
    }

    /// Return whether the LPCM descriptor supports the given sample size in bits.
    #[must_use]
    pub const fn lpcm_supports_sample_size(&self, bits: u8) -> bool {
        if self.format != 1 {
            return false;
        }
        let mask = match bits {
            16 => 0b001,
            20 => 0b010,
            24 => 0b100,
            _ => return false,
        };
        self.format_specific & mask != 0
    }
}

/// A CTA-861 HDR Dynamic Metadata entry.
///
/// Each entry is `[type_len][type_lo][type_hi][data...]`. The complete
/// type-specific bytes are kept verbatim in `data` so unknown fields survive a
/// typed round-trip; `version` and the SL-HDR flags are read-only views over it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CtaHdrDynamicMetadataEntry {
    /// HDR dynamic metadata type (e.g. 1 = ST 2094-10, 2 = SL-HDR, 4 = ST 2094-40).
    pub metadata_type: u16,
    /// Complete type-specific payload bytes (bytes after the 2-byte type).
    pub data: Vec<u8>,
    /// Original complete entry bytes (type_len byte + 2-byte type + data).
    pub raw: Vec<u8>,
}

impl CtaHdrDynamicMetadataEntry {
    /// Return the version for types 1, 2, and 4 (low nibble of the first data byte),
    /// or `None` for other types or empty payload.
    #[must_use]
    pub fn version(&self) -> Option<u8> {
        if matches!(self.metadata_type, 1 | 2 | 4) && !self.data.is_empty() {
            Some(self.data[0] & 0x0F)
        } else {
            None
        }
    }

    /// Whether SL-HDR1 (ETSI TS 103 433-1) is supported (type 2, version >= 1).
    #[must_use]
    pub fn sl_hdr1(&self) -> bool {
        self.sl_flag(0x10)
    }

    /// Whether SL-HDR2 (ETSI TS 103 433-2) is supported (type 2, version >= 1).
    #[must_use]
    pub fn sl_hdr2(&self) -> bool {
        self.sl_flag(0x20)
    }

    /// Whether SL-HDR3 (ETSI TS 103 433-3) is supported (type 2, version >= 1).
    #[must_use]
    pub fn sl_hdr3(&self) -> bool {
        self.sl_flag(0x40)
    }

    fn sl_flag(&self, mask: u8) -> bool {
        self.metadata_type == 2
            && self.version().is_some_and(|v| v >= 1)
            && !self.data.is_empty()
            && (self.data[0] & mask != 0)
    }
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
        /// Maximum Fixed Rate Link rate code (0..=6), if present (byte 6 bits 7..4).
        max_frl_rate: Option<u8>,
        /// Deep Color 4:2:0 support flags (byte 7).
        deep_color_420_flags: u8,
        /// VRR, ALLM, FVA, and gaming capability flags (byte 8), if present.
        vrr_flags: Option<u8>,
        /// Minimum supported VRR refresh rate in Hz (byte 9 bits 0..5), if present.
        vrr_min_hz: Option<u8>,
        /// Maximum supported VRR refresh rate in Hz (byte 9 bits 6..7 + byte 10), if present.
        vrr_max_hz: Option<u16>,
        /// DSC 1.2 capability flags (byte 11), if present.
        dsc_flags: Option<u8>,
        /// DSC max slices and max FRL rate (byte 12), if present.
        dsc_max_slices: Option<u8>,
        /// DSC total chunk kbytes (byte 13), if present.
        dsc_total_chunk_kbytes: Option<u8>,
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

impl CtaVendorSpecificBlock {
    /// Return whether this VSDB represents HDMI Forum with ALLM (Auto Low Latency Mode).
    #[must_use]
    pub fn is_allm_supported(&self) -> bool {
        match self {
            Self::HdmiForum { vrr_flags, .. } => vrr_flags.is_some_and(|f| (f & (1 << 1)) != 0),
            _ => false,
        }
    }

    /// Return whether this VSDB represents HDMI Forum with Fast Vactive (FVA).
    #[must_use]
    pub fn is_fva_supported(&self) -> bool {
        match self {
            Self::HdmiForum { vrr_flags, .. } => vrr_flags.is_some_and(|f| (f & (1 << 2)) != 0),
            _ => false,
        }
    }

    /// Return whether this VSDB represents HDMI Forum with Cinema VRR.
    #[must_use]
    pub fn is_cinema_vrr_supported(&self) -> bool {
        match self {
            Self::HdmiForum { vrr_flags, .. } => vrr_flags.is_some_and(|f| (f & (1 << 4)) != 0),
            _ => false,
        }
    }
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
    /// CTA-861 HDR Dynamic Metadata Data Block, extended tag 0x07.
    HdrDynamicMetadata {
        /// Dynamic HDR metadata entries in source order.
        entries: Vec<CtaHdrDynamicMetadataEntry>,
        /// Original extended-tag-prefixed payload.
        raw: Vec<u8>,
    },
    /// YCbCr 4:2:0 Video Data Block, extended tag 0x0E.
    Y420Video {
        /// 4:2:0-only Video Identification Code entries.
        modes: Vec<CtaVideoMode>,
    },
    /// YCbCr 4:2:0 Capability Map Data Block, extended tag 0x0F.
    Y420CapabilityMap {
        /// Original extended-tag-prefixed payload.
        raw: Vec<u8>,
    },
    /// CTA Adaptive-Sync Data Block, extended tag 0x1A.
    AdaptiveSync(CtaAdaptiveSync),
    /// An unsupported CTA extended data block.
    Unknown {
        /// Extended tag code.
        extended_tag: u8,
        /// Payload bytes after the extended tag.
        payload: Vec<u8>,
    },
}

/// A CTA-861 Adaptive-Sync Data Block (Extended Tag 0x1A).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CtaAdaptiveSync {
    /// Capability and descriptor flags (byte 1).
    pub flags: u8,
    /// Minimum supported refresh rate in Hz (byte 2).
    pub min_refresh_hz: u8,
    /// Maximum supported refresh rate in Hz (byte 3).
    pub max_refresh_hz: u8,
    /// Original extended-tag-prefixed payload (`[0x1A, ...]`).
    pub raw: Vec<u8>,
}

impl CtaAdaptiveSync {
    /// Create a new Adaptive-Sync descriptor with the specified refresh rates and flags.
    pub fn new(
        min_refresh_hz: u8,
        max_refresh_hz: u8,
        flags: u8,
    ) -> Result<Self, ExtensionWriteError> {
        if min_refresh_hz == 0 || max_refresh_hz == 0 || min_refresh_hz > max_refresh_hz {
            return Err(ExtensionWriteError::InvalidRefreshRateRange {
                min_refresh_hz,
                max_refresh_hz,
            });
        }
        let raw = vec![0x1A, flags, min_refresh_hz, max_refresh_hz];
        Ok(Self {
            flags,
            min_refresh_hz,
            max_refresh_hz,
            raw,
        })
    }
}

/// Resolved CTA-861 YCbCr 4:2:0 video capabilities for a CTA data collection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CtaY420Support {
    /// Video modes from regular Video Data Blocks that also support YCbCr 4:2:0 (via Capability Map).
    pub capability_map_modes: Vec<CtaVideoMode>,
    /// Video modes that only support YCbCr 4:2:0 (from Y420 Video Data Blocks).
    pub only_420_modes: Vec<CtaVideoMode>,
}

impl CtaY420Support {
    /// Resolve YCbCr 4:2:0 video capabilities across a collection of CTA data blocks.
    pub fn resolve_from_blocks(blocks: &[CtaDataBlock]) -> Result<Self, ExtensionError> {
        let mut standard_svds = Vec::new();
        let mut only_420_modes = Vec::new();
        let mut capability_maps = Vec::new();

        for block in blocks {
            match block.view()? {
                CtaDataBlockView::Video { modes } => {
                    standard_svds.extend(modes);
                }
                CtaDataBlockView::Extended(CtaExtendedDataBlockView::Y420Video { modes }) => {
                    only_420_modes.extend(modes);
                }
                CtaDataBlockView::Extended(CtaExtendedDataBlockView::Y420CapabilityMap { raw }) => {
                    capability_maps.push(raw);
                }
                _ => {}
            }
        }

        let mut capability_map_modes = Vec::new();
        if !capability_maps.is_empty() {
            if standard_svds.is_empty() {
                return Err(ExtensionError::Y420CapabilityMapMissingVideoDataBlock);
            }
            for raw in &capability_maps {
                if raw.len() <= 1 {
                    for &mode in &standard_svds {
                        if !capability_map_modes.contains(&mode) {
                            capability_map_modes.push(mode);
                        }
                    }
                } else {
                    for (byte_offset, &byte) in raw[1..].iter().enumerate() {
                        for bit in 0..8 {
                            if (byte & (1 << bit)) != 0 {
                                let svd_index = byte_offset * 8 + bit;
                                if svd_index >= standard_svds.len() {
                                    return Err(ExtensionError::Y420CapabilityMapIndexOutOfRange {
                                        index: svd_index,
                                        available_svds: standard_svds.len(),
                                    });
                                }
                                let mode = standard_svds[svd_index];
                                if !capability_map_modes.contains(&mode) {
                                    capability_map_modes.push(mode);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(Self {
            capability_map_modes,
            only_420_modes,
        })
    }

    /// Returns all VICs supporting YCbCr 4:2:0 without duplicates.
    #[must_use]
    pub fn all_420_vics(&self) -> Vec<u8> {
        let mut vics = Vec::new();
        for mode in self.capability_map_modes.iter().chain(&self.only_420_modes) {
            if !vics.contains(&mode.vic) {
                vics.push(mode.vic);
            }
        }
        vics
    }

    /// Returns whether the specified VIC supports YCbCr 4:2:0.
    #[must_use]
    pub fn supports_vic(&self, vic: u8) -> bool {
        self.capability_map_modes.iter().any(|m| m.vic == vic)
            || self.only_420_modes.iter().any(|m| m.vic == vic)
    }

    /// Returns whether the specified VIC is 4:2:0-only.
    #[must_use]
    pub fn is_420_only(&self, vic: u8) -> bool {
        self.only_420_modes.iter().any(|m| m.vic == vic)
            && !self.capability_map_modes.iter().any(|m| m.vic == vic)
    }
}
fn vendor_payload_template(
    raw: &[u8],
    expected_oui: [u8; 3],
) -> Result<Vec<u8>, ExtensionWriteError> {
    if raw.len() < 3 {
        return Err(ExtensionWriteError::CtaPayloadTooShort {
            tag: 3,
            length: raw.len(),
            minimum: 3,
        });
    }
    let actual_oui = [raw[0], raw[1], raw[2]];
    if actual_oui != expected_oui {
        return Err(ExtensionWriteError::InvalidCtaVendorOui {
            expected: expected_oui,
            actual: actual_oui,
        });
    }
    Ok(raw.to_vec())
}

fn write_vendor_byte(
    payload: &mut [u8],
    raw: &[u8],
    offset: usize,
    value: u8,
    default: u8,
) -> Result<(), ExtensionWriteError> {
    if raw.get(offset).copied().unwrap_or(default) != value {
        if raw.len() <= offset {
            return Err(ExtensionWriteError::CtaPayloadTooShort {
                tag: 3,
                length: raw.len(),
                minimum: offset + 1,
            });
        }
        payload[offset] = value;
    }
    Ok(())
}

fn write_vendor_optional_byte(
    payload: &mut [u8],
    raw: &[u8],
    offset: usize,
    value: Option<u8>,
    field: &'static str,
) -> Result<(), ExtensionWriteError> {
    if value.is_none() && raw.get(offset).is_some() {
        return Err(ExtensionWriteError::InvalidCtaVendorField { field });
    }
    if raw.get(offset).copied() != value {
        if raw.len() <= offset {
            return Err(ExtensionWriteError::CtaPayloadTooShort {
                tag: 3,
                length: raw.len(),
                minimum: offset + 1,
            });
        }
        payload[offset] = value.expect("a present value was checked above");
    }
    Ok(())
}

fn write_vendor_rate(
    payload: &mut [u8],
    raw: &[u8],
    offset: usize,
    value: Option<u16>,
    field: &'static str,
) -> Result<(), ExtensionWriteError> {
    let encoded = match value {
        None => 0,
        Some(rate) if rate != 0 && rate % 5 == 0 && rate / 5 <= u16::from(u8::MAX) => {
            (rate / 5) as u8
        }
        Some(rate) => {
            return Err(ExtensionWriteError::InvalidCtaVendorRate { field, value: rate });
        }
    };
    let decoded = raw
        .get(offset)
        .copied()
        .filter(|&byte| byte != 0)
        .map(|byte| u16::from(byte) * 5);
    if value != decoded {
        if raw.len() <= offset {
            return Err(ExtensionWriteError::CtaPayloadTooShort {
                tag: 3,
                length: raw.len(),
                minimum: offset + 1,
            });
        }
        payload[offset] = encoded;
    }
    Ok(())
}
impl CtaDataBlockView {
    /// Encode a typed CTA view without discarding fields represented by the view.
    pub fn to_data_block(&self) -> Result<CtaDataBlock, ExtensionWriteError> {
        (match self {
            Self::Video { modes } => {
                if modes.is_empty() {
                    return Err(ExtensionWriteError::CtaPayloadTooShort {
                        tag: 2,
                        length: 0,
                        minimum: 1,
                    });
                }
                let mut payload = Vec::with_capacity(modes.len());
                for (index, mode) in modes.iter().enumerate() {
                    if !(1..=127).contains(&mode.vic) {
                        return Err(ExtensionWriteError::InvalidCtaVideoCode {
                            index,
                            vic: mode.vic,
                        });
                    }
                    payload.push(mode.vic | u8::from(mode.native) << 7);
                }
                Ok(CtaDataBlock { tag: 2, payload })
            }
            Self::Audio { descriptors } => {
                if descriptors.is_empty() {
                    return Err(ExtensionWriteError::CtaPayloadTooShort {
                        tag: 1,
                        length: 0,
                        minimum: 3,
                    });
                }
                let mut payload = Vec::with_capacity(descriptors.len() * 3);
                for (index, descriptor) in descriptors.iter().enumerate() {
                    if !(1..=15).contains(&descriptor.format) {
                        return Err(ExtensionWriteError::InvalidCtaAudioFormat {
                            index,
                            format: descriptor.format,
                        });
                    }
                    if !(1..=8).contains(&descriptor.channels) {
                        return Err(ExtensionWriteError::InvalidCtaAudioChannels {
                            index,
                            channels: descriptor.channels,
                        });
                    }
                    if descriptor.sample_rates & 0x80 != 0 {
                        return Err(ExtensionWriteError::InvalidCtaAudioSampleRates {
                            index,
                            sample_rates: descriptor.sample_rates,
                        });
                    }
                    payload.extend_from_slice(&[
                        (descriptor.format << 3) | (descriptor.channels - 1),
                        descriptor.sample_rates,
                        descriptor.format_specific,
                    ]);
                }
                Ok(CtaDataBlock { tag: 1, payload })
            }
            Self::SpeakerAllocation(speaker) => Ok(CtaDataBlock {
                tag: 4,
                payload: speaker.raw_mask.to_vec(),
            }),
            Self::Unknown { tag, payload } => Ok(CtaDataBlock {
                tag: *tag,
                payload: payload.clone(),
            }),
            Self::Extended(CtaExtendedDataBlockView::Unknown {
                extended_tag,
                payload,
            }) => {
                let mut encoded = Vec::with_capacity(payload.len() + 1);
                encoded.push(*extended_tag);
                encoded.extend_from_slice(payload);
                Ok(CtaDataBlock {
                    tag: 7,
                    payload: encoded,
                })
            }
            Self::Extended(CtaExtendedDataBlockView::AdaptiveSync(adaptive_sync)) => {
                if adaptive_sync.raw.len() < 4 || adaptive_sync.raw.first().copied() != Some(0x1A) {
                    return Err(ExtensionWriteError::InvalidCtaExtendedPayload {
                        expected_tag: 0x1A,
                        actual_tag: adaptive_sync.raw.first().copied(),
                        length: adaptive_sync.raw.len(),
                    });
                }
                if adaptive_sync.min_refresh_hz == 0
                    || adaptive_sync.max_refresh_hz == 0
                    || adaptive_sync.min_refresh_hz > adaptive_sync.max_refresh_hz
                {
                    return Err(ExtensionWriteError::InvalidRefreshRateRange {
                        min_refresh_hz: adaptive_sync.min_refresh_hz,
                        max_refresh_hz: adaptive_sync.max_refresh_hz,
                    });
                }
                let mut payload = adaptive_sync.raw.clone();
                payload[1] = adaptive_sync.flags;
                payload[2] = adaptive_sync.min_refresh_hz;
                payload[3] = adaptive_sync.max_refresh_hz;
                Ok(CtaDataBlock { tag: 7, payload })
            }
            Self::Extended(CtaExtendedDataBlockView::Y420Video { modes }) => {
                if modes.is_empty() {
                    return Err(ExtensionWriteError::CtaPayloadTooShort {
                        tag: 7,
                        length: 0,
                        minimum: 1,
                    });
                }
                let mut payload = Vec::with_capacity(modes.len() + 1);
                payload.push(0x0E);
                for (index, mode) in modes.iter().enumerate() {
                    if !(1..=127).contains(&mode.vic) {
                        return Err(ExtensionWriteError::InvalidCtaVideoCode {
                            index,
                            vic: mode.vic,
                        });
                    }
                    payload.push(mode.vic | (u8::from(mode.native) << 7));
                }
                Ok(CtaDataBlock { tag: 7, payload })
            }
            Self::Extended(CtaExtendedDataBlockView::Y420CapabilityMap { raw }) => {
                if raw.first().copied() != Some(0x0F) {
                    return Err(ExtensionWriteError::InvalidCtaExtendedPayload {
                        expected_tag: 0x0F,
                        actual_tag: raw.first().copied(),
                        length: raw.len(),
                    });
                }
                Ok(CtaDataBlock {
                    tag: 7,
                    payload: raw.clone(),
                })
            }
            Self::VendorSpecific(CtaVendorSpecificBlock::Hdmi14b {
                physical_address,
                max_tmds_clock_mhz,
                deep_color_flags,
                feature_flags,
                raw,
            }) => {
                let mut payload = vendor_payload_template(raw, [0x03, 0x0C, 0x00])?;
                if raw.len() < 5 {
                    return Err(ExtensionWriteError::CtaPayloadTooShort {
                        tag: 3,
                        length: raw.len(),
                        minimum: 5,
                    });
                }
                write_vendor_byte(&mut payload, raw, 3, physical_address[0], 0)?;
                write_vendor_byte(&mut payload, raw, 4, physical_address[1], 0)?;
                write_vendor_rate(
                    &mut payload,
                    raw,
                    6,
                    *max_tmds_clock_mhz,
                    "max_tmds_clock_mhz",
                )?;
                write_vendor_byte(&mut payload, raw, 5, *deep_color_flags, 0)?;
                write_vendor_byte(&mut payload, raw, 7, *feature_flags, 0)?;
                Ok(CtaDataBlock { tag: 3, payload })
            }
            Self::VendorSpecific(CtaVendorSpecificBlock::HdmiForum {
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
            }) => {
                let mut payload = vendor_payload_template(raw, [0xD8, 0x5D, 0xC4])?;
                write_vendor_byte(&mut payload, raw, 3, *version, 1)?;
                write_vendor_rate(
                    &mut payload,
                    raw,
                    4,
                    *max_tmds_character_rate_mhz,
                    "max_tmds_character_rate_mhz",
                )?;
                write_vendor_byte(&mut payload, raw, 5, *scdc_flags, 0)?;
                if let Some(frl) = *max_frl_rate {
                    if frl > 6 {
                        return Err(ExtensionWriteError::InvalidCtaField {
                            field: "max_frl_rate",
                            value: frl,
                            maximum: 6,
                        });
                    }
                    if raw.len() <= 6 {
                        return Err(ExtensionWriteError::CtaPayloadTooShort {
                            tag: 3,
                            length: raw.len(),
                            minimum: 7,
                        });
                    }
                    payload[6] = (payload[6] & 0x0F) | (frl << 4);
                } else if raw.len() > 6 && (raw[6] >> 4) != 0 {
                    return Err(ExtensionWriteError::InvalidCtaVendorField {
                        field: "max_frl_rate",
                    });
                }
                write_vendor_byte(&mut payload, raw, 7, *deep_color_420_flags, 0)?;
                if let Some(flags) = *vrr_flags {
                    if raw.len() <= 8 {
                        return Err(ExtensionWriteError::CtaPayloadTooShort {
                            tag: 3,
                            length: raw.len(),
                            minimum: 9,
                        });
                    }
                    payload[8] = flags;
                } else if raw.len() > 8 && raw[8] != 0 {
                    return Err(ExtensionWriteError::InvalidCtaVendorField { field: "vrr_flags" });
                }
                if let (Some(min), Some(max)) = (*vrr_min_hz, *vrr_max_hz)
                    && max > 0
                    && u16::from(min) > max
                {
                    return Err(ExtensionWriteError::InvalidRefreshRateRange {
                        min_refresh_hz: min,
                        max_refresh_hz: max as u8,
                    });
                }
                if let Some(min) = *vrr_min_hz {
                    if min > 63 {
                        return Err(ExtensionWriteError::InvalidCtaField {
                            field: "vrr_min_hz",
                            value: min,
                            maximum: 63,
                        });
                    }
                    if raw.len() <= 9 {
                        return Err(ExtensionWriteError::CtaPayloadTooShort {
                            tag: 3,
                            length: raw.len(),
                            minimum: 10,
                        });
                    }
                    payload[9] = (payload[9] & 0xC0) | (min & 0x3F);
                }
                if let Some(max) = *vrr_max_hz {
                    if max > 1023 {
                        return Err(ExtensionWriteError::InvalidCtaVendorField {
                            field: "vrr_max_hz",
                        });
                    }
                    if raw.len() <= 10 {
                        return Err(ExtensionWriteError::CtaPayloadTooShort {
                            tag: 3,
                            length: raw.len(),
                            minimum: 11,
                        });
                    }
                    let upper = ((max >> 8) & 0x03) as u8;
                    payload[9] = (payload[9] & 0x3F) | (upper << 6);
                    payload[10] = (max & 0xFF) as u8;
                }
                if let Some(flags) = *dsc_flags {
                    if raw.len() <= 11 {
                        return Err(ExtensionWriteError::CtaPayloadTooShort {
                            tag: 3,
                            length: raw.len(),
                            minimum: 12,
                        });
                    }
                    payload[11] = flags;
                }
                if let Some(slices) = *dsc_max_slices {
                    if raw.len() <= 12 {
                        return Err(ExtensionWriteError::CtaPayloadTooShort {
                            tag: 3,
                            length: raw.len(),
                            minimum: 13,
                        });
                    }
                    payload[12] = slices;
                }
                if let Some(chunks) = *dsc_total_chunk_kbytes {
                    if raw.len() <= 13 {
                        return Err(ExtensionWriteError::CtaPayloadTooShort {
                            tag: 3,
                            length: raw.len(),
                            minimum: 14,
                        });
                    }
                    payload[13] = chunks;
                }
                Ok(CtaDataBlock { tag: 3, payload })
            }
            Self::VendorSpecific(CtaVendorSpecificBlock::AmdFreeSync {
                version,
                min_refresh_hz,
                max_refresh_hz,
                flags,
                raw,
            }) => {
                let mut payload = vendor_payload_template(raw, [0x1A, 0x00, 0x00])?;
                write_vendor_byte(&mut payload, raw, 3, *version, 1)?;
                write_vendor_optional_byte(
                    &mut payload,
                    raw,
                    4,
                    *min_refresh_hz,
                    "min_refresh_hz",
                )?;
                write_vendor_optional_byte(
                    &mut payload,
                    raw,
                    5,
                    *max_refresh_hz,
                    "max_refresh_hz",
                )?;
                write_vendor_byte(&mut payload, raw, 6, *flags, 0)?;
                Ok(CtaDataBlock { tag: 3, payload })
            }
            Self::VendorSpecific(CtaVendorSpecificBlock::DolbyVision { version, raw }) => {
                let mut payload = vendor_payload_template(raw, [0x46, 0xD0, 0x00])?;
                write_vendor_byte(&mut payload, raw, 3, *version, 0)?;
                Ok(CtaDataBlock { tag: 3, payload })
            }
            Self::VendorSpecific(CtaVendorSpecificBlock::Other { oui, payload }) => {
                let mut encoded = Vec::with_capacity(payload.len() + 3);
                encoded.extend_from_slice(oui);
                encoded.extend_from_slice(payload);
                Ok(CtaDataBlock {
                    tag: 3,
                    payload: encoded,
                })
            }
            Self::Extended(CtaExtendedDataBlockView::VideoCapability(capability)) => {
                for (field, value) in [
                    ("PT", capability.pt_behavior),
                    ("IT", capability.it_behavior),
                    ("CE", capability.ce_behavior),
                ] {
                    if value > 3 {
                        return Err(ExtensionWriteError::InvalidCtaField {
                            field,
                            value,
                            maximum: 3,
                        });
                    }
                }
                let value = u8::from(capability.selectable_quantization_range_rgb) << 7
                    | u8::from(capability.selectable_quantization_range_ycc) << 6
                    | capability.pt_behavior << 4
                    | capability.it_behavior << 2
                    | capability.ce_behavior;
                Ok(CtaDataBlock {
                    tag: 7,
                    payload: vec![0x00, value],
                })
            }
            Self::Extended(CtaExtendedDataBlockView::Colorimetry(colorimetry)) => {
                if colorimetry.md_flags > 3 {
                    return Err(ExtensionWriteError::InvalidCtaField {
                        field: "MD",
                        value: colorimetry.md_flags,
                        maximum: 3,
                    });
                }
                let flags = u8::from(colorimetry.xvycc601)
                    | u8::from(colorimetry.xvycc709) << 1
                    | u8::from(colorimetry.sycc601) << 2
                    | u8::from(colorimetry.adobe_ycc601) << 3
                    | u8::from(colorimetry.adobe_rgb) << 4
                    | u8::from(colorimetry.bt2020_cycc) << 5
                    | u8::from(colorimetry.bt2020_ycc) << 6
                    | u8::from(colorimetry.bt2020_rgb) << 7;
                Ok(CtaDataBlock {
                    tag: 7,
                    payload: vec![0x05, flags, colorimetry.md_flags],
                })
            }
            Self::Extended(CtaExtendedDataBlockView::HdrStaticMetadata {
                eotf_flags,
                metadata_descriptor_flags,
                max_luminance,
                max_frame_average_luminance,
                min_luminance,
                raw,
            }) => {
                if raw.len() < 3 || raw.first() != Some(&0x06) {
                    return Err(ExtensionWriteError::InvalidCtaExtendedPayload {
                        expected_tag: 0x06,
                        actual_tag: raw.first().copied(),
                        length: raw.len(),
                    });
                }
                if max_luminance.is_none()
                    && (max_frame_average_luminance.is_some() || min_luminance.is_some())
                    || max_frame_average_luminance.is_none() && min_luminance.is_some()
                {
                    return Err(ExtensionWriteError::InvalidCtaHdrLuminanceOrder);
                }
                let typed_prefix_length = 3
                    + usize::from(max_luminance.is_some())
                    + usize::from(max_frame_average_luminance.is_some())
                    + usize::from(min_luminance.is_some());
                if raw.len() > 6 && typed_prefix_length < 6 {
                    return Err(ExtensionWriteError::InvalidCtaHdrRawTail);
                }
                let raw_prefix_length = raw.len().min(6);
                let mut payload = vec![0x06, *eotf_flags, *metadata_descriptor_flags];
                if let Some(value) = max_luminance {
                    payload.push(*value);
                }
                if let Some(value) = max_frame_average_luminance {
                    payload.push(*value);
                }
                if let Some(value) = min_luminance {
                    payload.push(*value);
                }
                if raw.len() > raw_prefix_length {
                    payload.extend_from_slice(&raw[raw_prefix_length..]);
                }
                Ok(CtaDataBlock { tag: 7, payload })
            }
            Self::Extended(CtaExtendedDataBlockView::HdrDynamicMetadata { entries, .. }) => {
                let mut payload = vec![0x07];
                for (index, entry) in entries.iter().enumerate() {
                    let data_len = entry.data.len();
                    let type_len = 2 + data_len;
                    if type_len < 2 {
                        return Err(ExtensionWriteError::InvalidHdrDynamicMetadataEntry {
                            index,
                            reason: "entry length below minimum",
                        });
                    }
                    if (1 + type_len).checked_add(payload.len()).is_none()
                        || payload.len() + 1 + type_len > 0x1F
                    {
                        return Err(ExtensionWriteError::InvalidHdrDynamicMetadataEntry {
                            index,
                            reason: "entry overflows block payload",
                        });
                    }
                    payload.push(type_len as u8);
                    payload.extend_from_slice(&entry.metadata_type.to_le_bytes());
                    payload.extend_from_slice(&entry.data);
                }
                Ok(CtaDataBlock { tag: 7, payload })
            }
        })
        .and_then(validate_typed_cta_data_block)
    }
}
fn validate_typed_cta_data_block(block: CtaDataBlock) -> Result<CtaDataBlock, ExtensionWriteError> {
    if block.tag > 0x07 {
        return Err(ExtensionWriteError::InvalidCtaTag { tag: block.tag });
    }
    if block.payload.len() > 0x1F {
        return Err(ExtensionWriteError::CtaPayloadTooLong {
            length: block.payload.len(),
            maximum: 0x1F,
        });
    }
    Ok(block)
}
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
            0x07 => {
                let body = &self.payload[1..];
                let mut entries = Vec::new();
                let mut offset = 0;
                let mut index = 0;
                if body.is_empty() {
                    return Err(ExtensionError::TruncatedExtendedDataBlock {
                        extended_tag,
                        length: self.payload.len(),
                        minimum: 2,
                    });
                }
                while offset < body.len() {
                    if body.len() - offset < 3 {
                        return Err(ExtensionError::TruncatedDynamicHdrMetadataEntry {
                            index,
                            length: body.len() - offset,
                            minimum: 3,
                        });
                    }
                    let type_len = body[offset] as usize;
                    if type_len < 2 {
                        return Err(ExtensionError::InvalidDynamicHdrMetadataLength {
                            index,
                            length: type_len,
                        });
                    }
                    let entry_end = offset + 1 + type_len;
                    if entry_end > body.len() {
                        return Err(ExtensionError::TruncatedDynamicHdrMetadataEntry {
                            index,
                            length: type_len,
                            minimum: body.len() - offset,
                        });
                    }
                    let metadata_type = u16::from_le_bytes([body[offset + 1], body[offset + 2]]);
                    let data = body[offset + 3..entry_end].to_vec();
                    let raw = body[offset..entry_end].to_vec();
                    entries.push(CtaHdrDynamicMetadataEntry {
                        metadata_type,
                        data,
                        raw,
                    });
                    offset = entry_end;
                    index += 1;
                }
                Ok(CtaDataBlockView::Extended(
                    CtaExtendedDataBlockView::HdrDynamicMetadata {
                        entries,
                        raw: self.payload.clone(),
                    },
                ))
            }
            0x0E => {
                if self.payload.len() < 2 {
                    return Err(ExtensionError::TruncatedExtendedDataBlock {
                        extended_tag,
                        length: self.payload.len(),
                        minimum: 2,
                    });
                }
                let mut modes = Vec::with_capacity(self.payload.len() - 1);
                for (index, &code) in self.payload[1..].iter().enumerate() {
                    let vic = code & 0x7F;
                    if vic == 0 {
                        return Err(ExtensionError::InvalidVideoCode { index });
                    }
                    modes.push(CtaVideoMode {
                        vic,
                        native: code & 0x80 != 0,
                    });
                }
                Ok(CtaDataBlockView::Extended(
                    CtaExtendedDataBlockView::Y420Video { modes },
                ))
            }
            0x0F => Ok(CtaDataBlockView::Extended(
                CtaExtendedDataBlockView::Y420CapabilityMap {
                    raw: self.payload.clone(),
                },
            )),
            0x1A => {
                if self.payload.len() < 4 {
                    return Err(ExtensionError::TruncatedExtendedDataBlock {
                        extended_tag,
                        length: self.payload.len(),
                        minimum: 4,
                    });
                }
                let flags = self.payload[1];
                let min_refresh_hz = self.payload[2];
                let max_refresh_hz = self.payload[3];
                if min_refresh_hz == 0 || max_refresh_hz == 0 || min_refresh_hz > max_refresh_hz {
                    return Err(ExtensionError::InvalidRefreshRateRange {
                        min_refresh_hz,
                        max_refresh_hz,
                    });
                }
                Ok(CtaDataBlockView::Extended(
                    CtaExtendedDataBlockView::AdaptiveSync(CtaAdaptiveSync {
                        flags,
                        min_refresh_hz,
                        max_refresh_hz,
                        raw: self.payload.clone(),
                    }),
                ))
            }
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
                let max_frl_rate = self.payload.get(6).map(|&b| b >> 4);
                let deep_color_420_flags = self.payload.get(7).copied().unwrap_or(0);
                let vrr_flags = self.payload.get(8).copied();
                let vrr_min_hz = self.payload.get(9).map(|&b| b & 0x3F).filter(|&b| b != 0);
                let vrr_max_hz = if self.payload.len() >= 11 {
                    let b9 = self.payload[9];
                    let b10 = self.payload[10];
                    let max = (u16::from(b9 & 0xC0) << 2) | u16::from(b10);
                    if max != 0 { Some(max) } else { None }
                } else {
                    None
                };
                let dsc_flags = self.payload.get(11).copied();
                let dsc_max_slices = self.payload.get(12).copied();
                let dsc_total_chunk_kbytes = self.payload.get(13).copied();
                Ok(CtaDataBlockView::VendorSpecific(
                    CtaVendorSpecificBlock::HdmiForum {
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
