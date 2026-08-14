//! Resolution → EDID serialization pipeline.
//!
//! High-level counterpart to the low-level [`edid`](crate::edid) module:
//! takes user-facing resolution specs, computes DTDs (CVT-RB2 for PC
//! monitors, CEA/HDTV presets for TVs), rewrites the base block's
//! detailed-timing slots while preserving monitor descriptors, fixes the
//! checksum of every block, and returns the complete EDID binary.

use crate::edid::{EDID_BLOCK_SIZE, EdidBlock};
use crate::timing::{DetailedTiming, TimingFormula, compute_cvt, dtd_fits};

/// PC vs HDTV timing style. Mirrors the GUI's mode selector.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TimingKind {
    /// CVT 1.2 reduced blanking (recommended for PC monitors).
    Pc,
    /// Standard CEA/HDTV blanking from the preset table.
    Hdtv,
}

/// A user-requested resolution, before timing computation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ResolutionSpec {
    pub width: u32,
    pub height: u32,
    pub refresh: f64,
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
/// block. A trailing partial block (length not a multiple of 128) is dropped.
/// The base block's detailed-timing slots are rewritten, monitor descriptors
/// are preserved, every block's checksum is fixed, and the extension count
/// byte is normalized to the number of blocks actually present.
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

/// Compute the DTD for a resolution spec; `None` when the timing cannot be
/// computed or does not fit in an EDID DTD.
fn timing_for(spec: &ResolutionSpec) -> Option<DetailedTiming> {
    let t = match spec.kind {
        TimingKind::Hdtv => DetailedTiming::compute_blanking(spec.width, spec.height, spec.refresh)
            .or_else(|| compute_cvt(spec.width, spec.height, spec.refresh, TimingFormula::CVT)),
        TimingKind::Pc => compute_cvt(spec.width, spec.height, spec.refresh, TimingFormula::CVTRB2),
    }?;
    dtd_fits(&t).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edid::{DETAILED_START, EdidBlock};
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
}
