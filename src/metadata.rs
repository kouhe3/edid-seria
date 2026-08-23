//! Base-block metadata and monitor descriptor access.

use crate::edid::{DETAILED_START, EdidBlock};
use crate::error::{DescriptorError, MetadataError, MetadataWriteError};

const EDID_HEADER: [u8; 8] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];
const DESCRIPTOR_LEN: usize = 18;
const DESCRIPTOR_SLOTS: usize = 4;
const TEXT_PAYLOAD_LEN: usize = 13;

/// One EDID chromaticity coordinate stored as a 10-bit fixed-point value.
///
/// The encoded value is in the inclusive range `0..=1023` and represents the
/// normalized coordinate divided by 1024. Keeping the encoded integer avoids
/// losing precision during a read/write round-trip.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ChromaticityPoint {
    /// Red/green/blue/white x coordinate, encoded on 10 bits.
    pub x: u16,
    /// Red/green/blue/white y coordinate, encoded on 10 bits.
    pub y: u16,
}

impl ChromaticityPoint {
    fn validate(self) -> Result<(), MetadataWriteError> {
        if self.x > 1023 {
            return Err(MetadataWriteError::InvalidChromaticityValue { value: self.x });
        }
        if self.y > 1023 {
            return Err(MetadataWriteError::InvalidChromaticityValue { value: self.y });
        }
        Ok(())
    }
}

/// The four chromaticity points encoded in an EDID base block.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ChromaticityCoordinates {
    /// Red primary point.
    pub red: ChromaticityPoint,
    /// Green primary point.
    pub green: ChromaticityPoint,
    /// Blue primary point.
    pub blue: ChromaticityPoint,
    /// White point.
    pub white: ChromaticityPoint,
}

impl ChromaticityCoordinates {
    fn validate(self) -> Result<(), MetadataWriteError> {
        self.red.validate()?;
        self.green.validate()?;
        self.blue.validate()?;
        self.white.validate()?;
        Ok(())
    }
}

/// One of the 17 established timings defined by the EDID base block.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EstablishedTiming {
    /// 720x400 at 70 Hz.
    Mode720x400At70,
    /// 720x400 at 88 Hz.
    Mode720x400At88,
    /// 640x480 at 60 Hz.
    Mode640x480At60,
    /// 640x480 at 67 Hz.
    Mode640x480At67,
    /// 640x480 at 72 Hz.
    Mode640x480At72,
    /// 640x480 at 75 Hz.
    Mode640x480At75,
    /// 800x600 at 56 Hz.
    Mode800x600At56,
    /// 800x600 at 60 Hz.
    Mode800x600At60,
    /// 800x600 at 72 Hz.
    Mode800x600At72,
    /// 800x600 at 75 Hz.
    Mode800x600At75,
    /// 832x624 at 75 Hz.
    Mode832x624At75,
    /// 1024x768 interlaced at 87 Hz.
    Mode1024x768At87Interlaced,
    /// 1024x768 at 60 Hz.
    Mode1024x768At60,
    /// 1024x768 at 70 Hz.
    Mode1024x768At70,
    /// 1024x768 at 75 Hz.
    Mode1024x768At75,
    /// 1280x1024 at 75 Hz.
    Mode1280x1024At75,
    /// 1152x870 at 75 Hz.
    Mode1152x870At75,
}

impl EstablishedTiming {
    const fn bit(self) -> (usize, u8) {
        match self {
            Self::Mode720x400At70 => (0, 0x80),
            Self::Mode720x400At88 => (0, 0x40),
            Self::Mode640x480At60 => (0, 0x20),
            Self::Mode640x480At67 => (0, 0x10),
            Self::Mode640x480At72 => (0, 0x08),
            Self::Mode640x480At75 => (0, 0x04),
            Self::Mode800x600At56 => (0, 0x02),
            Self::Mode800x600At60 => (0, 0x01),
            Self::Mode800x600At72 => (1, 0x80),
            Self::Mode800x600At75 => (1, 0x40),
            Self::Mode832x624At75 => (1, 0x20),
            Self::Mode1024x768At87Interlaced => (1, 0x10),
            Self::Mode1024x768At60 => (1, 0x08),
            Self::Mode1024x768At70 => (1, 0x04),
            Self::Mode1024x768At75 => (1, 0x02),
            Self::Mode1280x1024At75 => (1, 0x01),
            Self::Mode1152x870At75 => (2, 0x80),
        }
    }
}

/// Established timing bitfields from bytes 35-37 of a base block.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EstablishedTimings {
    /// Raw EDID bytes, including the seven manufacturer-reserved bits.
    pub raw: [u8; 3],
}

impl EstablishedTimings {
    /// Construct an established-timing view from its raw three bytes.
    #[must_use]
    pub const fn from_raw(raw: [u8; 3]) -> Self {
        Self { raw }
    }

    /// Return whether a defined established timing bit is set.
    #[must_use]
    pub const fn contains(self, timing: EstablishedTiming) -> bool {
        let (byte, mask) = timing.bit();
        self.raw[byte] & mask != 0
    }

    /// Return the defined established timings in EDID bit order.
    #[must_use]
    pub fn modes(self) -> Vec<EstablishedTiming> {
        [
            EstablishedTiming::Mode720x400At70,
            EstablishedTiming::Mode720x400At88,
            EstablishedTiming::Mode640x480At60,
            EstablishedTiming::Mode640x480At67,
            EstablishedTiming::Mode640x480At72,
            EstablishedTiming::Mode640x480At75,
            EstablishedTiming::Mode800x600At56,
            EstablishedTiming::Mode800x600At60,
            EstablishedTiming::Mode800x600At72,
            EstablishedTiming::Mode800x600At75,
            EstablishedTiming::Mode832x624At75,
            EstablishedTiming::Mode1024x768At87Interlaced,
            EstablishedTiming::Mode1024x768At60,
            EstablishedTiming::Mode1024x768At70,
            EstablishedTiming::Mode1024x768At75,
            EstablishedTiming::Mode1280x1024At75,
            EstablishedTiming::Mode1152x870At75,
        ]
        .into_iter()
        .filter(|&timing| self.contains(timing))
        .collect()
    }

    /// Return the manufacturer-reserved low seven bits of byte 37.
    #[must_use]
    pub const fn manufacturer_reserved(self) -> u8 {
        self.raw[2] & 0x7F
    }
}

/// Aspect-ratio code used by an EDID standard-timing entry.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StandardTimingAspectRatio {
    /// 16:10.
    SixteenByTen,
    /// 4:3.
    FourByThree,
    /// 5:4.
    FiveByFour,
    /// 16:9.
    SixteenByNine,
}

impl StandardTimingAspectRatio {
    const fn bits(self) -> u8 {
        match self {
            Self::SixteenByTen => 0,
            Self::FourByThree => 1,
            Self::FiveByFour => 2,
            Self::SixteenByNine => 3,
        }
    }

    const fn from_bits(bits: u8) -> Self {
        match bits & 0x03 {
            0 => Self::SixteenByTen,
            1 => Self::FourByThree,
            2 => Self::FiveByFour,
            _ => Self::SixteenByNine,
        }
    }

    /// Calculate the vertical active size represented by a horizontal size.
    #[must_use]
    pub const fn vertical_pixels(self, horizontal_pixels: u16) -> u16 {
        match self {
            Self::SixteenByTen => horizontal_pixels * 10 / 16,
            Self::FourByThree => horizontal_pixels * 3 / 4,
            Self::FiveByFour => horizontal_pixels * 4 / 5,
            Self::SixteenByNine => horizontal_pixels * 9 / 16,
        }
    }
}

/// One valid EDID standard-timing mode.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StandardTiming {
    /// Horizontal active pixels. Must be 256..=2288 and divisible by 8.
    pub horizontal_pixels: u16,
    /// EDID's two-bit aspect-ratio code.
    pub aspect_ratio: StandardTimingAspectRatio,
    /// Vertical refresh rate in Hz, represented as 60..=123.
    pub refresh_rate_hz: u8,
}

impl StandardTiming {
    /// Construct a representable standard-timing entry.
    pub fn new(
        horizontal_pixels: u16,
        aspect_ratio: StandardTimingAspectRatio,
        refresh_rate_hz: u8,
    ) -> Result<Self, MetadataWriteError> {
        let timing = Self {
            horizontal_pixels,
            aspect_ratio,
            refresh_rate_hz,
        };
        timing.validate()?;
        Ok(timing)
    }

    fn validate(self) -> Result<(), MetadataWriteError> {
        if !(256..=2288).contains(&self.horizontal_pixels)
            || !self.horizontal_pixels.is_multiple_of(8)
            || !(60..=123).contains(&self.refresh_rate_hz)
        {
            return Err(MetadataWriteError::InvalidStandardTiming {
                horizontal_pixels: self.horizontal_pixels,
                refresh_rate_hz: self.refresh_rate_hz,
            });
        }
        if self.horizontal_pixels == 256
            && matches!(self.aspect_ratio, StandardTimingAspectRatio::SixteenByTen)
            && self.refresh_rate_hz == 61
        {
            return Err(MetadataWriteError::ReservedStandardTimingEncoding);
        }
        Ok(())
    }

    /// Return the calculated vertical active size.
    #[must_use]
    pub const fn vertical_pixels(self) -> u16 {
        self.aspect_ratio.vertical_pixels(self.horizontal_pixels)
    }
}

/// A standard-timing slot, preserving unused and reserved encodings.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StandardTimingEntry {
    /// The EDID unused-slot encoding `01 01`.
    Unused,
    /// A valid standard-timing mode.
    Timing(StandardTiming),
    /// A non-mode encoding that is preserved but never reported as a mode.
    Reserved {
        /// Original two-byte encoding.
        raw: [u8; 2],
    },
}

impl StandardTimingEntry {
    fn decode(raw: [u8; 2]) -> Self {
        if raw == [0x01, 0x01] {
            return Self::Unused;
        }
        if raw[0] == 0 {
            return Self::Reserved { raw };
        }
        let timing = StandardTiming {
            horizontal_pixels: (raw[0] as u16 + 31) * 8,
            aspect_ratio: StandardTimingAspectRatio::from_bits(raw[1] >> 6),
            refresh_rate_hz: (raw[1] & 0x3F) + 60,
        };
        Self::Timing(timing)
    }

    fn encode(self) -> Result<[u8; 2], MetadataWriteError> {
        match self {
            Self::Unused => Ok([0x01, 0x01]),
            Self::Reserved { raw } => {
                if raw[0] != 0 {
                    return Err(MetadataWriteError::InvalidReservedStandardTimingEncoding { raw });
                }
                Ok(raw)
            }
            Self::Timing(timing) => {
                timing.validate()?;
                Ok([
                    (timing.horizontal_pixels / 8 - 31) as u8,
                    (timing.aspect_ratio.bits() << 6) | (timing.refresh_rate_hz - 60),
                ])
            }
        }
    }
}

/// Decoded identity and capability fields from an EDID base block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseMetadata {
    /// Three-letter manufacturer ID.
    pub manufacturer_id: String,
    /// Product code from the base block.
    pub product_code: u16,
    /// Product serial number from the base block.
    pub serial_number: u32,
    /// Week of manufacture, as encoded by EDID.
    pub manufacture_week: u8,
    /// Absolute year of manufacture.
    pub manufacture_year: u16,
    /// Raw video input definition byte.
    pub input: u8,
    /// Horizontal display size in centimeters.
    pub horizontal_size_cm: u8,
    /// Vertical display size in centimeters.
    pub vertical_size_cm: u8,
    /// Gamma multiplied by 100; `None` means unspecified.
    pub gamma: Option<u16>,
    /// Raw feature-support flags byte.
    pub feature_flags: u8,
}

impl BaseMetadata {
    /// Validate fields before writing them into a base block.
    pub fn validate(&self) -> Result<(), MetadataError> {
        let id = self.manufacturer_id.as_bytes();
        if id.len() != 3 {
            return Err(MetadataError::InvalidManufacturerId);
        }
        for (index, &byte) in id.iter().enumerate() {
            if !byte.is_ascii_uppercase() {
                return Err(MetadataError::InvalidManufacturerCharacter {
                    index,
                    character: byte as char,
                });
            }
        }
        if !(1990..=2245).contains(&self.manufacture_year) {
            return Err(MetadataError::InvalidManufactureYear {
                year: self.manufacture_year,
            });
        }
        if let Some(gamma) = self.gamma
            && !(101..=355).contains(&gamma)
        {
            return Err(MetadataError::InvalidGamma { value: gamma });
        }
        Ok(())
    }
}

/// A monitor descriptor stored in one of the four base-block descriptor slots.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MonitorDescriptor {
    /// Display serial number text descriptor.
    SerialNumber(String),
    /// Display product name text descriptor.
    ProductName(String),
    /// Vertical/horizontal range limits and maximum pixel clock.
    RangeLimits {
        /// Minimum vertical frequency in Hz.
        min_vertical_hz: u8,
        /// Maximum vertical frequency in Hz.
        max_vertical_hz: u8,
        /// Minimum horizontal frequency in kHz.
        min_horizontal_khz: u8,
        /// Maximum horizontal frequency in kHz.
        max_horizontal_khz: u8,
        /// Maximum pixel clock in MHz.
        max_pixel_clock_mhz: u16,
    },
    /// Unknown descriptor tag and its raw 13-byte payload.
    Unknown {
        /// Descriptor tag byte.
        tag: u8,
        /// Raw descriptor payload bytes after the reserved byte.
        payload: [u8; TEXT_PAYLOAD_LEN],
    },
}

impl MonitorDescriptor {
    fn encode(&self) -> Result<(u8, [u8; TEXT_PAYLOAD_LEN]), DescriptorError> {
        match self {
            Self::SerialNumber(text) => Ok((0xFF, encode_text(text)?)),
            Self::ProductName(text) => Ok((0xFC, encode_text(text)?)),
            Self::RangeLimits {
                min_vertical_hz,
                max_vertical_hz,
                min_horizontal_khz,
                max_horizontal_khz,
                max_pixel_clock_mhz,
            } => {
                if min_vertical_hz > max_vertical_hz
                    || min_horizontal_khz > max_horizontal_khz
                    || *max_pixel_clock_mhz > 2550
                    || *max_pixel_clock_mhz % 10 != 0
                {
                    return Err(DescriptorError::RangeOutOfBounds);
                }
                let mut payload = [0u8; TEXT_PAYLOAD_LEN];
                payload[0] = *min_vertical_hz;
                payload[1] = *max_vertical_hz;
                payload[2] = *min_horizontal_khz;
                payload[3] = *max_horizontal_khz;
                payload[4] = (*max_pixel_clock_mhz / 10) as u8;
                Ok((0xFD, payload))
            }
            Self::Unknown { tag, payload } => Ok((*tag, *payload)),
        }
    }
}

impl EdidBlock {
    /// Decode the four 10-bit chromaticity points from the base block.
    pub fn chromaticity(&self) -> Result<ChromaticityCoordinates, MetadataError> {
        if self.raw[..8] != EDID_HEADER {
            return Err(MetadataError::NotBaseBlock);
        }
        let low_x = self.raw[25];
        let low_y = self.raw[26];
        let point = |x_high: usize, y_high: usize, x_shift: u8, y_shift: u8| ChromaticityPoint {
            x: ((self.raw[x_high] as u16) << 2) | ((low_x >> x_shift) as u16 & 0x03),
            y: ((self.raw[y_high] as u16) << 2) | ((low_y >> y_shift) as u16 & 0x03),
        };
        Ok(ChromaticityCoordinates {
            red: point(27, 28, 6, 6),
            green: point(29, 30, 4, 4),
            blue: point(31, 32, 2, 2),
            white: point(33, 34, 0, 0),
        })
    }

    /// Encode the four chromaticity points and update the base checksum.
    pub fn set_chromaticity(
        &mut self,
        chromaticity: &ChromaticityCoordinates,
    ) -> Result<(), MetadataWriteError> {
        if self.raw[..8] != EDID_HEADER {
            return Err(MetadataWriteError::NotBaseBlock);
        }
        chromaticity.validate()?;
        let points = [
            chromaticity.red,
            chromaticity.green,
            chromaticity.blue,
            chromaticity.white,
        ];
        self.raw[25] = ((points[0].x & 0x03) << 6
            | (points[1].x & 0x03) << 4
            | (points[2].x & 0x03) << 2
            | (points[3].x & 0x03)) as u8;
        self.raw[26] = ((points[0].y & 0x03) << 6
            | (points[1].y & 0x03) << 4
            | (points[2].y & 0x03) << 2
            | (points[3].y & 0x03)) as u8;
        for (index, point) in points.into_iter().enumerate() {
            self.raw[27 + index * 2] = (point.x >> 2) as u8;
            self.raw[28 + index * 2] = (point.y >> 2) as u8;
        }
        self.update_checksum();
        Ok(())
    }

    /// Decode the established-timing bitfields from bytes 35-37.
    pub fn established_timings(&self) -> Result<EstablishedTimings, MetadataError> {
        if self.raw[..8] != EDID_HEADER {
            return Err(MetadataError::NotBaseBlock);
        }
        Ok(EstablishedTimings::from_raw([
            self.raw[35],
            self.raw[36],
            self.raw[37],
        ]))
    }

    /// Replace established-timing bytes and update the base checksum.
    pub fn set_established_timings(
        &mut self,
        timings: &EstablishedTimings,
    ) -> Result<(), MetadataError> {
        if self.raw[..8] != EDID_HEADER {
            return Err(MetadataError::NotBaseBlock);
        }
        self.raw[35..38].copy_from_slice(&timings.raw);
        self.update_checksum();
        Ok(())
    }

    /// Decode all eight standard-timing slots, preserving unused/reserved bytes.
    pub fn standard_timings(&self) -> Result<[StandardTimingEntry; 8], MetadataError> {
        if self.raw[..8] != EDID_HEADER {
            return Err(MetadataError::NotBaseBlock);
        }
        let mut timings = [StandardTimingEntry::Unused; 8];
        for (index, entry) in timings.iter_mut().enumerate() {
            let offset = 38 + index * 2;
            *entry = StandardTimingEntry::decode([self.raw[offset], self.raw[offset + 1]]);
        }
        Ok(timings)
    }

    /// Replace all standard-timing slots and update the base checksum.
    pub fn set_standard_timings(
        &mut self,
        timings: &[StandardTimingEntry; 8],
    ) -> Result<(), MetadataWriteError> {
        if self.raw[..8] != EDID_HEADER {
            return Err(MetadataWriteError::NotBaseBlock);
        }
        let mut encoded = [[0u8; 2]; 8];
        for (index, timing) in timings.iter().copied().enumerate() {
            encoded[index] = timing.encode()?;
        }
        for (index, bytes) in encoded.into_iter().enumerate() {
            let offset = 38 + index * 2;
            self.raw[offset..offset + 2].copy_from_slice(&bytes);
        }
        self.update_checksum();
        Ok(())
    }

    /// Decode base-block identity and capability fields.
    pub fn metadata(&self) -> Result<BaseMetadata, MetadataError> {
        if self.raw[..8] != EDID_HEADER {
            return Err(MetadataError::NotBaseBlock);
        }
        let encoded = u16::from_be_bytes([self.raw[8], self.raw[9]]);
        let codes = [
            (encoded >> 10) & 0x1F,
            (encoded >> 5) & 0x1F,
            encoded & 0x1F,
        ];
        if codes.iter().any(|&code| !(1..=26).contains(&code)) {
            return Err(MetadataError::InvalidManufacturerId);
        }
        let manufacturer_id: String = codes
            .iter()
            .map(|&code| char::from(b'A' + code as u8 - 1))
            .collect();
        let gamma = (self.raw[23] != 0).then(|| 100 + self.raw[23] as u16);
        Ok(BaseMetadata {
            manufacturer_id,
            product_code: u16::from_le_bytes([self.raw[10], self.raw[11]]),
            serial_number: u32::from_le_bytes([
                self.raw[12],
                self.raw[13],
                self.raw[14],
                self.raw[15],
            ]),
            manufacture_week: self.raw[16],
            manufacture_year: 1990 + self.raw[17] as u16,
            input: self.raw[20],
            horizontal_size_cm: self.raw[21],
            vertical_size_cm: self.raw[22],
            gamma,
            feature_flags: self.raw[24],
        })
    }

    /// Encode base-block identity and capability fields.
    pub fn set_metadata(&mut self, metadata: &BaseMetadata) -> Result<(), MetadataError> {
        if self.raw[..8] != EDID_HEADER {
            return Err(MetadataError::NotBaseBlock);
        }
        metadata.validate()?;
        let id = metadata.manufacturer_id.as_bytes();
        let encoded = ((id[0] - b'A' + 1) as u16) << 10
            | ((id[1] - b'A' + 1) as u16) << 5
            | (id[2] - b'A' + 1) as u16;
        self.raw[8..10].copy_from_slice(&encoded.to_be_bytes());
        self.raw[10..12].copy_from_slice(&metadata.product_code.to_le_bytes());
        self.raw[12..16].copy_from_slice(&metadata.serial_number.to_le_bytes());
        self.raw[16] = metadata.manufacture_week;
        self.raw[17] = (metadata.manufacture_year - 1990) as u8;
        self.raw[20] = metadata.input;
        self.raw[21] = metadata.horizontal_size_cm;
        self.raw[22] = metadata.vertical_size_cm;
        self.raw[23] = metadata.gamma.map_or(0, |gamma| (gamma - 100) as u8);
        self.raw[24] = metadata.feature_flags;
        self.update_checksum();
        Ok(())
    }

    /// Decode a monitor descriptor slot, or return `None` for a timing/free slot.
    pub fn monitor_descriptor(
        &self,
        slot: usize,
    ) -> Result<Option<MonitorDescriptor>, DescriptorError> {
        let offset = descriptor_offset(slot)?;
        let descriptor = &self.raw[offset..offset + DESCRIPTOR_LEN];
        if descriptor.iter().all(|&byte| byte == 0x01)
            || descriptor[0] != 0
            || descriptor[1] != 0
            || descriptor.iter().all(|&byte| byte == 0)
            || (descriptor[3] == 0x10 && descriptor[4..].iter().all(|&byte| byte == 0))
        {
            return Ok(None);
        }

        let payload: [u8; TEXT_PAYLOAD_LEN] = descriptor[5..]
            .try_into()
            .expect("EDID descriptor payload is fixed at 13 bytes");
        let value = match descriptor[3] {
            0xFF => MonitorDescriptor::SerialNumber(decode_text(&payload)?),
            0xFC => MonitorDescriptor::ProductName(decode_text(&payload)?),
            0xFD => MonitorDescriptor::RangeLimits {
                min_vertical_hz: payload[0],
                max_vertical_hz: payload[1],
                min_horizontal_khz: payload[2],
                max_horizontal_khz: payload[3],
                max_pixel_clock_mhz: payload[4] as u16 * 10,
            },
            tag => MonitorDescriptor::Unknown { tag, payload },
        };
        Ok(Some(value))
    }

    /// Replace a monitor descriptor slot and update the block checksum.
    pub fn set_monitor_descriptor(
        &mut self,
        slot: usize,
        descriptor: &MonitorDescriptor,
    ) -> Result<(), DescriptorError> {
        if self.raw[..8] != EDID_HEADER {
            return Err(DescriptorError::NotBaseBlock);
        }
        let offset = descriptor_offset(slot)?;
        let (tag, payload) = descriptor.encode()?;
        let target = &mut self.raw[offset..offset + DESCRIPTOR_LEN];
        target.fill(0);
        target[3] = tag;
        target[5..].copy_from_slice(&payload);
        self.update_checksum();
        Ok(())
    }
}

fn descriptor_offset(slot: usize) -> Result<usize, DescriptorError> {
    if slot >= DESCRIPTOR_SLOTS {
        return Err(DescriptorError::SlotOutOfRange {
            slot,
            slots: DESCRIPTOR_SLOTS,
        });
    }
    Ok(DETAILED_START + slot * DESCRIPTOR_LEN)
}

fn encode_text(text: &str) -> Result<[u8; TEXT_PAYLOAD_LEN], DescriptorError> {
    let bytes = text.as_bytes();
    if bytes.len() > TEXT_PAYLOAD_LEN - 1 {
        return Err(DescriptorError::TextTooLong {
            max: TEXT_PAYLOAD_LEN - 1,
            actual: bytes.len(),
        });
    }
    if !bytes.is_ascii() {
        return Err(DescriptorError::NonAsciiText);
    }
    let mut payload = [b' '; TEXT_PAYLOAD_LEN];
    payload[..bytes.len()].copy_from_slice(bytes);
    payload[bytes.len()] = 0x0A;
    Ok(payload)
}

fn decode_text(payload: &[u8; TEXT_PAYLOAD_LEN]) -> Result<String, DescriptorError> {
    if payload.iter().any(|&byte| !byte.is_ascii()) {
        return Err(DescriptorError::NonAsciiText);
    }
    let end = payload
        .iter()
        .position(|&byte| byte == 0x0A || byte == 0)
        .unwrap_or(TEXT_PAYLOAD_LEN);
    let text = payload[..end].to_vec();
    let text = String::from_utf8_lossy(&text)
        .trim_end_matches(' ')
        .to_owned();
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::{
        BaseMetadata, ChromaticityCoordinates, ChromaticityPoint, EstablishedTiming,
        EstablishedTimings, MonitorDescriptor, StandardTiming, StandardTimingAspectRatio,
        StandardTimingEntry,
    };
    use crate::edid::EdidBlock;
    use crate::error::{DescriptorError, MetadataError, MetadataWriteError};

    #[test]
    fn metadata_roundtrips_identity_fields() {
        let mut block = EdidBlock::new_default();
        let metadata = BaseMetadata {
            manufacturer_id: "ABC".to_owned(),
            product_code: 0x1234,
            serial_number: 0x89AB_CDEF,
            manufacture_week: 12,
            manufacture_year: 2024,
            input: 0x80,
            horizontal_size_cm: 60,
            vertical_size_cm: 34,
            gamma: Some(220),
            feature_flags: 0x0A,
        };
        block.set_metadata(&metadata).unwrap();
        assert_eq!(block.metadata().unwrap(), metadata);
    }

    #[test]
    fn monitor_descriptors_roundtrip_text_and_range() {
        let mut block = EdidBlock::new_default();
        block
            .set_monitor_descriptor(2, &MonitorDescriptor::ProductName("TEST PANEL".to_owned()))
            .unwrap();
        assert_eq!(
            block.monitor_descriptor(2).unwrap(),
            Some(MonitorDescriptor::ProductName("TEST PANEL".to_owned()))
        );

        let range = MonitorDescriptor::RangeLimits {
            min_vertical_hz: 48,
            max_vertical_hz: 144,
            min_horizontal_khz: 30,
            max_horizontal_khz: 240,
            max_pixel_clock_mhz: 600,
        };
        block.set_monitor_descriptor(3, &range).unwrap();
        assert_eq!(block.monitor_descriptor(3).unwrap(), Some(range));
    }

    #[test]
    fn descriptor_rejects_long_text_without_mutation() {
        let mut block = EdidBlock::new_default();
        let before = block.raw;
        let result = block.set_monitor_descriptor(
            0,
            &MonitorDescriptor::ProductName("0123456789ABC".to_owned()),
        );
        assert!(result.is_err());
        assert_eq!(block.raw, before);
    }
    #[test]
    fn metadata_setters_reject_extension_blocks_without_mutation() {
        let metadata = BaseMetadata {
            manufacturer_id: "ABC".to_owned(),
            product_code: 1,
            serial_number: 2,
            manufacture_week: 1,
            manufacture_year: 2024,
            input: 0x80,
            horizontal_size_cm: 1,
            vertical_size_cm: 1,
            gamma: None,
            feature_flags: 0,
        };
        let mut block = EdidBlock::new_default();
        block.raw[0] = 0x02;
        block.update_checksum();
        let before = block.raw;
        assert_eq!(
            block.set_metadata(&metadata),
            Err(MetadataError::NotBaseBlock)
        );
        assert_eq!(
            block.set_monitor_descriptor(0, &MonitorDescriptor::ProductName("X".to_owned())),
            Err(DescriptorError::NotBaseBlock)
        );
        assert_eq!(block.raw, before);
    }

    #[test]
    fn chromaticity_roundtrips_edid_fixed_point_fields() {
        let mut block = EdidBlock::new_default();
        let expected = ChromaticityCoordinates {
            red: ChromaticityPoint { x: 0x155, y: 0x2AA },
            green: ChromaticityPoint { x: 0x3FF, y: 0x001 },
            blue: ChromaticityPoint { x: 0x100, y: 0x200 },
            white: ChromaticityPoint { x: 0x300, y: 0x3AA },
        };

        block.set_chromaticity(&expected).unwrap();

        assert_eq!(block.chromaticity().unwrap(), expected);
        assert_eq!(block.raw[25], 0x70);
        assert_eq!(block.raw[26], 0x92);
        assert_eq!(
            &block.raw[27..35],
            &[0x55, 0xAA, 0xFF, 0x00, 0x40, 0x80, 0xC0, 0xEA]
        );
        assert_eq!(block.validate(), Ok(()));
    }

    #[test]
    fn established_timings_decode_known_bits_and_preserve_reserved_bits() {
        let mut block = EdidBlock::new_default();
        block.raw[35] = 0xA0;
        block.raw[36] = 0x01;
        block.raw[37] = 0x95;
        block.update_checksum();

        let timings = block.established_timings().unwrap();
        assert!(timings.contains(EstablishedTiming::Mode720x400At70));
        assert!(timings.contains(EstablishedTiming::Mode640x480At60));
        assert!(timings.contains(EstablishedTiming::Mode1280x1024At75));
        assert!(timings.contains(EstablishedTiming::Mode1152x870At75));
        assert_eq!(timings.manufacturer_reserved(), 0x15);

        let replacement = EstablishedTimings::from_raw([0x01, 0x80, 0x7F]);
        block.set_established_timings(&replacement).unwrap();
        assert_eq!(block.established_timings().unwrap(), replacement);
        assert_eq!(block.validate(), Ok(()));
    }

    #[test]
    fn standard_timings_decode_unused_and_reserved_entries() {
        let mut block = EdidBlock::new_default();
        block.raw[38..40].copy_from_slice(&[0xDF, 0x01]);
        block.raw[40..42].copy_from_slice(&[0x01, 0x01]);
        block.raw[42..44].copy_from_slice(&[0x00, 0x00]);
        block.update_checksum();

        let timings = block.standard_timings().unwrap();
        assert_eq!(
            timings[0],
            StandardTimingEntry::Timing(
                StandardTiming::new(2032, StandardTimingAspectRatio::SixteenByTen, 61,).unwrap()
            )
        );
        assert_eq!(timings[1], StandardTimingEntry::Unused);
        assert_eq!(timings[2], StandardTimingEntry::Reserved { raw: [0, 0] });
    }

    #[test]
    fn standard_timings_write_is_checksum_safe_and_rejects_unrepresentable_modes() {
        let mut block = EdidBlock::new_default();
        let entries = [
            StandardTimingEntry::Timing(
                StandardTiming::new(1920, StandardTimingAspectRatio::SixteenByNine, 60).unwrap(),
            ),
            StandardTimingEntry::Unused,
            StandardTimingEntry::Reserved { raw: [0, 0] },
            StandardTimingEntry::Unused,
            StandardTimingEntry::Unused,
            StandardTimingEntry::Unused,
            StandardTimingEntry::Unused,
            StandardTimingEntry::Unused,
        ];

        block.set_standard_timings(&entries).unwrap();
        assert_eq!(&block.raw[38..42], &[0xD1, 0xC0, 0x01, 0x01]);
        assert_eq!(&block.raw[42..44], &[0, 0]);
        assert_eq!(block.standard_timings().unwrap(), entries);
        assert_eq!(block.validate(), Ok(()));

        let invalid = StandardTiming {
            horizontal_pixels: 1921,
            aspect_ratio: StandardTimingAspectRatio::SixteenByNine,
            refresh_rate_hz: 60,
        };
        let before = block.raw;
        let mut invalid_entries = entries;
        invalid_entries[0] = StandardTimingEntry::Timing(invalid);
        assert!(block.set_standard_timings(&invalid_entries).is_err());
        assert_eq!(block.raw, before);

        assert_eq!(
            StandardTiming::new(256, StandardTimingAspectRatio::SixteenByTen, 61),
            Err(MetadataWriteError::ReservedStandardTimingEncoding)
        );

        let reserved_before = block.raw;
        let mut invalid_reserved = entries;
        invalid_reserved[0] = StandardTimingEntry::Reserved { raw: [0xD1, 0xC0] };
        assert_eq!(
            block.set_standard_timings(&invalid_reserved),
            Err(MetadataWriteError::InvalidReservedStandardTimingEncoding { raw: [0xD1, 0xC0] })
        );
        assert_eq!(block.raw, reserved_before);
    }
}
