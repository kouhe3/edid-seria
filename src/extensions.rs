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

impl CtaDataBlock {
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
}

/// Errors returned while reading an extension's structured view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExtensionError {
    /// The block is not a CTA-861 extension.
    NotCta861,
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
}

impl std::fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotCta861 => f.write_str("EDID block is not a CTA-861 extension"),
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

        let mut blocks = Vec::new();
        let mut offset = 4;
        while offset < end {
            let header = self.raw[offset];
            let tag = header >> 5;
            let length = (header & 0x1F) as usize;
            let payload_start = offset + 1;
            let payload_end = payload_start + length;
            if payload_end > end {
                return Err(ExtensionError::TruncatedDataBlock { offset, length });
            }
            blocks.push(CtaDataBlock {
                tag,
                payload: self.raw[payload_start..payload_end].to_vec(),
            });
            offset = payload_end;
        }
        Ok(blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CtaDataBlock, CtaDataBlockView, CtaExtendedDataBlockView, CtaVideoMode, ExtensionError,
        ExtensionKind,
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
}
