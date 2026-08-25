//! EDID base-block read/write: the 18-byte detailed timing descriptor
//! (DTD), slot classification, and checksum handling.
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

use crate::error::EdidError;
use crate::timing::DetailedTiming;

const EDID_DESCRIPTOR_LEN: usize = 18;
const EDID_HEADER: [u8; 8] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];
/// Size of one EDID block in bytes.
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

/// One raw EDID block (base block or extension block).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EdidBlock {
    /// The raw 128 EDID bytes.
    pub raw: [u8; EDID_BLOCK_SIZE],
}

/// Raw flags from DTD byte 17.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DtdFlags {
    /// Original EDID byte 17, including interlace, stereo and sync fields.
    pub raw: u8,
}

impl DtdFlags {
    /// Whether the timing is interlaced.
    #[must_use]
    pub const fn interlaced(self) -> bool {
        self.raw & 0x80 != 0
    }
}

/// A decoded DTD with its timing and raw flag semantics.
#[derive(Clone, Debug, PartialEq)]
pub struct DecodedDtd {
    /// Decoded timing fields.
    pub timing: DetailedTiming,
    /// Original DTD flags.
    pub flags: DtdFlags,
}

impl EdidBlock {
    /// Create default minimal EDID block
    #[must_use]
    pub fn new_default() -> Self {
        let mut raw = [0u8; EDID_BLOCK_SIZE];
        // EDID header: 00 FF FF FF FF FF FF 00
        raw[..8].copy_from_slice(&EDID_HEADER);
        // Version 1.4
        raw[18] = 0x01;
        raw[19] = 0x04;
        // Digital display (bit 7); interface/color-depth undefined — the
        // typical starting point for a modern PC-monitor override.
        raw[20] = 0x80;
        // Unused standard timing entries use 01 01.
        raw[38..54].fill(0x01);
        // Unused descriptor slots use the dummy descriptor tag 0x10.
        for slot in 0..DETAILED_SLOTS {
            let offset = DETAILED_START + slot * EDID_DESCRIPTOR_LEN;
            raw[offset + 3] = 0x10;
        }
        // Extension count = 0
        raw[126] = 0x00;
        let mut block = Self { raw };
        block.update_checksum();
        block
    }

    /// Parse EDID from bytes
    #[must_use]
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < EDID_BLOCK_SIZE {
            return None;
        }
        let mut raw = [0u8; EDID_BLOCK_SIZE];
        raw.copy_from_slice(&data[..EDID_BLOCK_SIZE]);
        Some(Self { raw })
    }
    /// Parse and validate exactly one EDID block.
    ///
    /// Base blocks must contain the EDID header. Extension blocks are accepted
    /// by their non-zero tag and are validated by checksum only.
    pub fn from_bytes_checked(data: &[u8]) -> Result<Self, EdidError> {
        if data.len() != EDID_BLOCK_SIZE {
            return Err(EdidError::InvalidLength {
                expected: EDID_BLOCK_SIZE,
                actual: data.len(),
            });
        }
        let mut raw = [0u8; EDID_BLOCK_SIZE];
        raw.copy_from_slice(data);
        let block = Self { raw };
        block.validate()?;
        Ok(block)
    }

    /// Validate this block's base header when present and its checksum.
    pub fn validate(&self) -> Result<(), EdidError> {
        if self.raw[0] == 0 && self.raw[..8] != EDID_HEADER {
            return Err(EdidError::InvalidHeader);
        }
        let sum = self
            .raw
            .iter()
            .fold(0u8, |acc, &byte| acc.wrapping_add(byte));
        if sum != 0 {
            return Err(EdidError::InvalidChecksum { sum });
        }
        Ok(())
    }

    /// Read a progressive detailed timing from slot (0-3).
    ///
    /// Interlaced DTDs are available through [`Self::read_detailed_with_flags`]
    /// and are intentionally excluded from this legacy progressive-only view.
    #[must_use]
    pub fn read_detailed(&self, slot: usize) -> Option<DetailedTiming> {
        self.read_detailed_with_flags(slot)
            .and_then(|decoded| (!decoded.flags.interlaced()).then_some(decoded.timing))
    }

    /// Read a detailed timing together with its original DTD flags.
    #[must_use]
    pub fn read_detailed_with_flags(&self, slot: usize) -> Option<DecodedDtd> {
        if slot >= DETAILED_SLOTS {
            return None;
        }
        let off = DETAILED_START + slot * EDID_DESCRIPTOR_LEN;
        decode_dtd_bytes(&self.raw[off..off + EDID_DESCRIPTOR_LEN])
    }
    /// Validate fields specific to an EDID base block.
    pub(crate) fn validate_base(&self) -> Result<(), EdidError> {
        self.validate()?;
        if self.raw[..8] != EDID_HEADER {
            return Err(EdidError::InvalidHeader);
        }
        let major = self.raw[18];
        let minor = self.raw[19];
        if major != 1 || minor > 4 {
            return Err(EdidError::UnsupportedVersion { major, minor });
        }
        Ok(())
    }

    /// Write a detailed timing after validating every DTD field.
    pub fn write_detailed_checked(
        &mut self,
        slot: usize,
        t: &DetailedTiming,
    ) -> Result<(), crate::error::DtdError> {
        use crate::error::DtdError;

        if slot >= DETAILED_SLOTS {
            return Err(DtdError::SlotOutOfRange {
                slot,
                slots: DETAILED_SLOTS,
            });
        }
        crate::timing::validate_dtd(t)?;

        let pixel_clock = t
            .pixel_clock_khz
            .checked_add(5)
            .ok_or(DtdError::ArithmeticOverflow)?
            / 10;
        let h_blank = t
            .h_front
            .checked_add(t.h_sync)
            .and_then(|value| value.checked_add(t.h_back))
            .and_then(|value| {
                t.h_border
                    .checked_mul(2)
                    .and_then(|border| value.checked_add(border))
            })
            .ok_or(DtdError::ArithmeticOverflow)?;
        let v_blank = t
            .v_front
            .checked_add(t.v_sync)
            .and_then(|value| value.checked_add(t.v_back))
            .and_then(|value| {
                t.v_border
                    .checked_mul(2)
                    .and_then(|border| value.checked_add(border))
            })
            .ok_or(DtdError::ArithmeticOverflow)?;

        let off = DETAILED_START + slot * EDID_DESCRIPTOR_LEN;
        let d = &mut self.raw[off..off + EDID_DESCRIPTOR_LEN];
        let pixel_clock = pixel_clock as u16;
        let h_active = t.h_active as u16;
        let v_active = t.v_active as u16;
        let h_blank = h_blank as u16;
        let v_blank = v_blank as u16;

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
        d[11] = (((t.h_front as u16 >> 8) & 0x03) as u8) << 6
            | (((t.h_sync as u16 >> 8) & 0x03) as u8) << 4
            | (((t.v_front as u16 >> 4) & 0x03) as u8) << 2
            | ((t.v_sync as u16 >> 4) & 0x03) as u8;

        let hsize = h_active / 4;
        let vsize = v_active / 4;
        d[12] = hsize as u8;
        d[13] = vsize as u8;
        d[14] = (((hsize >> 8) & 0x0F) as u8) << 4 | ((vsize >> 8) & 0x0F) as u8;
        d[15] = t.h_border as u8;
        d[16] = t.v_border as u8;

        let mut flags = 0x18u8;
        if t.h_pol {
            flags |= 0x02;
        }
        if t.v_pol {
            flags |= 0x04;
        }
        d[17] = flags;
        Ok(())
    }

    /// Write a detailed timing, retaining the legacy infallible signature.
    ///
    /// Invalid timings are rejected without modifying the block. Use
    /// [`Self::write_detailed_checked`] when the failure reason is needed.
    pub fn write_detailed(&mut self, slot: usize, t: &DetailedTiming) {
        let _ = self.write_detailed_checked(slot, t);
    }
    /// Write a timing while preserving explicit DTD flag semantics.
    pub fn write_detailed_with_flags_checked(
        &mut self,
        slot: usize,
        timing: &DetailedTiming,
        flags: DtdFlags,
    ) -> Result<(), crate::error::DtdError> {
        self.write_detailed_checked(slot, timing)?;
        let offset = DETAILED_START + slot * EDID_DESCRIPTOR_LEN;
        self.raw[offset + 17] = flags.raw;
        Ok(())
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

    /// Replace timings after validating all DTD fields.
    ///
    /// Existing timing slots are reused before free slots. Monitor
    /// descriptors are never modified. If a timing is invalid, the block is
    /// left unchanged.
    pub fn write_resolutions_checked(
        &mut self,
        timings: &[DetailedTiming],
    ) -> Result<usize, crate::error::DtdError> {
        for timing in timings {
            crate::timing::validate_dtd(timing)?;
        }

        let kinds: Vec<SlotKind> = (0..DETAILED_SLOTS).map(|s| self.slot_kind(s)).collect();
        let mut slots: Vec<usize> = kinds
            .iter()
            .enumerate()
            .filter(|&(_, kind)| *kind != SlotKind::Descriptor)
            .map(|(index, _)| index)
            .collect();
        slots.sort_by_key(|&slot| {
            if kinds[slot] == SlotKind::Timing {
                0
            } else {
                1
            }
        });

        if timings.len() > slots.len() {
            return Err(crate::error::DtdError::NoAvailableSlot {
                requested: timings.len(),
                available: slots.len(),
            });
        }
        let mut written = 0;
        for (index, timing) in timings.iter().enumerate() {
            if let Some(&slot) = slots.get(index) {
                self.write_detailed_checked(slot, timing)?;
                written += 1;
            }
        }
        for &slot in &slots[written..] {
            if kinds[slot] == SlotKind::Timing {
                self.clear_slot(slot);
            }
        }
        Ok(written)
    }

    /// Replace timings while retaining the legacy infallible signature.
    ///
    /// Invalid timings are rejected without modifying the block. Use
    /// [`Self::write_resolutions_checked`] when the failure reason is needed.
    pub fn write_resolutions(&mut self, timings: &[DetailedTiming]) -> usize {
        if timings
            .iter()
            .any(|timing| crate::timing::validate_dtd(timing).is_err())
        {
            return 0;
        }
        let kinds: Vec<SlotKind> = (0..DETAILED_SLOTS)
            .map(|slot| self.slot_kind(slot))
            .collect();
        let mut slots: Vec<usize> = kinds
            .iter()
            .enumerate()
            .filter(|&(_, kind)| *kind != SlotKind::Descriptor)
            .map(|(index, _)| index)
            .collect();
        slots.sort_by_key(|&slot| {
            if kinds[slot] == SlotKind::Timing {
                0
            } else {
                1
            }
        });

        let mut written = 0;
        for (index, timing) in timings.iter().enumerate() {
            if let Some(&slot) = slots.get(index) {
                self.write_detailed(slot, timing);
                written += 1;
            }
        }
        for &slot in &slots[written..] {
            if kinds[slot] == SlotKind::Timing {
                self.clear_slot(slot);
            }
        }
        written
    }

    /// Get all detailed timings from this block
    #[must_use]
    pub fn detailed_timings(&self) -> Vec<DetailedTiming> {
        (0..DETAILED_SLOTS)
            .filter_map(|slot| self.read_detailed(slot))
            .collect()
    }

    /// Read all detailed timings with flags in this block.
    #[must_use]
    pub fn detailed_timings_flagged(&self) -> Vec<DecodedDtd> {
        (0..DETAILED_SLOTS)
            .filter_map(|slot| self.read_detailed_with_flags(slot))
            .collect()
    }

    /// Update checksum (byte 127)
    pub fn update_checksum(&mut self) {
        let sum: u8 = self.raw[..127].iter().fold(0u8, |a, &b| a.wrapping_add(b));
        self.raw[127] = (256u16 - sum as u16) as u8;
    }

    /// The raw 128 EDID bytes as a slice.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.raw
    }

    /// Format this 128-byte block as a lowercase continuous hex string (256 hex chars).
    #[must_use]
    pub fn to_hex(&self) -> String {
        let mut s = String::with_capacity(EDID_BLOCK_SIZE * 2);
        for &b in &self.raw {
            use std::fmt::Write;
            let _ = write!(s, "{b:02x}");
        }
        s
    }

    /// Format this block as 8 lines of 16 uppercase space-separated hex bytes.
    #[must_use]
    pub fn to_hex_formatted(&self) -> String {
        let mut s = String::with_capacity(EDID_BLOCK_SIZE * 3 + 8);
        for (i, &b) in self.raw.iter().enumerate() {
            if i > 0 {
                if i % 16 == 0 {
                    s.push('\n');
                } else {
                    s.push(' ');
                }
            }
            use std::fmt::Write;
            let _ = write!(s, "{b:02X}");
        }
        s
    }

    /// Parse a single 128-byte EDID block from a hex string.
    ///
    /// Accepts continuous hex, space-separated hex, and C-array hex (`0x00, 0xFF`),
    /// ignoring whitespace, commas, semicolons, brackets, and `0x`/`0X` prefixes.
    pub fn from_hex(s: &str) -> Result<Self, crate::error::HexError> {
        let bytes = parse_hex_bytes(s)?;
        if bytes.len() != EDID_BLOCK_SIZE {
            return Err(crate::error::HexError::InvalidLength { bytes: bytes.len() });
        }
        Self::from_bytes_checked(&bytes).map_err(crate::error::HexError::EdidError)
    }
}

/// A complete EDID made of one validated base block and its extensions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edid {
    /// Validated EDID base block.
    pub base: EdidBlock,
    /// Validated extension blocks in their original order.
    pub extensions: Vec<EdidBlock>,
}

impl Edid {
    /// Parse and validate a complete EDID byte sequence.
    pub fn from_bytes(data: &[u8]) -> Result<Self, EdidError> {
        if data.len() < EDID_BLOCK_SIZE || !data.len().is_multiple_of(EDID_BLOCK_SIZE) {
            return Err(EdidError::InvalidBlockSequenceLength { actual: data.len() });
        }

        let base = EdidBlock::from_bytes_checked(&data[..EDID_BLOCK_SIZE])?;
        base.validate_base()?;

        let actual = data.len() / EDID_BLOCK_SIZE - 1;
        let declared = base.raw[126] as usize;
        if declared != actual {
            return Err(EdidError::ExtensionCountMismatch { declared, actual });
        }

        let mut extensions = Vec::with_capacity(actual);
        let (extension_chunks, _) = data[EDID_BLOCK_SIZE..].as_chunks::<EDID_BLOCK_SIZE>();
        for chunk in extension_chunks {
            extensions.push(EdidBlock::from_bytes_checked(chunk)?);
        }

        Ok(Self { base, extensions })
    }

    /// Return the complete EDID as contiguous bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity((self.extensions.len() + 1) * EDID_BLOCK_SIZE);
        bytes.extend_from_slice(self.base.as_bytes());
        for extension in &self.extensions {
            bytes.extend_from_slice(extension.as_bytes());
        }
        bytes
    }
    /// Validate this EDID before a checked serialization.
    pub fn validate_for_serialization(&self) -> Result<(), EdidError> {
        self.validate()
    }

    /// Validate and return the complete EDID as contiguous bytes.
    ///
    /// Unlike [`Self::to_bytes`], this method checks the base block, extension
    /// count, and every extension checksum before producing output.
    pub fn to_bytes_checked(&self) -> Result<Vec<u8>, EdidError> {
        self.validate_for_serialization()?;
        Ok(self.to_bytes())
    }

    /// Validate the base block and every extension block.
    pub fn validate(&self) -> Result<(), EdidError> {
        self.base.validate_base()?;
        let actual = self.extensions.len();
        let declared = self.base.raw[126] as usize;
        if declared != actual {
            return Err(EdidError::ExtensionCountMismatch { declared, actual });
        }
        for extension in &self.extensions {
            extension.validate()?;
        }
        Ok(())
    }

    /// Format the complete EDID sequence (base + extensions) as continuous hex.
    #[must_use]
    pub fn to_hex(&self) -> String {
        let count = 1 + self.extensions.len();
        let mut s = String::with_capacity(count * EDID_BLOCK_SIZE * 2);
        s.push_str(&self.base.to_hex());
        for ext in &self.extensions {
            s.push_str(&ext.to_hex());
        }
        s
    }

    /// Format the complete EDID sequence as formatted 16-byte lines separated by empty lines.
    #[must_use]
    pub fn to_hex_formatted(&self) -> String {
        let count = 1 + self.extensions.len();
        let mut blocks = Vec::with_capacity(count);
        blocks.push(self.base.to_hex_formatted());
        for ext in &self.extensions {
            blocks.push(ext.to_hex_formatted());
        }
        blocks.join("\n\n")
    }

    /// Parse a complete EDID from a hex string (validates all blocks and sequence).
    pub fn from_hex(s: &str) -> Result<Self, crate::error::HexError> {
        let bytes = parse_hex_bytes(s)?;
        Self::from_bytes(&bytes).map_err(crate::error::HexError::EdidError)
    }

    /// Return all progressive detailed timings in this EDID (Base block DTDs + CTA-861 DTDs).
    #[must_use]
    pub fn all_detailed_timings(&self) -> Vec<DetailedTiming> {
        let mut timings = self.base.detailed_timings();
        for ext in &self.extensions {
            if let Ok(cta_timings) = ext.cta_detailed_timings() {
                timings.extend(cta_timings);
            }
        }
        timings
    }

    /// Return all detailed timings with flags in this EDID (Base block DTDs + CTA-861 DTDs).
    #[must_use]
    pub fn all_detailed_timings_flagged(&self) -> Vec<DecodedDtd> {
        let mut dtds = self.base.detailed_timings_flagged();
        for ext in &self.extensions {
            if let Ok(cta_dtds) = ext.cta_detailed_timings_flagged() {
                dtds.extend(cta_dtds);
            }
        }
        dtds
    }

    /// Return the preferred (first valid) detailed timing for this display, if defined.
    #[must_use]
    pub fn preferred_timing(&self) -> Option<DetailedTiming> {
        self.base
            .read_detailed(0)
            .or_else(|| self.all_detailed_timings().into_iter().next())
    }

    /// Extract the monitor product name from the Base Block descriptors, if present.
    #[must_use]
    pub fn monitor_name(&self) -> Option<String> {
        (0..DETAILED_SLOTS).find_map(|slot| match self.base.monitor_descriptor(slot) {
            Ok(Some(crate::metadata::MonitorDescriptor::ProductName(name))) => Some(name),
            _ => None,
        })
    }

    /// Extract the monitor serial number string from the Base Block descriptors, if present.
    #[must_use]
    pub fn serial_number(&self) -> Option<String> {
        (0..DETAILED_SLOTS).find_map(|slot| match self.base.monitor_descriptor(slot) {
            Ok(Some(crate::metadata::MonitorDescriptor::SerialNumber(serial))) => Some(serial),
            _ => None,
        })
    }
}

pub(crate) fn parse_hex_bytes(s: &str) -> Result<Vec<u8>, crate::error::HexError> {
    use crate::error::HexError;

    let mut bytes = Vec::new();
    let mut chars = s.char_indices().peekable();
    let mut current_nibble: Option<u8> = None;

    while let Some(&(idx, ch)) = chars.peek() {
        if ch.is_ascii_whitespace()
            || ch == ','
            || ch == ';'
            || ch == '{'
            || ch == '}'
            || ch == '['
            || ch == ']'
        {
            chars.next();
            continue;
        }
        if ch == '0' {
            let mut clone = chars.clone();
            clone.next();
            if let Some(&(_, next_ch)) = clone.peek()
                && (next_ch == 'x' || next_ch == 'X')
            {
                if current_nibble.is_some() {
                    return Err(HexError::OddLength {
                        length: bytes.len() * 2 + 1,
                    });
                }
                chars.next();
                chars.next();
                continue;
            }
        }
        chars.next();
        let digit = match ch {
            '0'..='9' => ch as u8 - b'0',
            'a'..='f' => ch as u8 - b'a' + 10,
            'A'..='F' => ch as u8 - b'A' + 10,
            _ => {
                return Err(HexError::InvalidHexCharacter {
                    offset: idx,
                    character: ch,
                });
            }
        };

        match current_nibble {
            None => {
                current_nibble = Some(digit);
            }
            Some(high) => {
                bytes.push((high << 4) | digit);
                current_nibble = None;
            }
        }
    }

    if current_nibble.is_some() {
        return Err(HexError::OddLength {
            length: bytes.len() * 2 + 1,
        });
    }

    Ok(bytes)
}

pub(crate) fn decode_dtd_bytes(d: &[u8]) -> Option<DecodedDtd> {
    if d.len() < EDID_DESCRIPTOR_LEN || d.iter().all(|&byte| byte == 0x01) {
        return None;
    }
    let pixel_clock = u16::from_le_bytes([d[0], d[1]]);
    if pixel_clock == 0 || pixel_clock < 1000 {
        return None;
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

    let sync_type = (d[17] >> 3) & 0x03;
    let h_pol = sync_type >= 0x02 && d[17] & 0x02 != 0;
    let v_pol = sync_type == 0x03 && d[17] & 0x04 != 0;
    let h_total = h_active + h_blank;
    let v_total = v_active + v_blank;
    let v_rate = (pixel_clock as f64 * 10_000.0) / (h_total as f64 * v_total as f64);

    Some(DecodedDtd {
        timing: DetailedTiming {
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
        },
        flags: DtdFlags { raw: d[17] },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::EdidError;
    use crate::timing::DetailedTiming;

    #[test]
    fn default_block_has_valid_checksum() {
        let block = EdidBlock::new_default();
        assert_eq!(block.validate(), Ok(()));
    }

    #[test]
    fn interlaced_dtd_is_available_through_flagged_api() {
        let mut block = EdidBlock::new_default();
        block.write_detailed_checked(0, &make_timing()).unwrap();
        let offset = DETAILED_START;
        block.raw[offset + 17] |= 0x80;

        assert!(block.read_detailed(0).is_none());
        let decoded = block.read_detailed_with_flags(0).unwrap();
        assert!(decoded.flags.interlaced());
        assert_eq!(decoded.timing.h_active, 1920);

        let mut roundtrip = EdidBlock::new_default();
        roundtrip
            .write_detailed_with_flags_checked(0, &decoded.timing, decoded.flags)
            .unwrap();
        assert_eq!(roundtrip.raw[offset + 17], block.raw[offset + 17]);
    }

    #[test]
    fn strict_parse_rejects_invalid_length_and_checksum() {
        let block = EdidBlock::new_default();
        assert!(matches!(
            EdidBlock::from_bytes_checked(&block.as_bytes()[..127]),
            Err(EdidError::InvalidLength {
                expected: EDID_BLOCK_SIZE,
                actual: 127
            })
        ));

        let mut corrupted = block.raw;
        corrupted[10] ^= 0x01;
        assert!(matches!(
            EdidBlock::from_bytes_checked(&corrupted),
            Err(EdidError::InvalidChecksum { .. })
        ));
    }

    #[test]
    fn strict_parse_rejects_invalid_base_header() {
        let mut block = EdidBlock::new_default();
        block.raw[1] = 0x00;
        block.update_checksum();
        assert!(matches!(
            EdidBlock::from_bytes_checked(block.as_bytes()),
            Err(EdidError::InvalidHeader)
        ));
    }

    #[test]
    fn strict_parse_accepts_valid_extension_block() {
        let mut block = EdidBlock::new_default();
        block.raw[0] = 0x02;
        block.update_checksum();
        assert!(EdidBlock::from_bytes_checked(block.as_bytes()).is_ok());
    }
    #[test]
    fn checked_dtd_write_rejects_invalid_timing_without_mutation() {
        let mut block = EdidBlock::new_default();
        let before = block.raw;
        let mut timing = make_timing();
        timing.h_front = 1024;
        assert!(matches!(
            block.write_detailed_checked(0, &timing),
            Err(crate::error::DtdError::FieldOutOfRange { .. })
        ));
        assert_eq!(block.raw, before);
        assert!(matches!(
            block.write_detailed_checked(4, &make_timing()),
            Err(crate::error::DtdError::SlotOutOfRange { slot: 4, slots: 4 })
        ));
    }

    #[test]
    fn aggregate_parser_preserves_valid_extensions() {
        let mut base = EdidBlock::new_default();
        let mut extension = EdidBlock::new_default();
        extension.raw[0] = 0x02;
        extension.raw[1] = 0x03;
        extension.update_checksum();
        base.raw[126] = 1;
        base.update_checksum();

        let mut bytes = base.as_bytes().to_vec();
        bytes.extend_from_slice(extension.as_bytes());
        let edid = Edid::from_bytes(&bytes).unwrap();

        assert_eq!(edid.extensions.len(), 1);
        assert_eq!(edid.extensions[0].raw[0], 0x02);
        assert_eq!(edid.to_bytes(), bytes);
    }

    #[test]
    fn checked_output_preserves_valid_edid_bytes() {
        let mut base = EdidBlock::new_default();
        let mut extension = EdidBlock::new_default();
        extension.raw[0] = 0x02;
        extension.raw[1] = 0x03;
        extension.update_checksum();
        base.raw[126] = 1;
        base.update_checksum();

        let mut bytes = base.as_bytes().to_vec();
        bytes.extend_from_slice(extension.as_bytes());
        let edid = Edid::from_bytes(&bytes).unwrap();

        assert_eq!(edid.to_bytes_checked().unwrap(), bytes);
    }

    #[test]
    fn checked_output_rejects_invalid_aggregate_state() {
        let mut invalid_checksum = Edid {
            base: EdidBlock::new_default(),
            extensions: Vec::new(),
        };
        invalid_checksum.base.raw[10] ^= 1;
        assert!(matches!(
            invalid_checksum.to_bytes_checked(),
            Err(EdidError::InvalidChecksum { .. })
        ));

        let invalid_count = Edid {
            base: EdidBlock::new_default(),
            extensions: vec![EdidBlock::new_default()],
        };
        assert!(matches!(
            invalid_count.to_bytes_checked(),
            Err(EdidError::ExtensionCountMismatch {
                declared: 0,
                actual: 1
            })
        ));
    }

    #[test]
    fn aggregate_parser_rejects_partial_and_count_mismatch() {
        assert!(matches!(
            Edid::from_bytes(&[0u8; 127]),
            Err(EdidError::InvalidBlockSequenceLength { actual: 127 })
        ));

        let base = EdidBlock::new_default();
        let mut bytes = base.as_bytes().to_vec();
        bytes.extend_from_slice(&[0u8; EDID_BLOCK_SIZE]);
        assert!(matches!(
            Edid::from_bytes(&bytes),
            Err(EdidError::ExtensionCountMismatch {
                declared: 0,
                actual: 1
            })
        ));
    }

    #[test]
    fn aggregate_parser_rejects_unsupported_base_version() {
        let mut base = EdidBlock::new_default();
        base.raw[18] = 2;
        base.update_checksum();
        assert!(matches!(
            Edid::from_bytes(base.as_bytes()),
            Err(EdidError::UnsupportedVersion { major: 2, minor: 4 })
        ));
    }

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

    #[test]
    fn hex_roundtrip_formatting_and_parsing() {
        let block = EdidBlock::new_default();
        let hex_compact = block.to_hex();
        assert_eq!(hex_compact.len(), 256);

        let parsed_compact = EdidBlock::from_hex(&hex_compact).unwrap();
        assert_eq!(parsed_compact, block);

        let hex_formatted = block.to_hex_formatted();
        let parsed_formatted = EdidBlock::from_hex(&hex_formatted).unwrap();
        assert_eq!(parsed_formatted, block);

        // C-array style hex parsing
        let c_array = "0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00";
        let parsed_chunk = parse_hex_bytes(c_array).unwrap();
        assert_eq!(
            parsed_chunk,
            &[0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00]
        );

        let edid = Edid {
            base: block.clone(),
            extensions: Vec::new(),
        };
        let edid_hex = edid.to_hex();
        let parsed_edid = Edid::from_hex(&edid_hex).unwrap();
        assert_eq!(parsed_edid, edid);
    }

    #[test]
    fn hex_parsing_rejects_invalid_inputs() {
        use crate::error::HexError;

        // Odd length
        assert!(matches!(
            EdidBlock::from_hex("00FF0"),
            Err(HexError::OddLength { length: 5 })
        ));

        // Invalid hex character
        assert!(matches!(
            EdidBlock::from_hex("00FFGG"),
            Err(HexError::InvalidHexCharacter { character: 'G', .. })
        ));

        // Odd nibble before 0x prefix
        assert!(matches!(
            EdidBlock::from_hex("A 0x12"),
            Err(HexError::OddLength { length: 1 })
        ));

        // Invalid length (not 128 bytes)
        assert!(matches!(
            EdidBlock::from_hex("00FFFFFFFFFFFF00"),
            Err(HexError::InvalidLength { bytes: 8 })
        ));
    }

    #[test]
    fn edid_aggregate_helpers_and_dtd_enumeration() {
        let mut base = EdidBlock::new_default();
        base.write_detailed_checked(0, &make_timing()).unwrap();
        base.set_monitor_descriptor(
            1,
            &crate::metadata::MonitorDescriptor::ProductName("TestMonitor".to_string()),
        )
        .unwrap();
        base.set_monitor_descriptor(
            2,
            &crate::metadata::MonitorDescriptor::SerialNumber("SN123456".to_string()),
        )
        .unwrap();
        base.raw[126] = 1; // 1 extension
        base.update_checksum();

        let mut cta = EdidBlock::new_default();
        cta.raw[0] = 0x02; // CTA
        cta.raw[1] = 3;
        cta.raw[2] = 4; // DTDs at offset 4
        // Write a DTD in CTA block (1920x1080)
        let dtd_bytes = base.raw[DETAILED_START..DETAILED_START + 18].to_vec();
        cta.raw[4..22].copy_from_slice(&dtd_bytes);
        cta.update_checksum();
        let mut full_bytes = base.as_bytes().to_vec();
        full_bytes.extend_from_slice(cta.as_bytes());
        let edid = Edid::from_bytes(&full_bytes).unwrap();
        assert_eq!(edid.monitor_name().as_deref(), Some("TestMonitor"));
        assert_eq!(edid.serial_number().as_deref(), Some("SN123456"));
        assert!(edid.preferred_timing().is_some());
        assert_eq!(edid.preferred_timing().unwrap().h_active, 1920);

        let all_timings = edid.all_detailed_timings();
        assert_eq!(all_timings.len(), 2);
        assert_eq!(all_timings[0].h_active, 1920);
        assert_eq!(all_timings[1].h_active, 1920);
    }
    #[test]
    fn default_block_uses_edid_unused_slot_encodings() {
        let block = EdidBlock::new_default();
        assert_eq!(&block.raw[38..54], &[0x01u8, 0x01].repeat(8));
        for slot in 0..4 {
            let offset = DETAILED_START + slot * EDID_DESCRIPTOR_LEN;
            assert_eq!(block.raw[offset..offset + 4], [0, 0, 0, 0x10]);
        }
    }
}
