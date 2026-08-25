//! Builder for valid EDID base blocks.

use crate::edid::EdidBlock;
use crate::error::{DescriptorError, DtdError, MetadataError, MetadataWriteError};
use crate::metadata::{
    BaseMetadata, ChromaticityCoordinates, EstablishedTimings, MonitorDescriptor,
    StandardTimingEntry,
};
use crate::timing::DetailedTiming;

/// Explicit placement policy used by [`BaseBlockBuilder`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimingPlacement {
    /// Reuse existing timing slots first, then dummy/free slots; never overwrite descriptors.
    #[default]
    ReuseTimingsThenFreePreserveDescriptors,
}

/// Errors produced while constructing a base block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BaseBlockError {
    /// Identity metadata is not representable.
    Metadata(MetadataError),
    /// Chromaticity or standard timings are not representable.
    MetadataWrite(MetadataWriteError),
    /// A monitor descriptor is invalid.
    Descriptor(DescriptorError),
    /// A detailed timing cannot be represented or no slot is available.
    Dtd(DtdError),
}
impl From<MetadataError> for BaseBlockError {
    fn from(e: MetadataError) -> Self {
        Self::Metadata(e)
    }
}
impl From<MetadataWriteError> for BaseBlockError {
    fn from(e: MetadataWriteError) -> Self {
        Self::MetadataWrite(e)
    }
}
impl From<DescriptorError> for BaseBlockError {
    fn from(e: DescriptorError) -> Self {
        Self::Descriptor(e)
    }
}
impl From<DtdError> for BaseBlockError {
    fn from(e: DtdError) -> Self {
        Self::Dtd(e)
    }
}
impl core::fmt::Display for BaseBlockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Metadata(e) => write!(f, "base metadata: {e}"),
            Self::MetadataWrite(e) => write!(f, "base metadata encoding: {e}"),
            Self::Descriptor(e) => write!(f, "monitor descriptor: {e}"),
            Self::Dtd(e) => write!(f, "detailed timing: {e}"),
        }
    }
}
impl std::error::Error for BaseBlockError {}

/// Incrementally constructs a valid EDID base block.
#[derive(Clone, Debug, Default)]
pub struct BaseBlockBuilder {
    metadata: Option<BaseMetadata>,
    chromaticity: Option<ChromaticityCoordinates>,
    established: Option<EstablishedTimings>,
    standard: Option<[StandardTimingEntry; 8]>,
    descriptors: [Option<MonitorDescriptor>; 4],
    timings: Vec<DetailedTiming>,
    placement: TimingPlacement,
    pending_error: Option<BaseBlockError>,
}
impl BaseBlockBuilder {
    /// Create a builder using the library's minimal EDID defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Set identity and capability metadata.
    #[must_use]
    pub fn metadata(mut self, value: BaseMetadata) -> Self {
        self.metadata = Some(value);
        self
    }
    /// Set the four chromaticity points.
    #[must_use]
    pub fn chromaticity(mut self, value: ChromaticityCoordinates) -> Self {
        self.chromaticity = Some(value);
        self
    }
    /// Set established timing bitfields.
    #[must_use]
    pub fn established_timings(mut self, value: EstablishedTimings) -> Self {
        self.established = Some(value);
        self
    }
    /// Set all eight standard timing slots.
    #[must_use]
    pub fn standard_timings(mut self, value: [StandardTimingEntry; 8]) -> Self {
        self.standard = Some(value);
        self
    }
    /// Set one monitor descriptor slot (zero-based).
    ///
    /// An out-of-range slot is recorded and returned by [`Self::build`]; it is
    /// never silently ignored.
    #[must_use]
    pub fn monitor_descriptor(mut self, slot: usize, value: MonitorDescriptor) -> Self {
        if let Some(target) = self.descriptors.get_mut(slot) {
            *target = Some(value);
        } else {
            self.pending_error = Some(BaseBlockError::Descriptor(
                DescriptorError::SlotOutOfRange { slot, slots: 4 },
            ));
        }
        self
    }
    /// Select the explicit detailed-timing placement policy.
    #[must_use]
    pub fn timing_placement(mut self, placement: TimingPlacement) -> Self {
        self.placement = placement;
        self
    }
    /// Set the detailed timings, which are placed into reusable slots.
    #[must_use]
    pub fn detailed_timings(mut self, value: Vec<DetailedTiming>) -> Self {
        self.timings = value;
        self
    }
    /// Build the block atomically; failures return no partially written block.
    ///
    /// Detailed timings use the selected policy: existing timing slots are
    /// replaced before free slots, monitor descriptors are preserved, and
    /// unused former timing slots are cleared.
    pub fn build(self) -> Result<EdidBlock, BaseBlockError> {
        if let Some(error) = self.pending_error {
            return Err(error);
        }
        let mut block = EdidBlock::new_default();
        if let Some(value) = self.metadata {
            block.set_metadata(&value)?;
        }
        if let Some(value) = self.chromaticity {
            block.set_chromaticity(&value)?;
        }
        if let Some(value) = self.established {
            block.set_established_timings(&value)?;
        }
        if let Some(value) = self.standard {
            block.set_standard_timings(&value)?;
        }
        for (slot, value) in self.descriptors.into_iter().enumerate() {
            if let Some(value) = value {
                block.set_monitor_descriptor(slot, &value)?;
            }
        }
        match self.placement {
            TimingPlacement::ReuseTimingsThenFreePreserveDescriptors => {
                block.write_resolutions_checked(&self.timings)?;
            }
        }
        block.update_checksum();
        Ok(block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DtdError, MetadataError, all_presets};

    #[test]
    fn default_builder_returns_valid_minimal_base() {
        let block = BaseBlockBuilder::new().build().unwrap();
        assert_eq!(block.validate(), Ok(()));
        assert_eq!(block.raw[126], 0);
        assert_eq!(
            block
                .raw
                .iter()
                .fold(0u8, |sum, byte| sum.wrapping_add(*byte)),
            0
        );
    }

    #[test]
    fn invalid_metadata_is_reported_without_partial_result() {
        let metadata = BaseMetadata {
            manufacturer_id: "abC".into(),
            product_code: 0,
            serial_number: 0,
            manufacture_week: 0,
            manufacture_year: 2024,
            input: 0x80,
            horizontal_size_cm: 0,
            vertical_size_cm: 0,
            gamma: None,
            feature_flags: 0,
        };
        assert_eq!(
            BaseBlockBuilder::new().metadata(metadata).build(),
            Err(BaseBlockError::Metadata(
                MetadataError::InvalidManufacturerCharacter {
                    index: 0,
                    character: 'a'
                }
            ))
        );
    }

    #[test]
    fn descriptors_are_preserved_when_timings_are_added() {
        let block = BaseBlockBuilder::new()
            .monitor_descriptor(0, MonitorDescriptor::ProductName("panel".into()))
            .detailed_timings(vec![all_presets()[0].clone()])
            .build()
            .unwrap();
        assert_eq!(
            block.monitor_descriptor(0).unwrap(),
            Some(MonitorDescriptor::ProductName("panel".into()))
        );
    }

    #[test]
    fn too_many_timings_returns_structured_slot_error() {
        let timings = all_presets()[..].iter().take(5).cloned().collect();
        assert!(matches!(
            BaseBlockBuilder::new().detailed_timings(timings).build(),
            Err(BaseBlockError::Dtd(DtdError::NoAvailableSlot {
                requested: 5,
                available: 4
            }))
        ));
    }
}

#[cfg(test)]
mod boundary_tests {
    use super::*;
    use crate::{ChromaticityCoordinates, ChromaticityPoint, all_presets};
    #[test]
    fn descriptor_slot_four_is_structured_error() {
        assert!(matches!(
            BaseBlockBuilder::new()
                .monitor_descriptor(4, MonitorDescriptor::Dummy)
                .build(),
            Err(BaseBlockError::Descriptor(
                DescriptorError::SlotOutOfRange { slot: 4, slots: 4 }
            ))
        ));
    }
    #[test]
    fn invalid_chromaticity_is_rejected() {
        let point = ChromaticityPoint { x: 1024, y: 0 };
        let c = ChromaticityCoordinates {
            red: point,
            green: point,
            blue: point,
            white: point,
        };
        assert!(matches!(
            BaseBlockBuilder::new().chromaticity(c).build(),
            Err(BaseBlockError::MetadataWrite(
                MetadataWriteError::InvalidChromaticityValue { value: 1024 }
            ))
        ));
    }
    #[test]
    fn invalid_timing_returns_error() {
        let mut timing = all_presets()[0].clone();
        timing.h_active = 0;
        assert!(matches!(
            BaseBlockBuilder::new()
                .detailed_timings(vec![timing])
                .build(),
            Err(BaseBlockError::Dtd(_))
        ));
    }
}
