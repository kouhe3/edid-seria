// EDID detailed resolution: binary read/write (18-byte block)
//
// EDID detailed timing descriptor (bytes 0-17):
//   [0-1]   Pixel clock (kHz/10, LE)
//   [2]     HActive[7:0]
//   [3]     HBlank[7:0]
//   [4]     HActive[11:8] << 4 | HBlank[11:8]
//   [5]     VActive[7:0]
//   [6]     VBlank[7:0]
//   [7]     VActive[11:8] << 4 | VBlank[11:8]
//   [8]     HFront[7:0]
//   [9]     HSync[7:0]
//   [10]    VFront[3:0] << 4 | VSync[3:0]
//   [11]    HFront[9:8] << 6 | HSync[9:8] << 4 | VFront[5:4] << 2 | VSync[5:4]
//   [12]    HSize[7:0]   (= HActive / 4)
//   [13]    VSize[7:0]   (= VActive / 4)
//   [14]    HSize[11:8] << 4 | VSize[11:8]
//   [15]    HBorder
//   [16]    VBorder
//   [17]    Flags: Interlaced|Stereo|Sync Type|Serrate/SyncOnGreen|VPol|HPol
//
// Note on byte 17 (E-EDID 1.4 §3.10.2): bits 1/2 are H/V polarity only for
// digital separate sync (bits 4-3 = 11). Digital composite encodes H polarity
// in bit 1 (V is always negative); analog composite/bipolar use bits 1/2 for
// serrate/sync-on-green and encode neither polarity. HBlank/VBlank include
// 2× the border values (bytes 15/16).

use crate::timing::DetailedTiming;

const EDID_DESCRIPTOR_LEN: usize = 18;
pub const EDID_BLOCK_SIZE: usize = 128;
const DETAILED_SLOTS: usize = 4;
pub(crate) const DETAILED_START: usize = 54;
/// Classification of a detailed-timing descriptor slot.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum SlotKind {
    /// Pixel clock != 0: holds a detailed timing (progressive or interlaced).
    Timing,
    /// Pixel clock == 0, no descriptor payload: reusable dummy slot.
    Free,
    /// Pixel clock == 0 with a descriptor tag/payload: monitor descriptor
    /// (name, range limits, serial, ...) — must be preserved.
    Descriptor,
}

pub struct EdidBlock {
    pub raw: [u8; EDID_BLOCK_SIZE],
}

impl EdidBlock {
    /// Create default minimal EDID block
    pub fn new_default() -> Self {
        let mut raw = [0u8; EDID_BLOCK_SIZE];
        // EDID header: 00 FF FF FF FF FF FF 00
        raw[..8].copy_from_slice(&[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]);
        // Version 1.4
        raw[18] = 0x01;
        raw[19] = 0x04;
        // Digital display (bit 7); interface/color-depth undefined — the
        // typical starting point for a modern PC-monitor override.
        raw[20] = 0x80;
        // Extension count = 0
        raw[126] = 0x00;
        Self { raw }
    }

    /// Parse EDID from bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < EDID_BLOCK_SIZE {
            return None;
        }
        let mut raw = [0u8; EDID_BLOCK_SIZE];
        raw.copy_from_slice(&data[..EDID_BLOCK_SIZE]);
        Some(Self { raw })
    }

    /// Read detailed timing from slot (0-3)
    pub fn read_detailed(&self, slot: usize) -> Option<DetailedTiming> {
        if slot >= DETAILED_SLOTS {
            return None;
        }
        let off = DETAILED_START + slot * EDID_DESCRIPTOR_LEN;
        let d = &self.raw[off..off + EDID_DESCRIPTOR_LEN];

        // All-0x01 is a known padding pattern for unused slots, not a timing.
        if d.iter().all(|&b| b == 0x01) {
            return None;
        }
        // Check if this is a monitor descriptor (not a timing)
        // Monitor descriptors have pixel clock == 0 for tag-based ones
        let pixel_clock = u16::from_le_bytes([d[0], d[1]]);
        if pixel_clock == 0 {
            return None; // monitor descriptor, not a timing
        }
        // Clocks below 10 MHz are invalid data, not timings (same heuristic
        // as edid-decode: padding/junk descriptors often carry tiny clocks).
        if pixel_clock < 1000 {
            return None;
        }
        if (d[17] & 0x80) != 0 {
            return None; // interlaced timings not supported
        }
        let h_border = d[15] as u16;
        let v_border = d[16] as u16;

        let h_active = d[2] as u16 | ((d[4] as u16 & 0xF0) << 4);
        let h_blank = (d[3] as u16 | ((d[4] as u16 & 0x0F) << 8)).saturating_sub(h_border * 2);
        let v_active = d[5] as u16 | ((d[7] as u16 & 0xF0) << 4);
        let v_blank = (d[6] as u16 | ((d[7] as u16 & 0x0F) << 8)).saturating_sub(v_border * 2);
        let h_front = d[8] as u16 | ((d[11] as u16 & 0xC0) << 2);
        let h_sync = d[9] as u16 | ((d[11] as u16 & 0x30) << 4);
        let v_front = (d[10] as u16 >> 4) | ((d[11] as u16 & 0x0C) << 2);
        let v_sync = (d[10] as u16 & 0x0F) | ((d[11] as u16 & 0x03) << 4);

        let h_back = h_blank.saturating_sub(h_front + h_sync);
        let v_back = v_blank.saturating_sub(v_front + v_sync);

        // Byte 17 polarity bits mean different things per sync type
        // (E-EDID 1.4 §3.10.2). Only digital separate sync encodes H/V
        // polarity in bits 1/2; digital composite encodes H polarity only
        // (V is always negative); analog composite/bipolar use those bits
        // for serrate/sync-on-green and encode neither polarity.
        let sync_type = (d[17] >> 3) & 0x03;
        let h_pol = sync_type >= 0x02 && (d[17] & 0x02) != 0;
        let v_pol = sync_type == 0x03 && (d[17] & 0x04) != 0;

        let h_total = h_active + h_blank;
        let v_total = v_active + v_blank;
        let v_rate = (pixel_clock as f64 * 10_000.0) / (h_total as f64 * v_total as f64);

        Some(DetailedTiming {
            h_active: h_active as u32,
            v_active: v_active as u32,
            h_front: h_front as u32,
            h_sync: h_sync as u32,
            h_back: h_back as u32,
            v_front: v_front as u32,
            v_sync: v_sync as u32,
            v_back: v_back as u32,
            h_border: h_border as u32,
            v_border: v_border as u32,
            pixel_clock_khz: pixel_clock as u32 * 10,
            h_pol,
            v_pol,
            v_rate,
        })
    }

    /// Write detailed timing to slot
    pub fn write_detailed(&mut self, slot: usize, t: &DetailedTiming) {
        if slot >= DETAILED_SLOTS {
            return;
        }
        let off = DETAILED_START + slot * EDID_DESCRIPTOR_LEN;
        let d = &mut self.raw[off..off + EDID_DESCRIPTOR_LEN];

        // DTD stores 10 kHz units; round to nearest instead of truncating.
        let pixel_clock = ((t.pixel_clock_khz + 5) / 10) as u16;
        let h_active = t.h_active as u16;
        let v_active = t.v_active as u16;
        let h_blank = (t.h_front + t.h_sync + t.h_back + 2 * t.h_border) as u16;
        let v_blank = (t.v_front + t.v_sync + t.v_back + 2 * t.v_border) as u16;

        d[0] = pixel_clock as u8;
        d[1] = (pixel_clock >> 8) as u8;
        d[2] = h_active as u8;
        d[3] = h_blank as u8;
        d[4] = ((h_active >> 4) & 0xF0) as u8 | ((h_blank >> 8) & 0x0F) as u8;
        d[5] = v_active as u8;
        d[6] = v_blank as u8;
        d[7] = ((v_active >> 4) & 0xF0) as u8 | ((v_blank >> 8) & 0x0F) as u8;
        d[8] = t.h_front as u8;
        d[9] = t.h_sync as u8;
        d[10] = ((t.v_front as u8 & 0x0F) << 4) | (t.v_sync as u8 & 0x0F);
        d[11] = ((t.h_front as u16 >> 8) as u8 & 0x03) << 6
            | ((t.h_sync as u16 >> 8) as u8 & 0x03) << 4
            | ((t.v_front as u16 >> 4) as u8 & 0x03) << 2
            | ((t.v_sync as u16 >> 4) as u8 & 0x03);

        // HSize / VSize (display physical size in mm, set to HActive/4 for 4px/mm)
        let hsize = h_active / 4;
        let vsize = v_active / 4;
        d[12] = hsize as u8;
        d[13] = vsize as u8;
        d[14] = (((hsize >> 8) & 0x0F) as u8) << 4 | (((vsize >> 8) & 0x0F) as u8);
        d[15] = t.h_border as u8;
        d[16] = t.v_border as u8;

        let mut flags = 0x18u8; // digital separate sync
        if t.h_pol {
            flags |= 0x02;
        }
        if t.v_pol {
            flags |= 0x04;
        }
        d[17] = flags;
    }

    /// Clear a detailed timing slot to CRU's dummy descriptor convention:
    /// pixel clock = 0, tag byte 0x10, payload zeroed.
    pub fn clear_slot(&mut self, slot: usize) {
        if slot >= DETAILED_SLOTS {
            return;
        }
        let off = DETAILED_START + slot * EDID_DESCRIPTOR_LEN;
        self.raw[off..off + EDID_DESCRIPTOR_LEN].fill(0);
        self.raw[off + 3] = 0x10;
    }

    /// Classify a detailed-timing descriptor slot.
    fn slot_kind(&self, slot: usize) -> SlotKind {
        let off = DETAILED_START + slot * EDID_DESCRIPTOR_LEN;
        let d = &self.raw[off..off + EDID_DESCRIPTOR_LEN];
        // All-0x01 is a known padding pattern for unused slots (seen in real
        // EDIDs); treat it like an all-zero slot, not a timing.
        if d.iter().all(|&b| b == 0x01) {
            return SlotKind::Free;
        }
        // Pixel clock != 0 means a detailed timing, regardless of whether it
        // is parseable — interlaced timings count as timing slots too.
        if d[0] != 0 || d[1] != 0 {
            return SlotKind::Timing;
        }
        // Dummy descriptors (tag 0x10, zero payload — CRU's clear convention)
        // and all-zero slots are reusable. Anything else with a descriptor
        // tag or payload is a monitor descriptor that must be preserved.
        if d.iter().all(|&b| b == 0) || (d[3] == 0x10 && d[4..].iter().all(|&b| b == 0)) {
            SlotKind::Free
        } else {
            SlotKind::Descriptor
        }
    }

    /// Replace the timings in the base block, preserving monitor descriptors.
    ///
    /// Timings are written into existing timing slots first (so replacements
    /// keep their positions), then into free (dummy/empty) slots. Monitor
    /// descriptors (name, range limits, ...) are left untouched, and timing
    /// slots left over after the write are cleared to dummy descriptors.
    /// Returns the number of timings written.
    pub fn write_resolutions(&mut self, timings: &[DetailedTiming]) -> usize {
        let kinds: Vec<SlotKind> = (0..DETAILED_SLOTS).map(|s| self.slot_kind(s)).collect();
        let mut slots: Vec<usize> = kinds
            .iter()
            .enumerate()
            .filter(|&(_, k)| *k != SlotKind::Descriptor)
            .map(|(i, _)| i)
            .collect();
        slots.sort_by_key(|&s| if kinds[s] == SlotKind::Timing { 0 } else { 1 });

        let mut written = 0;
        for (i, t) in timings.iter().enumerate() {
            if let Some(&s) = slots.get(i) {
                self.write_detailed(s, t);
                written += 1;
            }
        }
        for &s in &slots[written..] {
            if kinds[s] == SlotKind::Timing {
                self.clear_slot(s);
            }
        }
        written
    }

    /// Get all detailed timings from this block
    pub fn detailed_timings(&self) -> Vec<DetailedTiming> {
        (0..DETAILED_SLOTS)
            .filter_map(|slot| self.read_detailed(slot))
            .collect()
    }

    /// Update checksum (byte 127)
    pub fn update_checksum(&mut self) {
        let sum: u8 = self.raw[..127].iter().fold(0u8, |a, &b| a.wrapping_add(b));
        self.raw[127] = (256u16 - sum as u16) as u8;
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timing::DetailedTiming;

    fn make_timing() -> DetailedTiming {
        DetailedTiming {
            h_active: 1920,
            v_active: 1080,
            h_front: 88,
            h_sync: 44,
            h_back: 148,
            v_front: 4,
            v_sync: 5,
            v_back: 36,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 148500,
            h_pol: true,
            v_pol: true,
            v_rate: 60.0,
        }
    }

    #[test]
    fn roundtrip_detailed_timing() {
        let t = make_timing();
        let mut block = EdidBlock::new_default();
        block.write_detailed(0, &t);

        let read = block.read_detailed(0).expect("should read back");
        assert_eq!(read.h_active, 1920);
        assert_eq!(read.v_active, 1080);
        assert_eq!(read.h_front, 88);
        assert_eq!(read.h_sync, 44);
        assert_eq!(read.h_back, 148);
        assert!(read.h_pol);
        assert!(read.v_pol);
        assert!((read.v_rate - 60.0).abs() < 1.0);
    }

    /// Golden bytes: the canonical CEA-861 1920x1080@60 DTD.
    #[test]
    fn writes_known_1080p60_dtd_bytes() {
        let mut block = EdidBlock::new_default();
        block.write_detailed(0, &make_timing());

        let off = DETAILED_START;
        let d = &block.raw[off..off + EDID_DESCRIPTOR_LEN];
        assert_eq!(
            d,
            &[
                0x02, 0x3A, 0x80, 0x18, 0x71, 0x38, 0x2D, 0x40, 0x58, 0x2C, 0x45, 0x00, 0xE0, 0x0E,
                0x11, 0x00, 0x00, 0x1E,
            ]
        );
    }

    /// HSize/VSize are 12-bit fields; the old hardcoded 0x10 broke >= 1024px.
    #[test]
    fn writes_correct_physical_size_for_4k() {
        let mut block = EdidBlock::new_default();
        let t = DetailedTiming {
            h_active: 3840,
            v_active: 2160,
            h_front: 176,
            h_sync: 88,
            h_back: 296,
            v_front: 8,
            v_sync: 10,
            v_back: 72,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 297000,
            h_pol: true,
            v_pol: true,
            v_rate: 60.0,
        };
        block.write_detailed(0, &t);

        let off = DETAILED_START;
        let d = &block.raw[off..off + EDID_DESCRIPTOR_LEN];
        assert_eq!(d[12], 0xC0); // 960 & 0xFF
        assert_eq!(d[13], 0x1C); // 540 & 0xFF
        assert_eq!(d[14], 0x32); // HSize[11:8]=3, VSize[11:8]=2
    }

    /// Interlaced DTDs are skipped, not misread as progressive timings.
    #[test]
    fn skips_interlaced_timing() {
        let mut block = EdidBlock::new_default();
        block.write_detailed(0, &make_timing());
        let off = DETAILED_START;
        block.raw[off + 17] |= 0x80; // set interlaced flag
        assert!(block.read_detailed(0).is_none());
        assert_eq!(block.slot_kind(0), SlotKind::Timing); // still a timing slot for replacement
    }

    /// Borders are part of the DTD: HBlank/VBlank include 2× the border,
    /// and bytes 15/16 must round-trip.
    #[test]
    fn roundtrips_border_fields() {
        let mut t = make_timing();
        t.h_border = 8;
        t.v_border = 4;
        let mut block = EdidBlock::new_default();
        block.write_detailed(0, &t);

        let off = DETAILED_START;
        let d = &block.raw[off..off + EDID_DESCRIPTOR_LEN];
        // HBlank byte = front+sync+back + 2*border = 280 + 16 = 296 = 0x128
        assert_eq!(d[3], 0x28);
        assert_eq!(d[4] & 0x0F, 0x01);
        // VBlank byte = 4+5+36 + 8 = 53 = 0x35
        assert_eq!(d[6], 0x35);
        assert_eq!(d[15], 8);
        assert_eq!(d[16], 4);

        let read = block.read_detailed(0).unwrap();
        assert_eq!(read.h_border, 8);
        assert_eq!(read.v_border, 4);
        assert_eq!(read.h_back, 148);
        assert_eq!(read.v_back, 36);
    }

    /// Byte 17 polarity bits: only digital separate sync encodes H/V
    /// polarity in bits 1/2. Analog composite uses them for
    /// serrate/sync-on-green, so both polarities read as negative.
    #[test]
    fn polarity_follows_sync_type() {
        let mut block = EdidBlock::new_default();
        block.write_detailed(0, &make_timing()); // writes digital separate +H+V
        assert_eq!(block.raw[DETAILED_START + 17], 0x1E);
        assert!(block.read_detailed(0).unwrap().h_pol);
        assert!(block.read_detailed(0).unwrap().v_pol);

        // Analog composite with serrate (bit 2) + sync-on-green (bit 1):
        // those bits are NOT polarities.
        block.raw[DETAILED_START + 17] = 0x06;
        let t = block.read_detailed(0).unwrap();
        assert!(!t.h_pol);
        assert!(!t.v_pol);

        // Digital composite: bit 1 = H polarity, V is always negative.
        block.raw[DETAILED_START + 17] = 0x12; // sync type 10, +H
        let t = block.read_detailed(0).unwrap();
        assert!(t.h_pol);
        assert!(!t.v_pol);
    }

    /// Clocks below 10 MHz are padding/junk, not timings (edid-decode
    /// heuristic); such slots still count as timing slots for replacement.
    #[test]
    fn sub_10mhz_clock_is_not_a_timing() {
        let mut block = EdidBlock::new_default();
        block.write_detailed(0, &make_timing());
        let off = DETAILED_START;
        block.raw[off] = 0xDF; // 0x00DF = 2.23 MHz
        block.raw[off + 1] = 0x00;
        assert!(block.read_detailed(0).is_none());
        assert_eq!(block.slot_kind(0), SlotKind::Timing);
    }

    /// Pixel clock is stored in 10 kHz units and rounded, not truncated.
    #[test]
    fn pixel_clock_rounds_to_10khz() {
        let mut t = make_timing();
        t.pixel_clock_khz = 133_187;
        let mut block = EdidBlock::new_default();
        block.write_detailed(0, &t);
        let off = DETAILED_START;
        let stored = u16::from_le_bytes([block.raw[off], block.raw[off + 1]]);
        assert_eq!(stored, 13319); // 13318.7 rounded up, not truncated
    }

    /// All-0x01 padding slots are free slots, not timings.
    #[test]
    fn zero_one_padding_is_free() {
        let mut block = EdidBlock::new_default();
        let off = DETAILED_START;
        block.raw[off..off + EDID_DESCRIPTOR_LEN].fill(0x01);
        assert_eq!(block.slot_kind(0), SlotKind::Free);
        assert!(block.read_detailed(0).is_none());
        // write_resolutions can use the slot.
        assert_eq!(block.write_resolutions(&[make_timing()]), 1);
        assert_eq!(block.slot_kind(0), SlotKind::Timing);
    }

    /// Replacing timings must not clobber monitor descriptors (name/range).
    #[test]
    fn write_resolutions_preserves_monitor_descriptors() {
        let mut block = EdidBlock::new_default();
        block.write_detailed(0, &make_timing());
        block.write_detailed(1, &make_timing());

        // Slot 2: display name descriptor (pixel clock 0, tag 0xFC).
        let off = DETAILED_START + 2 * EDID_DESCRIPTOR_LEN;
        block.raw[off + 3] = 0xFC;
        block.raw[off + 5..off + 13].copy_from_slice(b"TESTMON\0");
        let name_before = block.raw[off..off + EDID_DESCRIPTOR_LEN].to_vec();

        // Two original timing slots, one new resolution → one overwrite, one clear.
        let written = block.write_resolutions(&[make_timing()]);
        assert_eq!(written, 1);
        assert_eq!(block.raw[off..off + EDID_DESCRIPTOR_LEN], name_before);
        assert_eq!(
            block.read_detailed(0).expect("slot 0 rewritten").h_active,
            1920
        );
        assert!(block.read_detailed(1).is_none()); // leftover timing slot cleared
    }

    /// Timings fill existing timing slots first, then free slots; anything
    /// beyond the four DTD slots is dropped.
    #[test]
    fn write_resolutions_uses_free_slots_and_caps() {
        let mut block = EdidBlock::new_default();
        block.write_detailed(0, &make_timing());
        block.write_detailed(1, &make_timing());
        // Slots 2-3 are free: four timings fit, the fifth does not.
        let five: Vec<DetailedTiming> = (0..5).map(|_| make_timing()).collect();
        assert_eq!(block.write_resolutions(&five), 4);
    }

    /// A default block has no timing slots; free slots must still be usable.
    #[test]
    fn write_resolutions_fills_fresh_block() {
        let mut block = EdidBlock::new_default();
        assert_eq!(block.write_resolutions(&[make_timing()]), 1);
        assert_eq!(block.read_detailed(0).expect("slot 0").h_active, 1920);
    }

    /// Cleared slots use CRU's dummy descriptor convention (tag 0x10).
    #[test]
    fn clear_slot_writes_cru_dummy_descriptor() {
        let mut block = EdidBlock::new_default();
        block.write_detailed(0, &make_timing());
        block.clear_slot(0);
        let off = DETAILED_START;
        assert_eq!(&block.raw[off..off + 2], &[0, 0]);
        assert_eq!(block.raw[off + 3], 0x10);
        assert!(
            block.raw[off + 4..off + EDID_DESCRIPTOR_LEN]
                .iter()
                .all(|&b| b == 0)
        );
        assert_eq!(block.slot_kind(0), SlotKind::Free);
    }

    /// Replacing fewer timings than slots exist clears the leftover slots
    /// but leaves monitor descriptors and free slots untouched.
    #[test]
    fn write_resolutions_clears_only_leftover_timing_slots() {
        let mut block = EdidBlock::new_default();
        block.write_detailed(0, &make_timing());
        block.write_detailed(1, &make_timing());
        // Slot 3 free, slot 2 monitor descriptor.
        let name_off = DETAILED_START + 2 * EDID_DESCRIPTOR_LEN;
        block.raw[name_off + 3] = 0xFC;
        block.raw[name_off + 5..name_off + 13].copy_from_slice(b"TESTMON\0");
        let name_before = block.raw[name_off..name_off + EDID_DESCRIPTOR_LEN].to_vec();
        let free_before = block.raw
            [DETAILED_START + 3 * EDID_DESCRIPTOR_LEN..DETAILED_START + 4 * EDID_DESCRIPTOR_LEN]
            .to_vec();

        let written = block.write_resolutions(&[make_timing()]);
        assert_eq!(written, 1);
        assert_eq!(block.slot_kind(0), SlotKind::Timing);
        assert_eq!(block.slot_kind(1), SlotKind::Free); // leftover timing cleared
        assert_eq!(
            block.raw[name_off..name_off + EDID_DESCRIPTOR_LEN],
            name_before
        );
        assert_eq!(
            block.raw[DETAILED_START + 3 * EDID_DESCRIPTOR_LEN
                ..DETAILED_START + 4 * EDID_DESCRIPTOR_LEN],
            free_before
        );
    }

    #[test]
    fn checksum_is_valid() {
        let mut block = EdidBlock::new_default();
        block.update_checksum();
        let sum: u16 = block.as_bytes()[..128].iter().map(|&b| b as u16).sum();
        assert_eq!(sum % 256, 0);
    }

    /// The default block is digital, not analog.
    #[test]
    fn default_block_is_digital() {
        let block = EdidBlock::new_default();
        assert_ne!(block.raw[20] & 0x80, 0);
    }
}
