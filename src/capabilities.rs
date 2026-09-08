//! Read-only aggregate capabilities across the Base, CTA-861, and DisplayID blocks.
//!
//! Every aggregated item carries its [`CapabilitySource`] so callers can trace
//! where a value came from. Conflicting ranges (e.g. multiple VRR sources) are
//! never silently resolved; use [`DisplayCapabilities::vrr_safe_intersection`]
//! to ask for an explicit safe overlap.

use crate::edid::Edid;
use crate::extensions::{
    CtaAudioDescriptor, DisplayIdEnumeratedTiming, DisplayIdFormulaTiming,
    DisplayIdInterfaceFeatures,
};
use crate::timing::DetailedTiming;

/// Where an aggregated capability was read from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapabilitySource {
    /// The base EDID block.
    Base,
    /// A CTA-861 extension block, identified by its zero-based index.
    CtaExtension {
        /// Zero-based extension index.
        index: usize,
    },
    /// A DisplayID extension block, identified by its zero-based index.
    DisplayIdExtension {
        /// Zero-based extension index.
        index: usize,
    },
    /// A block whose kind was not recognized.
    UnknownExtension {
        /// Zero-based extension index.
        index: usize,
    },
}

/// A discrete timing, source-tracked.
#[derive(Clone, Debug, PartialEq)]
pub enum DisplayTiming {
    /// A detailed timing descriptor from the Base block or a CTA-861 extension.
    Detailed(DetailedTiming),
    /// A DisplayID Type I/VII detailed timing descriptor.
    DisplayIdDetailed(crate::extensions::DisplayIdDetailedTiming),
    /// A DisplayID Type IX formula-based timing.
    Formula(DisplayIdFormulaTiming),
    /// A DisplayID Type VIII enumerated timing code.
    Enumerated(DisplayIdEnumeratedTiming),
}

/// A source-tracked timing entry.
#[derive(Clone, Debug, PartialEq)]
pub struct TimingSource {
    /// Where the timing came from.
    pub source: CapabilitySource,
    /// The timing value.
    pub timing: DisplayTiming,
}

/// A source-tracked VRR refresh-rate range (inclusive).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VrrRange {
    /// Where the range came from.
    pub source: CapabilitySource,
    /// Minimum refresh rate (Hz).
    pub min_hz: u16,
    /// Maximum refresh rate (Hz).
    pub max_hz: u16,
}

/// A source-tracked CTA audio descriptor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioSource {
    /// Where the descriptor came from.
    pub source: CapabilitySource,
    /// The audio descriptor.
    pub descriptor: CtaAudioDescriptor,
}

/// A source-tracked DisplayID interface-features block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterfaceSource {
    /// Where the block came from.
    pub source: CapabilitySource,
    /// The interface features.
    pub features: DisplayIdInterfaceFeatures,
}

/// Read-only aggregate display capabilities.
#[derive(Clone, Debug, PartialEq)]
pub struct DisplayCapabilities {
    /// Three-letter manufacturer ID from the base block, if present.
    pub manufacturer_id: Option<String>,
    /// Monitor product name, if present.
    pub product_name: Option<String>,
    /// All timings in source order (Base + CTA + DisplayID).
    pub timings: Vec<TimingSource>,
    /// All CTA audio descriptors in source order.
    pub audio: Vec<AudioSource>,
    /// All VRR refresh ranges in source order.
    pub vrr_ranges: Vec<VrrRange>,
    /// All DisplayID interface-features blocks in source order.
    pub interfaces: Vec<InterfaceSource>,
}

impl DisplayCapabilities {
    /// Compute the safe VRR intersection across all reported ranges.
    ///
    /// Returns `(low, high)` where `low` is the greatest minimum and `high` the
    /// least maximum across every source. Returns `None` when the sources
    /// conflict (i.e. no common refresh rate).
    #[must_use]
    pub fn vrr_safe_intersection(&self) -> Option<(u16, u16)> {
        let low = self.vrr_ranges.iter().map(|r| r.min_hz).max()?;
        let high = self.vrr_ranges.iter().map(|r| r.max_hz).min()?;
        (low <= high).then_some((low, high))
    }

    /// Return whether two or more VRR ranges conflict (no common refresh rate).
    #[must_use]
    pub fn has_vrr_conflict(&self) -> bool {
        self.vrr_ranges.len() > 1 && self.vrr_safe_intersection().is_none()
    }
}

impl Edid {
    /// Aggregate read-only display capabilities across all blocks.
    ///
    /// Blocks that cannot be fully parsed are skipped; the underlying blocks
    /// remain unchanged, preserving byte-for-byte lossless round-trips.
    #[must_use]
    pub fn display_capabilities(&self) -> DisplayCapabilities {
        let mut caps = DisplayCapabilities {
            manufacturer_id: None,
            product_name: None,
            timings: Vec::new(),
            audio: Vec::new(),
            vrr_ranges: Vec::new(),
            interfaces: Vec::new(),
        };

        if let Ok(meta) = self.base.metadata()
            && !meta.manufacturer_id.is_empty()
        {
            caps.manufacturer_id = Some(meta.manufacturer_id);
        }
        caps.product_name = self.monitor_name();

        for timing in self.base.detailed_timings() {
            caps.timings.push(TimingSource {
                source: CapabilitySource::Base,
                timing: DisplayTiming::Detailed(timing),
            });
        }

        for (index, extension) in self.extensions.iter().enumerate() {
            let source = match extension.extension_kind() {
                crate::extensions::ExtensionKind::Cta861 { .. } => {
                    CapabilitySource::CtaExtension { index }
                }
                crate::extensions::ExtensionKind::DisplayId { .. } => {
                    CapabilitySource::DisplayIdExtension { index }
                }
                crate::extensions::ExtensionKind::Unknown { .. } => {
                    CapabilitySource::UnknownExtension { index }
                }
            };

            match source {
                CapabilitySource::CtaExtension { .. } => {
                    collect_cta_extension(extension, source, &mut caps);
                }
                CapabilitySource::DisplayIdExtension { .. } => {
                    collect_display_id_extension(extension, source, &mut caps);
                }
                _ => {}
            }
        }

        caps
    }
}

fn collect_cta_extension(
    block: &crate::edid::EdidBlock,
    source: CapabilitySource,
    caps: &mut DisplayCapabilities,
) {
    if let Ok(timings) = block.cta_detailed_timings() {
        for timing in timings {
            caps.timings.push(TimingSource {
                source,
                timing: DisplayTiming::Detailed(timing),
            });
        }
    }
    if let Ok(blocks) = block.cta_data_blocks() {
        for data_block in blocks {
            if let Ok(view) = data_block.view() {
                use crate::extensions::CtaDataBlockView;
                use crate::extensions::CtaExtendedDataBlockView;
                use crate::extensions::CtaVendorSpecificBlock;
                match view {
                    CtaDataBlockView::Audio { descriptors } => {
                        for descriptor in descriptors {
                            caps.audio.push(AudioSource { source, descriptor });
                        }
                    }
                    CtaDataBlockView::VendorSpecific(CtaVendorSpecificBlock::AmdFreeSync {
                        min_refresh_hz,
                        max_refresh_hz,
                        ..
                    }) => {
                        if let (Some(min), Some(max)) = (min_refresh_hz, max_refresh_hz) {
                            caps.vrr_ranges.push(VrrRange {
                                source,
                                min_hz: u16::from(min),
                                max_hz: u16::from(max),
                            });
                        }
                    }
                    CtaDataBlockView::VendorSpecific(CtaVendorSpecificBlock::HdmiForum {
                        vrr_min_hz,
                        vrr_max_hz,
                        ..
                    }) => {
                        if let (Some(min), Some(max)) = (vrr_min_hz, vrr_max_hz) {
                            caps.vrr_ranges.push(VrrRange {
                                source,
                                min_hz: u16::from(min),
                                max_hz: max,
                            });
                        }
                    }
                    CtaDataBlockView::Extended(CtaExtendedDataBlockView::AdaptiveSync(sync)) => {
                        caps.vrr_ranges.push(VrrRange {
                            source,
                            min_hz: u16::from(sync.min_refresh_hz),
                            max_hz: u16::from(sync.max_refresh_hz),
                        });
                    }
                    _ => {}
                }
            }
        }
    }
}

fn collect_display_id_extension(
    block: &crate::edid::EdidBlock,
    source: CapabilitySource,
    caps: &mut DisplayCapabilities,
) {
    if let Ok(timings) = block.display_id_detailed_timings() {
        for timing in timings {
            caps.timings.push(TimingSource {
                source,
                timing: DisplayTiming::DisplayIdDetailed(timing),
            });
        }
    }
    if let Ok(timings) = block.display_id_formula_timings() {
        for timing in timings {
            caps.timings.push(TimingSource {
                source,
                timing: DisplayTiming::Formula(timing),
            });
        }
    }
    if let Ok(timings) = block.display_id_enumerated_timings() {
        for timing in timings {
            caps.timings.push(TimingSource {
                source,
                timing: DisplayTiming::Enumerated(timing),
            });
        }
    }
    if let Ok(ranges) = block.display_id_dynamic_video_timing_ranges() {
        for range in ranges {
            caps.vrr_ranges.push(VrrRange {
                source,
                min_hz: u16::from(range.min_vfreq_hz),
                max_hz: range.max_vfreq_hz,
            });
        }
    }
    if let Ok(ranges) = block.display_id_video_timing_range_limits() {
        for range in ranges {
            caps.vrr_ranges.push(VrrRange {
                source,
                min_hz: u16::from(range.min_vfreq_hz),
                max_hz: u16::from(range.max_vfreq_hz),
            });
        }
    }
    if let Ok(features) = block.display_id_interface_features() {
        for feature in features {
            caps.interfaces.push(InterfaceSource {
                source,
                features: feature,
            });
        }
    }
}
