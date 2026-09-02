//! Resolution → EDID serialization pipeline.
//!
//! High-level counterpart to the low-level [`edid`](crate::edid) module:
//! takes user-facing resolution specs, computes DTDs (CVT-RB2 for PC
//! monitors, CEA/HDTV presets for TVs), rewrites the base block's
//! detailed-timing slots while preserving monitor descriptors, fixes the
//! checksum of every block, and returns the complete EDID binary.

use crate::edid::{EDID_BLOCK_SIZE, EdidBlock};
use crate::error::SerializeError;
use crate::timing::{DetailedTiming, TimingFormula, compute_cvt, dtd_fits};

/// PC vs HDTV timing style. Mirrors the GUI's mode selector.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TimingKind {
    /// CVT 1.2 reduced blanking (recommended for PC monitors).
    Pc,
    /// Standard CEA/HDTV blanking from the preset table.
    Hdtv,
}

/// Strategy for automatically creating or extending CTA-861 and DisplayID extension blocks.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ExtensionPolicy {
    /// Only write to the 4 Base Block DTD slots (legacy behavior).
    BaseOnly,
    /// Automatically allocate overflow timings to CTA-861 DTDs, and
    /// DTD-incompatible timings (e.g. v_front > 63) to DisplayID Detailed Timings.
    #[default]
    Auto,
    /// Prefer CTA-861 extension blocks for DTD-compatible timings.
    /// Returns an error if any timing cannot be represented as a DTD.
    PreferCta,
    /// Prefer DisplayID extension blocks for all overflow and DTD-incompatible timings.
    PreferDisplayId,
}

/// Options controlling high-level resolution and timing serialization.
#[derive(Clone, Debug, PartialEq)]
pub struct SerializeOptions {
    /// Policy for creating or appending extension blocks.
    pub extension_policy: ExtensionPolicy,
    /// Maximum allowed total EDID blocks (Base + Extensions, up to 256).
    pub max_blocks: usize,
    /// Whether to retain existing extension blocks from input EDID.
    pub preserve_existing_extensions: bool,
}

impl Default for SerializeOptions {
    fn default() -> Self {
        Self {
            extension_policy: ExtensionPolicy::Auto,
            max_blocks: crate::edid::MAX_EDID_BLOCKS,
            preserve_existing_extensions: true,
        }
    }
}

/// A user-requested resolution, before timing computation.
#[derive(Copy, Clone, Debug, PartialEq)]
/// A user-requested resolution, before timing computation.
pub struct ResolutionSpec {
    /// Horizontal active pixels.
    pub width: u32,
    /// Vertical active lines.
    pub height: u32,
    /// Refresh rate in Hz.
    pub refresh: f64,
    /// PC (CVT-RB2) or HDTV (CEA preset) timing style.
    pub kind: TimingKind,
}

/// Outcome of a serialization run.
#[derive(Clone, Debug, PartialEq)]
pub struct SerializedEdid {
    /// Complete EDID binary: base block + preserved extension blocks.
    pub bytes: Vec<u8>,
    /// Number of resolutions actually written to a DTD slot.
    pub written: usize,
    /// Number of resolutions skipped: timing could not be computed, did not
    /// fit a DTD, or no slot was available.
    pub skipped: usize,
}

/// Serialize resolutions into an EDID override binary.
///
/// `existing` is the display's current EDID (registry override or original
/// EDID); `None` or data shorter than one block starts from a minimal default
/// block. A trailing partial block is dropped, and at most 255 extension
/// blocks are retained because the base-block count is one byte. The base
/// block's detailed-timing slots are rewritten, monitor descriptors are
/// preserved, every retained block's checksum is fixed, and the extension
/// count byte is normalized to the retained blocks.
///
/// This compatibility API may discard malformed or excess input. Use
/// [`serialize_resolutions_checked`] when every input block must be retained.
#[must_use]
pub fn serialize_resolutions(
    existing: Option<&[u8]>,
    resolutions: &[ResolutionSpec],
) -> SerializedEdid {
    let mut blocks: Vec<EdidBlock> = existing
        .filter(|data| data.len() >= EDID_BLOCK_SIZE)
        .map(|data| {
            data.chunks(EDID_BLOCK_SIZE)
                .filter_map(EdidBlock::from_bytes)
                .take(u8::MAX as usize + 1)
                .collect()
        })
        .unwrap_or_default();

    if blocks.is_empty() {
        blocks.push(EdidBlock::new_default());
    }

    let timings: Vec<DetailedTiming> = resolutions.iter().filter_map(timing_for).collect();
    let mut skipped = resolutions.len() - timings.len();
    let written = blocks[0].write_resolutions(&timings);
    skipped += timings.len() - written;

    blocks[0].raw[126] = (blocks.len() - 1) as u8;
    for block in &mut blocks {
        block.update_checksum();
    }

    SerializedEdid {
        bytes: blocks.into_iter().flat_map(|b| b.raw.to_vec()).collect(),
        written,
        skipped,
    }
}
/// Strict counterpart to [`serialize_resolutions`].
///
/// Existing data must contain complete, checksum-valid EDID blocks. The base
/// block header and extension count are validated, and every requested
/// resolution must produce a DTD that fits an available slot.
pub fn serialize_resolutions_checked(
    existing: Option<&[u8]>,
    resolutions: &[ResolutionSpec],
) -> Result<SerializedEdid, SerializeError> {
    let blocks = parse_existing_blocks(existing)?;
    let timings: Vec<DetailedTiming> = resolutions
        .iter()
        .enumerate()
        .map(|(index, spec)| timing_for_checked(index, spec))
        .collect::<Result<_, _>>()?;
    serialize_timing_blocks(blocks, &timings)
}

/// Serialize caller-provided detailed timings into an EDID override.
///
/// This is the strict manual-timing entry point. Existing blocks are validated
/// and every timing must fit the EDID DTD representation.
pub fn serialize_timings(
    existing: Option<&[u8]>,
    timings: &[DetailedTiming],
) -> Result<SerializedEdid, SerializeError> {
    let blocks = parse_existing_blocks(existing)?;
    serialize_timing_blocks(blocks, timings)
}

/// Serialize resolutions into an EDID binary with automatic CTA/DisplayID extension generation.
///
/// When the base block's 4 DTD slots are filled or when timings exceed EDID 1.4 DTD constraints
/// (such as CVT-RB2 high refresh rates with vertical front porch > 63), this function creates
/// CTA-861 DTD extension blocks or DisplayID Type VII timing extension blocks according to `options.extension_policy`.
pub fn serialize_resolutions_extended(
    existing: Option<&[u8]>,
    resolutions: &[ResolutionSpec],
    options: &SerializeOptions,
) -> Result<SerializedEdid, SerializeError> {
    let blocks = parse_existing_blocks(existing)?;
    let allow_extensions = options.extension_policy != ExtensionPolicy::BaseOnly;
    let timings: Vec<DetailedTiming> = resolutions
        .iter()
        .enumerate()
        .map(|(index, spec)| timing_for_extended(index, spec, allow_extensions))
        .collect::<Result<_, _>>()?;
    serialize_extended_pipeline(blocks, &timings, options)
}

/// Serialize caller-provided detailed timings into an EDID binary with automatic extension generation.
///
/// Timings that fit standard DTDs are allocated to the base block first. Any overflow timings or
/// timings exceeding DTD constraints are automatically split into CTA-861 (up to 6 DTDs per block)
/// or DisplayID (up to 5 detailed timings per block) extension blocks according to `options.extension_policy`.
pub fn serialize_timings_extended(
    existing: Option<&[u8]>,
    timings: &[DetailedTiming],
    options: &SerializeOptions,
) -> Result<SerializedEdid, SerializeError> {
    let blocks = parse_existing_blocks(existing)?;
    serialize_extended_pipeline(blocks, timings, options)
}

fn parse_existing_blocks(existing: Option<&[u8]>) -> Result<Vec<EdidBlock>, SerializeError> {
    let Some(data) = existing else {
        return Ok(vec![EdidBlock::new_default()]);
    };
    if data.len() < EDID_BLOCK_SIZE
        || data.len() > crate::edid::MAX_EDID_BYTES
        || !data.len().is_multiple_of(EDID_BLOCK_SIZE)
    {
        return Err(SerializeError::InvalidExistingLength { actual: data.len() });
    }
    let mut blocks = Vec::with_capacity(data.len() / EDID_BLOCK_SIZE);
    for (index, chunk) in data.chunks(EDID_BLOCK_SIZE).enumerate() {
        let block = EdidBlock::from_bytes_checked(chunk)
            .map_err(|source| SerializeError::InvalidExistingBlock { index, source })?;
        if index == 0 {
            block
                .validate_base()
                .map_err(|source| SerializeError::InvalidExistingBlock { index, source })?;
        } else {
            block
                .validate_extension()
                .map_err(|source| SerializeError::InvalidExistingBlock { index, source })?;
        }
        blocks.push(block);
    }
    Ok(blocks)
}

fn validate_block_sequence(blocks: &[EdidBlock]) -> Result<(), SerializeError> {
    let extension_count = blocks.len() - 1;
    if extension_count > u8::MAX as usize {
        return Err(SerializeError::TooManyExtensions {
            count: extension_count,
        });
    }
    let declared = blocks[0].raw[126] as usize;
    if declared != extension_count {
        return Err(SerializeError::ExtensionCountMismatch {
            declared,
            actual: extension_count,
        });
    }
    Ok(())
}

fn serialize_timing_blocks(
    mut blocks: Vec<EdidBlock>,
    timings: &[DetailedTiming],
) -> Result<SerializedEdid, SerializeError> {
    validate_block_sequence(&blocks)?;
    for (index, timing) in timings.iter().enumerate() {
        crate::timing::validate_dtd(timing)
            .map_err(|source| SerializeError::InvalidTiming { index, source })?;
    }

    let written = blocks[0]
        .write_resolutions_checked(timings)
        .map_err(|source| match source {
            crate::error::DtdError::NoAvailableSlot { .. } => SerializeError::NoDtdSlot {
                index: timings.len().saturating_sub(1),
            },
            source => SerializeError::InvalidTiming { index: 0, source },
        })?;
    if written != timings.len() {
        return Err(SerializeError::NoDtdSlot { index: written });
    }

    for block in &mut blocks {
        block.update_checksum();
    }

    Ok(SerializedEdid {
        bytes: blocks.into_iter().flat_map(|block| block.raw).collect(),
        written,
        skipped: 0,
    })
}

fn timing_for_checked(
    index: usize,
    spec: &ResolutionSpec,
) -> Result<DetailedTiming, SerializeError> {
    if !spec.refresh.is_finite() {
        return Err(SerializeError::TimingUnavailable { index });
    }
    let timing = match spec.kind {
        TimingKind::Hdtv => {
            DetailedTiming::compute_hdtv_blanking(spec.width, spec.height, spec.refresh)
                .or_else(|| compute_cvt(spec.width, spec.height, spec.refresh, TimingFormula::CVT))
        }
        TimingKind::Pc => compute_cvt(spec.width, spec.height, spec.refresh, TimingFormula::CVTRB2),
    }
    .ok_or(SerializeError::TimingUnavailable { index })?;
    if !dtd_fits(&timing) {
        return Err(SerializeError::TimingDoesNotFit { index });
    }
    Ok(timing)
}

/// Compute the DTD for a resolution spec; `None` when the timing cannot be
/// computed or does not fit in an EDID DTD.
fn timing_for(spec: &ResolutionSpec) -> Option<DetailedTiming> {
    let timing = match spec.kind {
        TimingKind::Hdtv => {
            DetailedTiming::compute_hdtv_blanking(spec.width, spec.height, spec.refresh)
                .or_else(|| compute_cvt(spec.width, spec.height, spec.refresh, TimingFormula::CVT))
        }
        TimingKind::Pc => compute_cvt(spec.width, spec.height, spec.refresh, TimingFormula::CVTRB2),
    }?;
    dtd_fits(&timing).then_some(timing)
}

fn serialize_extended_pipeline(
    mut blocks: Vec<EdidBlock>,
    timings: &[DetailedTiming],
    options: &SerializeOptions,
) -> Result<SerializedEdid, SerializeError> {
    validate_block_sequence(&blocks)?;

    if !options.preserve_existing_extensions {
        blocks.truncate(1);
    }

    if options.extension_policy == ExtensionPolicy::BaseOnly {
        return serialize_timing_blocks(blocks, timings);
    }

    for (index, timing) in timings.iter().enumerate() {
        if timing.pixel_clock_khz == 0 {
            return Err(SerializeError::InvalidTiming {
                index,
                source: crate::error::DtdError::InvalidField {
                    field: crate::error::DtdField::PixelClockKHz,
                    value: 0,
                },
            });
        }
        if timing.h_active == 0 {
            return Err(SerializeError::InvalidTiming {
                index,
                source: crate::error::DtdError::InvalidField {
                    field: crate::error::DtdField::HorizontalActive,
                    value: 0,
                },
            });
        }
        if timing.v_active == 0 {
            return Err(SerializeError::InvalidTiming {
                index,
                source: crate::error::DtdError::InvalidField {
                    field: crate::error::DtdField::VerticalActive,
                    value: 0,
                },
            });
        }
        if !timing.v_rate.is_finite() {
            return Err(SerializeError::TimingUnavailable { index });
        }
    }

    let mut dtd_compatible = Vec::new();
    let mut dtd_incompatible = Vec::new();

    for (index, timing) in timings.iter().enumerate() {
        if dtd_fits(timing) {
            dtd_compatible.push(timing.clone());
        } else {
            if options.extension_policy == ExtensionPolicy::PreferCta {
                return Err(SerializeError::TimingDoesNotFit { index });
            }
            dtd_incompatible.push(timing.clone());
        }
    }

    let written_to_base = blocks[0].write_resolutions(&dtd_compatible);
    let overflow_dtds = &dtd_compatible[written_to_base..];

    match options.extension_policy {
        ExtensionPolicy::Auto => {
            for chunk in overflow_dtds.chunks(6) {
                let cta_block = EdidBlock::from_cta_data_blocks_and_timings(3, &[], chunk)
                    .map_err(|_| SerializeError::NoDtdSlot { index: 0 })?;
                blocks.push(cta_block);
            }
            let did_timings: Vec<crate::extensions::DisplayIdDetailedTiming> = dtd_incompatible
                .iter()
                .map(|t| t.to_display_id(false))
                .collect();
            for chunk in did_timings.chunks(5) {
                let view = crate::extensions::DisplayIdDataBlockView::DetailedTiming {
                    timings: chunk.to_vec(),
                };
                let db = view
                    .to_data_block()
                    .map_err(|_| SerializeError::NoDtdSlot { index: 0 })?;
                let did_block = EdidBlock::from_display_id_data_blocks(0x20, 0, 0, &[db])
                    .map_err(|_| SerializeError::NoDtdSlot { index: 0 })?;
                blocks.push(did_block);
            }
        }
        ExtensionPolicy::PreferCta => {
            for chunk in overflow_dtds.chunks(6) {
                let cta_block = EdidBlock::from_cta_data_blocks_and_timings(3, &[], chunk)
                    .map_err(|_| SerializeError::NoDtdSlot { index: 0 })?;
                blocks.push(cta_block);
            }
        }
        ExtensionPolicy::PreferDisplayId => {
            let mut all_display_id: Vec<crate::extensions::DisplayIdDetailedTiming> = overflow_dtds
                .iter()
                .map(|t| t.to_display_id(false))
                .collect();
            all_display_id.extend(dtd_incompatible.iter().map(|t| t.to_display_id(false)));

            for chunk in all_display_id.chunks(5) {
                let view = crate::extensions::DisplayIdDataBlockView::DetailedTiming {
                    timings: chunk.to_vec(),
                };
                let db = view
                    .to_data_block()
                    .map_err(|_| SerializeError::NoDtdSlot { index: 0 })?;
                let did_block = EdidBlock::from_display_id_data_blocks(0x20, 0, 0, &[db])
                    .map_err(|_| SerializeError::NoDtdSlot { index: 0 })?;
                blocks.push(did_block);
            }
        }
        ExtensionPolicy::BaseOnly => unreachable!(),
    }

    let total_blocks = blocks.len();
    if total_blocks > options.max_blocks || total_blocks > crate::edid::MAX_EDID_BLOCKS {
        return Err(SerializeError::TooManyExtensions {
            count: total_blocks.saturating_sub(1),
        });
    }

    blocks[0].raw[126] = (blocks.len() - 1) as u8;
    for block in &mut blocks {
        block.update_checksum();
    }

    Ok(SerializedEdid {
        bytes: blocks.into_iter().flat_map(|block| block.raw).collect(),
        written: timings.len(),
        skipped: 0,
    })
}

fn timing_for_extended(
    index: usize,
    spec: &ResolutionSpec,
    allow_extensions: bool,
) -> Result<DetailedTiming, SerializeError> {
    if !spec.refresh.is_finite() {
        return Err(SerializeError::TimingUnavailable { index });
    }
    let timing = match spec.kind {
        TimingKind::Hdtv => {
            DetailedTiming::compute_hdtv_blanking(spec.width, spec.height, spec.refresh)
                .or_else(|| compute_cvt(spec.width, spec.height, spec.refresh, TimingFormula::CVT))
        }
        TimingKind::Pc => compute_cvt(spec.width, spec.height, spec.refresh, TimingFormula::CVTRB2),
    }
    .ok_or(SerializeError::TimingUnavailable { index })?;

    if !allow_extensions && !dtd_fits(&timing) {
        return Err(SerializeError::TimingDoesNotFit { index });
    }
    Ok(timing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edid::{DETAILED_START, EdidBlock};
    use crate::error::{EdidError, SerializeError};
    use crate::timing::all_presets;

    const DESCRIPTOR_LEN: usize = 18;

    fn spec(width: u32, height: u32, refresh: f64, kind: TimingKind) -> ResolutionSpec {
        ResolutionSpec {
            width,
            height,
            refresh,
            kind,
        }
    }
    fn checksum_ok(edid: &[u8]) {
        for block in edid.chunks(EDID_BLOCK_SIZE) {
            assert_eq!(block.len(), EDID_BLOCK_SIZE);
            let sum: u16 = block.iter().map(|&b| b as u16).sum();
            assert_eq!(sum % 256, 0, "block checksum invalid");
        }
    }

    /// Serializing with no existing EDID yields a fresh single-block EDID;
    /// free slots are usable and the result is a valid EDID.
    #[test]
    fn fresh_edid_writes_into_free_slots() {
        let out = serialize_resolutions(None, &[spec(1920, 1080, 60.0, TimingKind::Pc)]);
        assert_eq!(out.bytes.len(), EDID_BLOCK_SIZE);
        assert_eq!(out.written, 1);
        assert_eq!(out.skipped, 0);
        assert_eq!(out.bytes[126], 0); // no extension blocks
        let t = EdidBlock::from_bytes(&out.bytes)
            .unwrap()
            .read_detailed(0)
            .unwrap();
        assert_eq!((t.h_active, t.v_active), (1920, 1080));
        assert!(t.h_pol && !t.v_pol); // CVT-RB2 polarity
        checksum_ok(&out.bytes);
    }

    /// HDTV mode resolves through the CEA preset table (1080p60 DTD).
    #[test]
    fn hdtv_mode_uses_cea_preset() {
        let out = serialize_resolutions(None, &[spec(1920, 1080, 60.0, TimingKind::Hdtv)]);
        assert_eq!(out.written, 1);
        let t = EdidBlock::from_bytes(&out.bytes)
            .unwrap()
            .read_detailed(0)
            .unwrap();
        assert_eq!((t.h_front, t.h_sync, t.h_back), (88, 44, 148));
        assert!(t.h_pol && t.v_pol);
    }

    /// More resolutions than DTD slots: extras are counted as skipped.
    #[test]
    fn caps_at_four_dtd_slots() {
        let res: Vec<ResolutionSpec> = (0..5)
            .map(|i| spec(640 + 40 * i, 480 + 30 * i, 60.0, TimingKind::Pc))
            .collect();
        let out = serialize_resolutions(None, &res);
        assert_eq!(out.written, 4);
        assert_eq!(out.skipped, 1);
        assert_eq!(out.bytes.len(), EDID_BLOCK_SIZE);
        checksum_ok(&out.bytes);
    }

    /// Existing extension blocks survive serialization; their checksums are
    /// fixed and the extension count byte matches the blocks present.
    #[test]
    fn preserves_extension_blocks_and_checksums() {
        let mut base = EdidBlock::new_default();
        base.write_detailed(0, &all_presets()[0]);
        base.raw[126] = 1;
        base.update_checksum();

        // Stand-in for a CEA-861 extension block.
        let mut ext = EdidBlock::new_default();
        ext.raw[0] = 0x02; // CEA-861 tag
        ext.raw[1] = 3; // revision
        ext.update_checksum();
        ext.raw[127] = ext.raw[127].wrapping_add(1); // corrupt on purpose

        let mut existing = base.as_bytes().to_vec();
        existing.extend_from_slice(ext.as_bytes());

        let out = serialize_resolutions(Some(&existing), &[spec(1280, 720, 60.0, TimingKind::Pc)]);
        assert_eq!(out.bytes.len(), 2 * EDID_BLOCK_SIZE);
        assert_eq!(out.bytes[126], 1);
        assert_eq!(out.bytes[128], 0x02); // extension preserved
        assert_eq!(&out.bytes[129..255], &ext.as_bytes()[1..127]); // body untouched
        checksum_ok(&out.bytes);
    }

    /// The extension count byte must match the blocks actually present.
    #[test]
    fn normalizes_extension_count_byte() {
        let mut base = EdidBlock::new_default();
        base.raw[126] = 1; // claims an extension that is not provided
        base.update_checksum();
        let out = serialize_resolutions(
            Some(base.as_bytes()),
            &[spec(1024, 768, 60.0, TimingKind::Pc)],
        );
        assert_eq!(out.bytes.len(), EDID_BLOCK_SIZE);
        assert_eq!(out.bytes[126], 0);
        checksum_ok(&out.bytes);
    }

    /// Timings that cannot fit a DTD (12-bit active area) are skipped, not
    /// silently truncated.
    #[test]
    fn skips_oversized_timings() {
        let out = serialize_resolutions(None, &[spec(4096, 2160, 60.0, TimingKind::Pc)]);
        assert_eq!(out.written, 0);
        assert_eq!(out.skipped, 1);
    }

    /// Uncomputable timings are skipped too.
    #[test]
    fn skips_uncomputable_timings() {
        let out = serialize_resolutions(None, &[spec(0, 1080, 60.0, TimingKind::Pc)]);
        assert_eq!(out.written, 0);
        assert_eq!(out.skipped, 1);
    }

    /// Monitor descriptors in the existing EDID are untouched.
    #[test]
    fn preserves_monitor_descriptors() {
        let mut base = EdidBlock::new_default();
        base.write_detailed(0, &all_presets()[0]);
        let off = DETAILED_START + 2 * DESCRIPTOR_LEN;
        base.raw[off + 3] = 0xFC;
        base.raw[off + 5..off + 13].copy_from_slice(b"TESTMON\0");
        let before = base.raw[off..off + DESCRIPTOR_LEN].to_vec();
        base.update_checksum();

        let out = serialize_resolutions(
            Some(base.as_bytes()),
            &[
                spec(1920, 1080, 60.0, TimingKind::Pc),
                spec(1280, 720, 60.0, TimingKind::Pc),
            ],
        );
        assert_eq!(out.written, 2);
        assert_eq!(&out.bytes[off..off + DESCRIPTOR_LEN], &before[..]);
        checksum_ok(&out.bytes);
    }

    /// Existing data shorter than one block falls back to a fresh EDID.
    #[test]
    fn short_existing_data_falls_back_to_default() {
        let out =
            serialize_resolutions(Some(&[0u8; 100]), &[spec(1920, 1080, 60.0, TimingKind::Pc)]);
        assert_eq!(out.written, 1);
        assert_eq!(out.bytes.len(), EDID_BLOCK_SIZE);
        checksum_ok(&out.bytes);
    }

    #[test]
    fn compatibility_serializer_does_not_wrap_extension_count() {
        let mut existing = Vec::with_capacity(257 * EDID_BLOCK_SIZE);
        let mut base = EdidBlock::new_default();
        base.raw[126] = u8::MAX;
        base.update_checksum();
        existing.extend_from_slice(base.as_bytes());

        for index in 0..=u8::MAX {
            let mut extension = EdidBlock::new_default();
            extension.raw[0] = 0x02;
            extension.raw[1] = index;
            extension.update_checksum();
            existing.extend_from_slice(extension.as_bytes());
        }

        let out = serialize_resolutions(Some(&existing), &[]);
        assert_eq!(out.bytes.len(), 256 * EDID_BLOCK_SIZE);
        assert_eq!(out.bytes[126], u8::MAX);
        checksum_ok(&out.bytes);
    }

    #[test]
    fn strict_serializer_rejects_invalid_existing_length() {
        assert!(matches!(
            serialize_resolutions_checked(Some(&[0u8; 127]), &[]),
            Err(SerializeError::InvalidExistingLength { actual: 127 })
        ));
    }

    #[test]
    fn strict_serializer_reports_existing_block_errors() {
        let mut block = EdidBlock::new_default();
        block.raw[10] ^= 0x01;
        assert!(matches!(
            serialize_resolutions_checked(Some(block.as_bytes()), &[]),
            Err(SerializeError::InvalidExistingBlock {
                index: 0,
                source: EdidError::InvalidChecksum { .. }
            })
        ));
    }

    #[test]
    fn strict_serializer_reports_timing_and_slot_errors() {
        assert!(matches!(
            serialize_resolutions_checked(
                Some(EdidBlock::new_default().as_bytes()),
                &[spec(0, 1080, 60.0, TimingKind::Pc)]
            ),
            Err(SerializeError::TimingUnavailable { index: 0 })
        ));

        let resolutions = [spec(1920, 1080, 60.0, TimingKind::Pc); 5];
        assert!(matches!(
            serialize_resolutions_checked(None, &resolutions),
            Err(SerializeError::NoDtdSlot { index: 4 })
        ));
    }

    #[test]
    fn strict_serializer_writes_valid_output() {
        let out =
            serialize_resolutions_checked(None, &[spec(1920, 1080, 60.0, TimingKind::Pc)]).unwrap();
        assert_eq!(out.skipped, 0);
        checksum_ok(&out.bytes);
    }
    #[test]
    fn manual_timing_serializer_writes_detailed_timing() {
        let timing = all_presets()[0].clone();
        let out = serialize_timings(None, std::slice::from_ref(&timing)).unwrap();
        let read = EdidBlock::from_bytes(&out.bytes)
            .unwrap()
            .read_detailed(0)
            .unwrap();
        assert_eq!(read.h_active, timing.h_active);
        assert_eq!(read.v_active, timing.v_active);
        assert_eq!(out.written, 1);
        checksum_ok(&out.bytes);
    }

    #[test]
    fn manual_timing_serializer_reports_invalid_timing_index() {
        let mut timing = all_presets()[0].clone();
        timing.h_front = 1024;
        assert!(matches!(
            serialize_timings(None, &[all_presets()[0].clone(), timing]),
            Err(SerializeError::InvalidTiming { index: 1, .. })
        ));
    }
    #[test]
    fn hdtv_serializer_does_not_use_pc_wide_preset() {
        let out = serialize_resolutions_checked(None, &[spec(1280, 800, 60.0, TimingKind::Hdtv)])
            .unwrap();
        let timing = EdidBlock::from_bytes(&out.bytes)
            .unwrap()
            .read_detailed(0)
            .unwrap();
        assert_ne!(timing.h_sync, 32);
    }
    #[test]
    fn manual_timing_serializer_rejects_low_pixel_clock() {
        let mut timing = all_presets()[0].clone();
        timing.pixel_clock_khz = 9;
        assert!(matches!(
            serialize_timings(None, std::slice::from_ref(&timing)),
            Err(SerializeError::InvalidTiming { index: 0, .. })
        ));
    }

    #[test]
    fn serialize_rejects_oversized_existing_edid_blocks() {
        let oversized = vec![0u8; crate::edid::MAX_EDID_BYTES + EDID_BLOCK_SIZE];
        assert!(matches!(
            serialize_resolutions_checked(Some(&oversized), &[]),
            Err(SerializeError::InvalidExistingLength { actual }) if actual == crate::edid::MAX_EDID_BYTES + EDID_BLOCK_SIZE
        ));
    }

    #[test]
    fn serialize_extended_base_only_rejects_overflow() {
        let specs = vec![
            spec(1920, 1080, 60.0, TimingKind::Hdtv),
            spec(1280, 720, 60.0, TimingKind::Hdtv),
            spec(800, 600, 60.0, TimingKind::Pc),
            spec(640, 480, 60.0, TimingKind::Pc),
            spec(1024, 768, 60.0, TimingKind::Pc),
        ];
        let opts = SerializeOptions {
            extension_policy: ExtensionPolicy::BaseOnly,
            ..Default::default()
        };
        let res = serialize_resolutions_extended(None, &specs, &opts);
        assert!(matches!(res, Err(SerializeError::NoDtdSlot { index: 4 })));
    }

    #[test]
    fn serialize_extended_auto_overflows_to_cta_extension() {
        // 6 standard DTD resolutions: 4 in Base Block, 2 in CTA-861 Extension Block
        let specs = vec![
            spec(1920, 1080, 60.0, TimingKind::Hdtv),
            spec(1280, 720, 60.0, TimingKind::Hdtv),
            spec(800, 600, 60.0, TimingKind::Pc),
            spec(640, 480, 60.0, TimingKind::Pc),
            spec(1024, 768, 60.0, TimingKind::Pc),
            spec(1600, 900, 60.0, TimingKind::Pc),
        ];
        let opts = SerializeOptions::default();
        let out = serialize_resolutions_extended(None, &specs, &opts).unwrap();
        assert_eq!(out.written, 6);
        assert_eq!(out.bytes.len(), 2 * EDID_BLOCK_SIZE);
        checksum_ok(&out.bytes);

        // Base block extension count must be 1
        assert_eq!(out.bytes[126], 1);

        // Second block must be CTA-861 (tag 0x02)
        assert_eq!(out.bytes[128], 0x02);

        // Parse the CTA block and verify the 2 overflow DTDs
        let cta_block = EdidBlock::from_bytes(&out.bytes[128..256]).unwrap();
        let dtds = cta_block.cta_detailed_timings().unwrap();
        assert_eq!(dtds.len(), 2);
        assert_eq!(dtds[0].h_active, 1024);
        assert_eq!(dtds[0].v_active, 768);
        assert_eq!(dtds[1].h_active, 1600);
        assert_eq!(dtds[1].v_active, 900);
    }

    #[test]
    fn serialize_extended_auto_routes_high_refresh_cvtrb2_to_displayid() {
        // 2560x1440@144Hz CVT-RB2 produces v_front = 89, which exceeds DTD limit (v_front <= 63)
        let specs = vec![
            spec(1920, 1080, 60.0, TimingKind::Hdtv),
            spec(2560, 1440, 144.0, TimingKind::Pc),
        ];
        let opts = SerializeOptions::default();
        let out = serialize_resolutions_extended(None, &specs, &opts).unwrap();
        assert_eq!(out.written, 2);
        assert_eq!(out.bytes.len(), 2 * EDID_BLOCK_SIZE);
        checksum_ok(&out.bytes);

        // Base block should have the 1080p timing
        let base_block = EdidBlock::from_bytes(&out.bytes[0..128]).unwrap();
        assert_eq!(base_block.raw[126], 1);
        let base_dtd = base_block.read_detailed(0).unwrap();
        assert_eq!(base_dtd.h_active, 1920);
        assert_eq!(base_dtd.v_active, 1080);

        // Second block should be DisplayID (tag 0x70)
        assert_eq!(out.bytes[128], 0x70);
        let did_block = EdidBlock::from_bytes(&out.bytes[128..256]).unwrap();
        let did_blocks = did_block.display_id_data_blocks().unwrap();
        assert_eq!(did_blocks.len(), 1);

        match did_blocks[0].view().unwrap() {
            crate::extensions::DisplayIdDataBlockView::DetailedTiming { timings } => {
                assert_eq!(timings.len(), 1);
                assert_eq!(timings[0].h_active, 2560);
                assert_eq!(timings[0].v_active, 1440);
                assert_eq!(timings[0].v_sync_offset, 89);
            }
            other => panic!("expected DetailedTiming block, got {:?}", other),
        }
    }

    #[test]
    fn serialize_extended_splits_multiple_cta_blocks() {
        // 16 standard DTDs: 4 Base + 6 in CTA #1 + 6 in CTA #2 = 3 blocks total
        let mut timings = Vec::new();
        for i in 0..16 {
            let width = 640 + i * 32;
            let timing = compute_cvt(width, 480, 60.0, TimingFormula::CVT).unwrap();
            timings.push(timing);
        }

        let opts = SerializeOptions {
            extension_policy: ExtensionPolicy::PreferCta,
            ..Default::default()
        };
        let out = serialize_timings_extended(None, &timings, &opts).unwrap();
        assert_eq!(out.written, 16);
        assert_eq!(out.bytes.len(), 3 * EDID_BLOCK_SIZE);
        checksum_ok(&out.bytes);

        // Base block extension count must be 2
        assert_eq!(out.bytes[126], 2);
        assert_eq!(out.bytes[128], 0x02);
        assert_eq!(out.bytes[256], 0x02);

        let cta1 = EdidBlock::from_bytes(&out.bytes[128..256]).unwrap();
        let cta2 = EdidBlock::from_bytes(&out.bytes[256..384]).unwrap();
        assert_eq!(cta1.cta_detailed_timings().unwrap().len(), 6);
        assert_eq!(cta2.cta_detailed_timings().unwrap().len(), 6);
    }

    #[test]
    fn serialize_extended_splits_multiple_displayid_blocks() {
        // 12 DisplayID timings in PreferDisplayId mode:
        // 4 in Base (since they are DTD-compatible) + 5 in DisplayID #1 + 3 in DisplayID #2 = 3 blocks
        let mut timings = Vec::new();
        for i in 0..12 {
            let width = 640 + i * 32;
            let timing = compute_cvt(width, 480, 60.0, TimingFormula::CVT).unwrap();
            timings.push(timing);
        }

        let opts = SerializeOptions {
            extension_policy: ExtensionPolicy::PreferDisplayId,
            ..Default::default()
        };
        let out = serialize_timings_extended(None, &timings, &opts).unwrap();
        assert_eq!(out.written, 12);
        assert_eq!(out.bytes.len(), 3 * EDID_BLOCK_SIZE);
        checksum_ok(&out.bytes);

        assert_eq!(out.bytes[126], 2);
        assert_eq!(out.bytes[128], 0x70);
        assert_eq!(out.bytes[256], 0x70);
    }

    #[test]
    fn serialize_extended_prefer_cta_rejects_incompatible_timings() {
        let specs = vec![spec(2560, 1440, 144.0, TimingKind::Pc)];
        let opts = SerializeOptions {
            extension_policy: ExtensionPolicy::PreferCta,
            ..Default::default()
        };
        let res = serialize_resolutions_extended(None, &specs, &opts);
        assert!(matches!(
            res,
            Err(SerializeError::TimingDoesNotFit { index: 0 })
        ));
    }

    #[test]
    fn serialize_extended_respects_max_blocks_limit() {
        let mut timings = Vec::new();
        for i in 0..16 {
            let width = 640 + i * 32;
            let timing = compute_cvt(width, 480, 60.0, TimingFormula::CVT).unwrap();
            timings.push(timing);
        }
        let opts = SerializeOptions {
            extension_policy: ExtensionPolicy::Auto,
            max_blocks: 2, // Only allow 2 blocks (Base + 1 Extension), but 16 timings need 3 blocks
            ..Default::default()
        };
        let res = serialize_timings_extended(None, &timings, &opts);
        assert!(matches!(
            res,
            Err(SerializeError::TooManyExtensions { count: 2 })
        ));
    }
}
