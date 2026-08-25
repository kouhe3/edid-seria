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
        if !matches!(self.manufacture_week, 0..=54 | 255) {
            return Err(MetadataError::InvalidManufactureWeek {
                week: self.manufacture_week,
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

/// One white-point entry in an Additional Color Point descriptor (tag 0xFB).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AdditionalColorPoint {
    /// White point index (1..=255).
    pub index: u8,
    /// 10-bit Chromaticity point.
    pub point: ChromaticityPoint,
    /// Gamma multiplied by 100, e.g. 220 for gamma 2.2; `None` when unspecified.
    pub gamma: Option<u16>,
}

/// One of the 44 established timings defined by the Established Timings III descriptor (Tag 0xF7).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EstablishedTiming3 {
    /// 640 x 350 @ 85Hz
    Res640x350_85Hz,
    /// 640 x 400 @ 85Hz
    Res640x400_85Hz,
    /// 720 x 400 @ 85Hz
    Res720x400_85Hz,
    /// 640 x 480 @ 85Hz
    Res640x480_85Hz,
    /// 848 x 480 @ 60Hz
    Res848x480_60Hz,
    /// 800 x 600 @ 85Hz
    Res800x600_85Hz,
    /// 1024 x 768 @ 85Hz
    Res1024x768_85Hz,
    /// 1152 x 864 @ 75Hz
    Res1152x864_75Hz,
    /// 1280 x 768 @ 60Hz (Reduced Blanking)
    Res1280x768_60HzRb,
    /// 1280 x 768 @ 60Hz
    Res1280x768_60Hz,
    /// 1280 x 768 @ 75Hz
    Res1280x768_75Hz,
    /// 1280 x 768 @ 85Hz
    Res1280x768_85Hz,
    /// 1280 x 960 @ 60Hz
    Res1280x960_60Hz,
    /// 1280 x 960 @ 85Hz
    Res1280x960_85Hz,
    /// 1280 x 1024 @ 60Hz
    Res1280x1024_60Hz,
    /// 1280 x 1024 @ 85Hz
    Res1280x1024_85Hz,
    /// 1360 x 768 @ 60Hz
    Res1360x768_60Hz,
    /// 1440 x 900 @ 60Hz (Reduced Blanking)
    Res1440x900_60HzRb,
    /// 1440 x 900 @ 60Hz
    Res1440x900_60Hz,
    /// 1440 x 900 @ 75Hz
    Res1440x900_75Hz,
    /// 1440 x 900 @ 85Hz
    Res1440x900_85Hz,
    /// 1400 x 1050 @ 60Hz (Reduced Blanking)
    Res1400x1050_60HzRb,
    /// 1400 x 1050 @ 60Hz
    Res1400x1050_60Hz,
    /// 1400 x 1050 @ 75Hz
    Res1400x1050_75Hz,
    /// 1400 x 1050 @ 85Hz
    Res1400x1050_85Hz,
    /// 1680 x 1050 @ 60Hz (Reduced Blanking)
    Res1680x1050_60HzRb,
    /// 1680 x 1050 @ 60Hz
    Res1680x1050_60Hz,
    /// 1680 x 1050 @ 75Hz
    Res1680x1050_75Hz,
    /// 1680 x 1050 @ 85Hz
    Res1680x1050_85Hz,
    /// 1600 x 1200 @ 60Hz
    Res1600x1200_60Hz,
    /// 1600 x 1200 @ 65Hz
    Res1600x1200_65Hz,
    /// 1600 x 1200 @ 70Hz
    Res1600x1200_70Hz,
    /// 1600 x 1200 @ 75Hz
    Res1600x1200_75Hz,
    /// 1600 x 1200 @ 85Hz
    Res1600x1200_85Hz,
    /// 1792 x 1344 @ 60Hz
    Res1792x1344_60Hz,
    /// 1792 x 1344 @ 75Hz
    Res1792x1344_75Hz,
    /// 1856 x 1392 @ 60Hz
    Res1856x1392_60Hz,
    /// 1856 x 1392 @ 75Hz
    Res1856x1392_75Hz,
    /// 1920 x 1200 @ 60Hz (Reduced Blanking)
    Res1920x1200_60HzRb,
    /// 1920 x 1200 @ 60Hz
    Res1920x1200_60Hz,
    /// 1920 x 1200 @ 75Hz
    Res1920x1200_75Hz,
    /// 1920 x 1200 @ 85Hz
    Res1920x1200_85Hz,
    /// 1920 x 1440 @ 60Hz
    Res1920x1440_60Hz,
    /// 1920 x 1440 @ 75Hz
    Res1920x1440_75Hz,
}

/// Established Timings III bitfield (Tag 0xF7, EDID 1.4 §3.10.3.5).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EstablishedTimings3 {
    /// Descriptor revision (typically 0x0A).
    pub revision: u8,
    /// Raw 6-byte bitmap payload (bytes 6..=11 of the 18-byte descriptor).
    pub raw: [u8; 6],
}

impl Default for EstablishedTimings3 {
    fn default() -> Self {
        Self {
            revision: 0x0A,
            raw: [0u8; 6],
        }
    }
}

impl EstablishedTimings3 {
    /// Create a new Established Timings III descriptor with default revision 0x0A.
    #[must_use]
    pub const fn new(raw: [u8; 6]) -> Self {
        Self {
            revision: 0x0A,
            raw,
        }
    }

    /// Check whether a specific Established Timing III is supported.
    #[must_use]
    pub fn has_timing(&self, timing: EstablishedTiming3) -> bool {
        let (byte_idx, bit_idx) = match timing {
            EstablishedTiming3::Res640x350_85Hz => (0, 7),
            EstablishedTiming3::Res640x400_85Hz => (0, 6),
            EstablishedTiming3::Res720x400_85Hz => (0, 5),
            EstablishedTiming3::Res640x480_85Hz => (0, 4),
            EstablishedTiming3::Res848x480_60Hz => (0, 3),
            EstablishedTiming3::Res800x600_85Hz => (0, 2),
            EstablishedTiming3::Res1024x768_85Hz => (0, 1),
            EstablishedTiming3::Res1152x864_75Hz => (0, 0),
            EstablishedTiming3::Res1280x768_60HzRb => (1, 7),
            EstablishedTiming3::Res1280x768_60Hz => (1, 6),
            EstablishedTiming3::Res1280x768_75Hz => (1, 5),
            EstablishedTiming3::Res1280x768_85Hz => (1, 4),
            EstablishedTiming3::Res1280x960_60Hz => (1, 3),
            EstablishedTiming3::Res1280x960_85Hz => (1, 2),
            EstablishedTiming3::Res1280x1024_60Hz => (1, 1),
            EstablishedTiming3::Res1280x1024_85Hz => (1, 0),
            EstablishedTiming3::Res1360x768_60Hz => (2, 7),
            EstablishedTiming3::Res1440x900_60HzRb => (2, 6),
            EstablishedTiming3::Res1440x900_60Hz => (2, 5),
            EstablishedTiming3::Res1440x900_75Hz => (2, 4),
            EstablishedTiming3::Res1440x900_85Hz => (2, 3),
            EstablishedTiming3::Res1400x1050_60HzRb => (2, 2),
            EstablishedTiming3::Res1400x1050_60Hz => (2, 1),
            EstablishedTiming3::Res1400x1050_75Hz => (2, 0),
            EstablishedTiming3::Res1400x1050_85Hz => (3, 7),
            EstablishedTiming3::Res1680x1050_60HzRb => (3, 6),
            EstablishedTiming3::Res1680x1050_60Hz => (3, 5),
            EstablishedTiming3::Res1680x1050_75Hz => (3, 4),
            EstablishedTiming3::Res1680x1050_85Hz => (3, 3),
            EstablishedTiming3::Res1600x1200_60Hz => (3, 2),
            EstablishedTiming3::Res1600x1200_65Hz => (3, 1),
            EstablishedTiming3::Res1600x1200_70Hz => (3, 0),
            EstablishedTiming3::Res1600x1200_75Hz => (4, 7),
            EstablishedTiming3::Res1600x1200_85Hz => (4, 6),
            EstablishedTiming3::Res1792x1344_60Hz => (4, 5),
            EstablishedTiming3::Res1792x1344_75Hz => (4, 4),
            EstablishedTiming3::Res1856x1392_60Hz => (4, 3),
            EstablishedTiming3::Res1856x1392_75Hz => (4, 2),
            EstablishedTiming3::Res1920x1200_60HzRb => (4, 1),
            EstablishedTiming3::Res1920x1200_60Hz => (4, 0),
            EstablishedTiming3::Res1920x1200_75Hz => (5, 7),
            EstablishedTiming3::Res1920x1200_85Hz => (5, 6),
            EstablishedTiming3::Res1920x1440_60Hz => (5, 5),
            EstablishedTiming3::Res1920x1440_75Hz => (5, 4),
        };
        (self.raw[byte_idx] & (1 << bit_idx)) != 0
    }

    /// Enable or disable a specific Established Timing III.
    pub fn set_timing(&mut self, timing: EstablishedTiming3, enabled: bool) {
        let (byte_idx, bit_idx) = match timing {
            EstablishedTiming3::Res640x350_85Hz => (0, 7),
            EstablishedTiming3::Res640x400_85Hz => (0, 6),
            EstablishedTiming3::Res720x400_85Hz => (0, 5),
            EstablishedTiming3::Res640x480_85Hz => (0, 4),
            EstablishedTiming3::Res848x480_60Hz => (0, 3),
            EstablishedTiming3::Res800x600_85Hz => (0, 2),
            EstablishedTiming3::Res1024x768_85Hz => (0, 1),
            EstablishedTiming3::Res1152x864_75Hz => (0, 0),
            EstablishedTiming3::Res1280x768_60HzRb => (1, 7),
            EstablishedTiming3::Res1280x768_60Hz => (1, 6),
            EstablishedTiming3::Res1280x768_75Hz => (1, 5),
            EstablishedTiming3::Res1280x768_85Hz => (1, 4),
            EstablishedTiming3::Res1280x960_60Hz => (1, 3),
            EstablishedTiming3::Res1280x960_85Hz => (1, 2),
            EstablishedTiming3::Res1280x1024_60Hz => (1, 1),
            EstablishedTiming3::Res1280x1024_85Hz => (1, 0),
            EstablishedTiming3::Res1360x768_60Hz => (2, 7),
            EstablishedTiming3::Res1440x900_60HzRb => (2, 6),
            EstablishedTiming3::Res1440x900_60Hz => (2, 5),
            EstablishedTiming3::Res1440x900_75Hz => (2, 4),
            EstablishedTiming3::Res1440x900_85Hz => (2, 3),
            EstablishedTiming3::Res1400x1050_60HzRb => (2, 2),
            EstablishedTiming3::Res1400x1050_60Hz => (2, 1),
            EstablishedTiming3::Res1400x1050_75Hz => (2, 0),
            EstablishedTiming3::Res1400x1050_85Hz => (3, 7),
            EstablishedTiming3::Res1680x1050_60HzRb => (3, 6),
            EstablishedTiming3::Res1680x1050_60Hz => (3, 5),
            EstablishedTiming3::Res1680x1050_75Hz => (3, 4),
            EstablishedTiming3::Res1680x1050_85Hz => (3, 3),
            EstablishedTiming3::Res1600x1200_60Hz => (3, 2),
            EstablishedTiming3::Res1600x1200_65Hz => (3, 1),
            EstablishedTiming3::Res1600x1200_70Hz => (3, 0),
            EstablishedTiming3::Res1600x1200_75Hz => (4, 7),
            EstablishedTiming3::Res1600x1200_85Hz => (4, 6),
            EstablishedTiming3::Res1792x1344_60Hz => (4, 5),
            EstablishedTiming3::Res1792x1344_75Hz => (4, 4),
            EstablishedTiming3::Res1856x1392_60Hz => (4, 3),
            EstablishedTiming3::Res1856x1392_75Hz => (4, 2),
            EstablishedTiming3::Res1920x1200_60HzRb => (4, 1),
            EstablishedTiming3::Res1920x1200_60Hz => (4, 0),
            EstablishedTiming3::Res1920x1200_75Hz => (5, 7),
            EstablishedTiming3::Res1920x1200_85Hz => (5, 6),
            EstablishedTiming3::Res1920x1440_60Hz => (5, 5),
            EstablishedTiming3::Res1920x1440_75Hz => (5, 4),
        };
        if enabled {
            self.raw[byte_idx] |= 1 << bit_idx;
        } else {
            self.raw[byte_idx] &= !(1 << bit_idx);
        }
    }
}

/// Aspect ratio code for a CVT 3-byte timing code (Tag 0xF8).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CvtAspectRatio {
    /// 4:3 aspect ratio (code 00b).
    Ratio4x3,
    /// 16:9 aspect ratio (code 01b).
    Ratio16x9,
    /// 16:10 aspect ratio (code 10b).
    Ratio16x10,
    /// 15:9 aspect ratio (code 11b).
    Ratio15x9,
}
/// Preferred vertical refresh rate code for a CVT 3-byte timing code (Tag 0xF8).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CvtPreferredRate {
    /// 50 Hz vertical rate, standard blanking (code 000b).
    Hz50Standard,
    /// 60 Hz vertical rate, standard blanking (code 001b).
    Hz60Standard,
    /// 75 Hz vertical rate, standard blanking (code 010b).
    Hz75Standard,
    /// 85 Hz vertical rate, standard blanking (code 011b).
    Hz85Standard,
    /// 60 Hz vertical rate, reduced blanking (code 100b).
    Hz60ReducedBlanking,
    /// Reserved preferred rate code (code 101b..111b).
    Reserved(u8),
}

/// Supported vertical refresh rates bitmask for a CVT 3-byte timing code (Tag 0xF8).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct CvtSupportedRates {
    /// Raw byte 2 supported rate bits 4..=0.
    pub raw: u8,
}

impl CvtSupportedRates {
    /// 50 Hz with standard blanking (bit 4, mask 0x10).
    #[must_use]
    pub fn supports_50hz_standard(&self) -> bool {
        self.raw & 0x10 != 0
    }
    /// 60 Hz with standard blanking (bit 3, mask 0x08).
    #[must_use]
    pub fn supports_60hz_standard(&self) -> bool {
        self.raw & 0x08 != 0
    }
    /// 75 Hz with standard blanking (bit 2, mask 0x04).
    #[must_use]
    pub fn supports_75hz_standard(&self) -> bool {
        self.raw & 0x04 != 0
    }
    /// 85 Hz with standard blanking (bit 1, mask 0x02).
    #[must_use]
    pub fn supports_85hz_standard(&self) -> bool {
        self.raw & 0x02 != 0
    }
    /// 60 Hz with reduced blanking (bit 0, mask 0x01).
    #[must_use]
    pub fn supports_60hz_reduced_blanking(&self) -> bool {
        self.raw & 0x01 != 0
    }
}
/// One 3-byte CVT timing code entry in an EDID 1.4 Tag 0xF8 descriptor.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Cvt3ByteTimingEntry {
    /// Empty or unused timing slot (all 3 bytes 0x00).
    Unused,
    /// Active CVT 3-byte timing definition.
    Active {
        /// Addressable vertical lines (e.g. 768, 900, 1050, 1080, 1200).
        addressable_lines: u16,
        /// Aspect ratio.
        aspect_ratio: CvtAspectRatio,
        /// Preferred vertical rate.
        preferred_rate: CvtPreferredRate,
        /// Supported vertical rates bitmask.
        supported_rates: CvtSupportedRates,
    },
    /// Reserved or unparseable 3-byte timing code.
    Reserved([u8; 3]),
}

impl Cvt3ByteTimingEntry {
    /// Decode a 3-byte CVT timing entry per EDID 1.4 §3.10.3.6.
    #[must_use]
    pub fn decode(bytes: [u8; 3]) -> Self {
        if bytes == [0, 0, 0] {
            return Self::Unused;
        }
        let lines_low = bytes[0] as u16;
        let lines_high = ((bytes[1] >> 4) & 0x0F) as u16;
        let aspect_code = (bytes[1] >> 2) & 0x03;
        let addressable_lines = ((lines_low | (lines_high << 8)) + 1) * 2;

        let aspect_ratio = match aspect_code {
            0 => CvtAspectRatio::Ratio4x3,
            1 => CvtAspectRatio::Ratio16x9,
            2 => CvtAspectRatio::Ratio16x10,
            _ => CvtAspectRatio::Ratio15x9,
        };
        let rate_code = (bytes[2] >> 5) & 0x07;
        let preferred_rate = match rate_code {
            0 => CvtPreferredRate::Hz50Standard,
            1 => CvtPreferredRate::Hz60Standard,
            2 => CvtPreferredRate::Hz75Standard,
            3 => CvtPreferredRate::Hz85Standard,
            4 => CvtPreferredRate::Hz60ReducedBlanking,
            code => CvtPreferredRate::Reserved(code),
        };

        Self::Active {
            addressable_lines,
            aspect_ratio,
            preferred_rate,
            supported_rates: CvtSupportedRates {
                raw: bytes[2] & 0x1F,
            },
        }
    }

    /// Encode this CVT 3-byte timing entry into 3 bytes.
    pub fn encode(&self) -> Result<[u8; 3], &'static str> {
        match self {
            Self::Unused => Ok([0, 0, 0]),
            Self::Reserved(raw) => Ok(*raw),
            Self::Active {
                addressable_lines,
                aspect_ratio,
                preferred_rate,
                supported_rates,
            } => {
                if *addressable_lines == 0
                    || *addressable_lines > 8192
                    || addressable_lines % 2 != 0
                {
                    return Err("addressable lines must be non-zero even number <= 8192");
                }
                let encoded_lines = (addressable_lines / 2).saturating_sub(1);
                let lines_low = (encoded_lines & 0xFF) as u8;
                let lines_high = ((encoded_lines >> 8) & 0x0F) as u8;
                let aspect_code = match aspect_ratio {
                    CvtAspectRatio::Ratio4x3 => 0,
                    CvtAspectRatio::Ratio16x9 => 1,
                    CvtAspectRatio::Ratio16x10 => 2,
                    CvtAspectRatio::Ratio15x9 => 3,
                };
                let preferred_code = match preferred_rate {
                    CvtPreferredRate::Hz50Standard => 0,
                    CvtPreferredRate::Hz60Standard => 1,
                    CvtPreferredRate::Hz75Standard => 2,
                    CvtPreferredRate::Hz85Standard => 3,
                    CvtPreferredRate::Hz60ReducedBlanking => 4,
                    CvtPreferredRate::Reserved(code) => code & 0x07,
                };
                let byte1 = (lines_high << 4) | (aspect_code << 2);
                let byte2 = (preferred_code << 5) | (supported_rates.raw & 0x1F);
                Ok([lines_low, byte1, byte2])
            }
        }
    }
}

/// Display Color Management (DCM) data descriptor (Tag 0xF9, EDID 1.4 §3.10.3.7).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ColorManagementDescriptor {
    /// Descriptor version/revision (typically 0x03).
    pub revision: u8,
    /// Red a3 coefficient.
    pub red_a3: u16,
    /// Red a2 coefficient.
    pub red_a2: u16,
    /// Green a3 coefficient.
    pub green_a3: u16,
    /// Green a2 coefficient.
    pub green_a2: u16,
    /// Blue a3 coefficient.
    pub blue_a3: u16,
    /// Blue a2 coefficient.
    pub blue_a2: u16,
}

/// Secondary GTF curve parameters in a Range Limits descriptor (Tag 0xFD, EDID 1.4 §3.10.3.3.1).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SecondaryGtfParameters {
    /// Start break frequency in kHz (must be even, encoded as F_start / 2 in 2 kHz units).
    pub start_horizontal_frequency_khz: u16,
    /// C parameter multiplied by 2 (e.g. 80 for C = 40.0, stored in 0.5% units).
    pub parameter_c: u8,
    /// M parameter slope in kHz / % (16-bit little-endian, e.g. 600).
    pub slope_m: u16,
    /// K parameter offset (0..=255, e.g. 128).
    pub offset_k: u8,
    /// J parameter scaling factor multiplied by 2 (e.g. 40 for J = 20.0, stored in 0.5% units).
    pub scaling_j: u8,
}

/// CVT support definition in a Range Limits descriptor (Tag 0xFD, EDID 1.4 §3.10.3.3.1).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CvtRangeSupport {
    /// CVT revision (e.g. 0x11 for CVT 1.1).
    pub revision: u8,
    /// Additional pixel clock precision / clock step.
    pub max_pixel_clock_precision: u8,
    /// Maximum active horizontal pixels in 8-pixel units (pixels = value * 8).
    pub max_active_pixels: u16,
    /// Supported aspect ratios bitmask.
    pub supported_aspect_ratios: u8,
    /// Preferred aspect ratio and blanking flags.
    pub preferred_aspect_ratio_and_flags: u8,
    /// Display scaling support flags.
    pub scaling_support: u8,
    /// Preferred vertical refresh rate in Hz.
    pub preferred_vertical_rate_hz: u8,
}

/// Optional extended timing formula definition in a Range Limits descriptor.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RangeLimitsExtension {
    /// Standard GTF or Range Limits only (code 0x00 or 0x01).
    Standard,
    /// Secondary GTF curve definition (code 0x02).
    SecondaryGtf(SecondaryGtfParameters),
    /// CVT support definition (code 0x04).
    Cvt(CvtRangeSupport),
    /// Raw unparsed extension definition.
    Unknown {
        /// Timing definition code (byte 10 of descriptor).
        definition_code: u8,
        /// Raw 7 payload bytes (bytes 11..17 of descriptor).
        payload: [u8; 7],
    },
}
/// A monitor descriptor stored in one of the four base-block descriptor slots.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MonitorDescriptor {
    /// Display serial number text descriptor.
    SerialNumber(String),
    /// Display product name text descriptor.
    ProductName(String),
    /// Alphanumeric Data String text descriptor (Tag 0xFE).
    AlphanumericString(String),
    /// Additional White Point Data (Tag 0xFB).
    AdditionalColorPoint {
        /// First white point entry.
        point1: AdditionalColorPoint,
        /// Optional second white point entry.
        point2: Option<AdditionalColorPoint>,
    },
    /// Additional Standard Timing Identifications (Tag 0xFA, 6 slots).
    AdditionalStandardTimings([StandardTimingEntry; 6]),
    /// Established Timings III (Tag 0xF7, 6 bytes bitmap).
    EstablishedTimings3(EstablishedTimings3),
    /// CVT 3-Byte Timing Codes (Tag 0xF8, 4 slots).
    Cvt3ByteTimings([Cvt3ByteTimingEntry; 4]),
    /// Display Color Management (Tag 0xF9).
    ColorManagement(ColorManagementDescriptor),
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
        /// Extended timing definition (GTF secondary curve, CVT support, etc.).
        extension: RangeLimitsExtension,
    },
    /// Dummy descriptor (Tag 0x10, zero payload).
    Dummy,
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
            Self::AlphanumericString(text) => Ok((0xFE, encode_text(text)?)),
            Self::AdditionalColorPoint { point1, point2 } => {
                let mut payload = [0u8; TEXT_PAYLOAD_LEN];
                encode_color_point(point1, &mut payload[0..5])?;
                if let Some(p2) = point2 {
                    encode_color_point(p2, &mut payload[5..10])?;
                }
                payload[10] = 0x0A;
                payload[11] = 0x20;
                payload[12] = 0x20;
                Ok((0xFB, payload))
            }
            Self::AdditionalStandardTimings(timings) => {
                let mut payload = [0u8; TEXT_PAYLOAD_LEN];
                for (i, timing) in timings.iter().copied().enumerate() {
                    let off = i * 2;
                    let bytes = timing
                        .encode()
                        .map_err(DescriptorError::StandardTimingError)?;
                    payload[off..off + 2].copy_from_slice(&bytes);
                }
                payload[12] = 0x0A;
                Ok((0xFA, payload))
            }
            Self::EstablishedTimings3(timings) => {
                if timings.revision != 0x0A {
                    return Err(DescriptorError::InvalidRevision {
                        tag: 0xF7,
                        revision: timings.revision,
                    });
                }
                let mut payload = [0u8; TEXT_PAYLOAD_LEN];
                payload[0] = timings.revision;
                payload[1..7].copy_from_slice(&timings.raw);
                Ok((0xF7, payload))
            }
            Self::Cvt3ByteTimings(timings) => {
                let mut payload = [0u8; TEXT_PAYLOAD_LEN];
                payload[0] = 0x01; // Revision 1
                for (i, timing) in timings.iter().enumerate() {
                    let off = 1 + i * 3;
                    let encoded = timing
                        .encode()
                        .map_err(|reason| DescriptorError::InvalidCvtTiming { slot: i, reason })?;
                    payload[off..off + 3].copy_from_slice(&encoded);
                }
                Ok((0xF8, payload))
            }
            Self::ColorManagement(dcm) => {
                if dcm.revision != 0x03 {
                    return Err(DescriptorError::InvalidRevision {
                        tag: 0xF9,
                        revision: dcm.revision,
                    });
                }
                let mut payload = [0u8; TEXT_PAYLOAD_LEN];
                payload[0] = dcm.revision;
                payload[1..3].copy_from_slice(&dcm.red_a3.to_le_bytes());
                payload[3..5].copy_from_slice(&dcm.red_a2.to_le_bytes());
                payload[5..7].copy_from_slice(&dcm.green_a3.to_le_bytes());
                payload[7..9].copy_from_slice(&dcm.green_a2.to_le_bytes());
                payload[9..11].copy_from_slice(&dcm.blue_a3.to_le_bytes());
                payload[11..13].copy_from_slice(&dcm.blue_a2.to_le_bytes());
                Ok((0xF9, payload))
            }
            Self::Dummy => Ok((0x10, [0u8; TEXT_PAYLOAD_LEN])),
            Self::RangeLimits {
                min_vertical_hz,
                max_vertical_hz,
                min_horizontal_khz,
                max_horizontal_khz,
                max_pixel_clock_mhz,
                extension,
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
                match extension {
                    RangeLimitsExtension::Standard => {
                        payload[5] = 0x00;
                        payload[6..13].copy_from_slice(&[0x0A, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20]);
                    }
                    RangeLimitsExtension::SecondaryGtf(gtf) => {
                        if gtf.start_horizontal_frequency_khz > 510
                            || gtf.start_horizontal_frequency_khz % 2 != 0
                        {
                            return Err(DescriptorError::InvalidRangeExtension);
                        }
                        payload[5] = 0x02;
                        payload[6] = 0x00;
                        payload[7] = (gtf.start_horizontal_frequency_khz / 2) as u8;
                        payload[8] = gtf.parameter_c;
                        payload[9..11].copy_from_slice(&gtf.slope_m.to_le_bytes());
                        payload[11] = gtf.offset_k;
                        payload[12] = gtf.scaling_j;
                    }
                    RangeLimitsExtension::Cvt(cvt) => {
                        if cvt.max_active_pixels > 8184
                            || cvt.max_active_pixels % 8 != 0
                            || cvt.max_pixel_clock_precision > 63
                        {
                            return Err(DescriptorError::InvalidRangeExtension);
                        }
                        payload[5] = 0x04;
                        payload[6] = cvt.revision;
                        let max_pix_units = cvt.max_active_pixels / 8;
                        let max_pix_msb = ((max_pix_units >> 8) & 0x03) as u8;
                        payload[7] = (cvt.max_pixel_clock_precision << 2) | max_pix_msb;
                        payload[8] = (max_pix_units & 0xFF) as u8;
                        payload[9] = cvt.supported_aspect_ratios;
                        payload[10] = cvt.preferred_aspect_ratio_and_flags;
                        payload[11] = cvt.scaling_support;
                        payload[12] = cvt.preferred_vertical_rate_hz;
                    }
                    RangeLimitsExtension::Unknown {
                        definition_code,
                        payload: ext_payload,
                    } => {
                        payload[5] = *definition_code;
                        payload[6..13].copy_from_slice(ext_payload);
                    }
                }
                Ok((0xFD, payload))
            }
            Self::Unknown { tag, payload } => Ok((*tag, *payload)),
        }
    }
}

impl EdidBlock {
    /// Construct a valid default base block populated with identity metadata.
    pub fn from_metadata(metadata: &BaseMetadata) -> Result<Self, MetadataError> {
        let mut block = Self::new_default();
        block.raw[38..54].fill(0x01);
        for slot in 0..4 {
            let offset = 54 + slot * 18;
            block.raw[offset..offset + 18].fill(0);
            block.raw[offset + 3] = 0x10;
        }
        block.set_metadata(metadata)?;
        Ok(block)
    }
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
        {
            return Ok(None);
        }

        let payload: [u8; TEXT_PAYLOAD_LEN] = descriptor[5..]
            .try_into()
            .expect("EDID descriptor payload is fixed at 13 bytes");
        let value = match descriptor[3] {
            0xFF => MonitorDescriptor::SerialNumber(decode_text(&payload)?),
            0xFC => MonitorDescriptor::ProductName(decode_text(&payload)?),
            0xFE => MonitorDescriptor::AlphanumericString(decode_text(&payload)?),
            0xFB => {
                let point1 =
                    decode_color_point(&payload[0..5]).ok_or(DescriptorError::RangeOutOfBounds)?;
                let point2 = decode_color_point(&payload[5..10]);
                MonitorDescriptor::AdditionalColorPoint { point1, point2 }
            }
            0xFA => {
                let mut timings = [StandardTimingEntry::Unused; 6];
                for (i, entry) in timings.iter_mut().enumerate() {
                    let off = i * 2;
                    *entry = StandardTimingEntry::decode([payload[off], payload[off + 1]]);
                }
                MonitorDescriptor::AdditionalStandardTimings(timings)
            }
            0xF7 if payload[0] == 0x0A => {
                let raw: [u8; 6] = payload[1..7]
                    .try_into()
                    .expect("established timings 3 is 6 bytes");
                MonitorDescriptor::EstablishedTimings3(EstablishedTimings3 {
                    revision: payload[0],
                    raw,
                })
            }
            0xF8 if payload[0] == 0x01 => {
                let slot0 = Cvt3ByteTimingEntry::decode([payload[1], payload[2], payload[3]]);
                let slot1 = Cvt3ByteTimingEntry::decode([payload[4], payload[5], payload[6]]);
                let slot2 = Cvt3ByteTimingEntry::decode([payload[7], payload[8], payload[9]]);
                let slot3 = Cvt3ByteTimingEntry::decode([payload[10], payload[11], payload[12]]);
                MonitorDescriptor::Cvt3ByteTimings([slot0, slot1, slot2, slot3])
            }
            0xF9 if payload[0] == 0x03 => {
                MonitorDescriptor::ColorManagement(ColorManagementDescriptor {
                    revision: payload[0],
                    red_a3: u16::from_le_bytes([payload[1], payload[2]]),
                    red_a2: u16::from_le_bytes([payload[3], payload[4]]),
                    green_a3: u16::from_le_bytes([payload[5], payload[6]]),
                    green_a2: u16::from_le_bytes([payload[7], payload[8]]),
                    blue_a3: u16::from_le_bytes([payload[9], payload[10]]),
                    blue_a2: u16::from_le_bytes([payload[11], payload[12]]),
                })
            }
            0xFD => {
                let definition_code = payload[5];
                let extension = match definition_code {
                    0x00 | 0x01 => RangeLimitsExtension::Standard,
                    0x02 => RangeLimitsExtension::SecondaryGtf(SecondaryGtfParameters {
                        start_horizontal_frequency_khz: (payload[7] as u16) * 2,
                        parameter_c: payload[8],
                        slope_m: u16::from_le_bytes([payload[9], payload[10]]),
                        offset_k: payload[11],
                        scaling_j: payload[12],
                    }),
                    0x04 => RangeLimitsExtension::Cvt(CvtRangeSupport {
                        revision: payload[6],
                        max_pixel_clock_precision: payload[7] >> 2,
                        max_active_pixels: (((payload[7] & 0x03) as u16) << 8
                            | (payload[8] as u16))
                            * 8,
                        supported_aspect_ratios: payload[9],
                        preferred_aspect_ratio_and_flags: payload[10],
                        scaling_support: payload[11],
                        preferred_vertical_rate_hz: payload[12],
                    }),
                    _ => {
                        let ext_payload: [u8; 7] = payload[6..13]
                            .try_into()
                            .expect("range limits extension is 7 bytes");
                        RangeLimitsExtension::Unknown {
                            definition_code,
                            payload: ext_payload,
                        }
                    }
                };
                MonitorDescriptor::RangeLimits {
                    min_vertical_hz: payload[0],
                    max_vertical_hz: payload[1],
                    min_horizontal_khz: payload[2],
                    max_horizontal_khz: payload[3],
                    max_pixel_clock_mhz: payload[4] as u16 * 10,
                    extension,
                }
            }
            0x10 if payload.iter().all(|&b| b == 0) => MonitorDescriptor::Dummy,
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

fn decode_color_point(data: &[u8]) -> Option<AdditionalColorPoint> {
    let index = data[0];
    if index == 0 {
        return None;
    }
    let low = data[1];
    let x_low = (low >> 2) & 0x03;
    let y_low = low & 0x03;
    let x = ((data[2] as u16) << 2) | (x_low as u16);
    let y = ((data[3] as u16) << 2) | (y_low as u16);
    let gamma = (data[4] != 0).then(|| 100 + data[4] as u16);
    Some(AdditionalColorPoint {
        index,
        point: ChromaticityPoint { x, y },
        gamma,
    })
}

fn encode_color_point(cp: &AdditionalColorPoint, target: &mut [u8]) -> Result<(), DescriptorError> {
    if cp.index == 0 {
        return Err(DescriptorError::RangeOutOfBounds);
    }
    if cp.point.x > 1023 {
        return Err(DescriptorError::InvalidChromaticityCoordinate { value: cp.point.x });
    }
    if cp.point.y > 1023 {
        return Err(DescriptorError::InvalidChromaticityCoordinate { value: cp.point.y });
    }
    if let Some(gamma) = cp.gamma
        && !(101..=355).contains(&gamma)
    {
        return Err(DescriptorError::InvalidGamma { value: gamma });
    }
    target[0] = cp.index;
    let x_low = (cp.point.x & 0x03) as u8;
    let y_low = (cp.point.y & 0x03) as u8;
    target[1] = (x_low << 2) | y_low;
    target[2] = (cp.point.x >> 2) as u8;
    target[3] = (cp.point.y >> 2) as u8;
    target[4] = cp.gamma.map_or(0, |g| (g - 100) as u8);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn constructs_base_block_from_metadata() {
        let metadata = BaseMetadata {
            manufacturer_id: "ABC".to_owned(),
            product_code: 0x1234,
            serial_number: 0x5678_9ABC,
            manufacture_week: 25,
            manufacture_year: 2024,
            input: 0x80,
            horizontal_size_cm: 60,
            vertical_size_cm: 34,
            gamma: Some(220),
            feature_flags: 0x0A,
        };

        let block = EdidBlock::from_metadata(&metadata).unwrap();
        assert_eq!(block.metadata().unwrap(), metadata);
        assert_eq!(block.validate(), Ok(()));
    }

    #[test]
    fn rejects_reserved_manufacture_week() {
        let metadata = BaseMetadata {
            manufacturer_id: "ABC".to_owned(),
            product_code: 1,
            serial_number: 1,
            manufacture_week: 55,
            manufacture_year: 2024,
            input: 0x80,
            horizontal_size_cm: 1,
            vertical_size_cm: 1,
            gamma: None,
            feature_flags: 0,
        };

        assert!(matches!(
            EdidBlock::from_metadata(&metadata),
            Err(MetadataError::InvalidManufactureWeek { week: 55 })
        ));
    }

    #[test]
    fn constructed_base_block_uses_edid_unused_markers() {
        let metadata = BaseMetadata {
            manufacturer_id: "ABC".to_owned(),
            product_code: 1,
            serial_number: 1,
            manufacture_week: 0,
            manufacture_year: 2024,
            input: 0x80,
            horizontal_size_cm: 1,
            vertical_size_cm: 1,
            gamma: None,
            feature_flags: 0,
        };

        let block = EdidBlock::from_metadata(&metadata).unwrap();
        let unused_timings = [0x01u8, 0x01].repeat(8);
        assert_eq!(&block.raw[38..54], unused_timings.as_slice());
        for slot in 0..4 {
            let offset = 54 + slot * 18;
            assert_eq!(block.raw[offset..offset + 4], [0, 0, 0, 0x10]);
        }
        assert_eq!(block.validate(), Ok(()));
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
            extension: RangeLimitsExtension::Standard,
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

    #[test]
    fn alphanumeric_string_descriptor_roundtrips() {
        let mut block = EdidBlock::new_default();
        let descriptor = MonitorDescriptor::AlphanumericString("DISPLAY-01".to_owned());
        block.set_monitor_descriptor(1, &descriptor).unwrap();
        assert_eq!(block.monitor_descriptor(1).unwrap(), Some(descriptor));
        assert_eq!(block.validate(), Ok(()));
    }

    #[test]
    fn additional_color_point_descriptor_roundtrips() {
        use super::AdditionalColorPoint;

        let mut block = EdidBlock::new_default();
        let descriptor = MonitorDescriptor::AdditionalColorPoint {
            point1: AdditionalColorPoint {
                index: 1,
                point: ChromaticityPoint { x: 313, y: 329 },
                gamma: Some(220),
            },
            point2: Some(AdditionalColorPoint {
                index: 2,
                point: ChromaticityPoint { x: 283, y: 298 },
                gamma: None,
            }),
        };
        block.set_monitor_descriptor(2, &descriptor).unwrap();
        assert_eq!(block.monitor_descriptor(2).unwrap(), Some(descriptor));
        assert_eq!(block.validate(), Ok(()));

        // Rejection of invalid coordinates or gamma
        let invalid_coord = MonitorDescriptor::AdditionalColorPoint {
            point1: AdditionalColorPoint {
                index: 1,
                point: ChromaticityPoint { x: 1024, y: 300 },
                gamma: None,
            },
            point2: None,
        };
        assert!(matches!(
            block.set_monitor_descriptor(2, &invalid_coord),
            Err(DescriptorError::InvalidChromaticityCoordinate { value: 1024 })
        ));

        let invalid_index = MonitorDescriptor::AdditionalColorPoint {
            point1: AdditionalColorPoint {
                index: 0,
                point: ChromaticityPoint { x: 300, y: 300 },
                gamma: None,
            },
            point2: None,
        };
        assert!(matches!(
            block.set_monitor_descriptor(2, &invalid_index),
            Err(DescriptorError::RangeOutOfBounds)
        ));
    }

    #[test]
    fn additional_standard_timings_descriptor_roundtrips() {
        let mut block = EdidBlock::new_default();
        let descriptor = MonitorDescriptor::AdditionalStandardTimings([
            StandardTimingEntry::Timing(
                StandardTiming::new(1920, StandardTimingAspectRatio::SixteenByNine, 60).unwrap(),
            ),
            StandardTimingEntry::Timing(
                StandardTiming::new(1280, StandardTimingAspectRatio::FourByThree, 75).unwrap(),
            ),
            StandardTimingEntry::Unused,
            StandardTimingEntry::Unused,
            StandardTimingEntry::Unused,
            StandardTimingEntry::Unused,
        ]);
        block.set_monitor_descriptor(3, &descriptor).unwrap();
        assert_eq!(block.monitor_descriptor(3).unwrap(), Some(descriptor));
        assert_eq!(block.validate(), Ok(()));
    }

    #[test]
    fn dummy_descriptor_roundtrips_and_preserves_checksum() {
        let mut block = EdidBlock::new_default();
        block
            .set_monitor_descriptor(0, &MonitorDescriptor::Dummy)
            .unwrap();
        assert_eq!(
            block.monitor_descriptor(0).unwrap(),
            Some(MonitorDescriptor::Dummy)
        );
        assert_eq!(block.validate(), Ok(()));
    }

    #[test]
    fn established_timings_3_roundtrips_and_queries() {
        let mut block = EdidBlock::new_default();
        let mut et3 = EstablishedTimings3::default();
        assert!(!et3.has_timing(EstablishedTiming3::Res1920x1200_60Hz));
        assert!(!et3.has_timing(EstablishedTiming3::Res1440x900_60HzRb));

        et3.set_timing(EstablishedTiming3::Res1920x1200_60Hz, true);
        et3.set_timing(EstablishedTiming3::Res1440x900_60HzRb, true);
        et3.set_timing(EstablishedTiming3::Res848x480_60Hz, true);

        assert!(et3.has_timing(EstablishedTiming3::Res1920x1200_60Hz));
        assert!(et3.has_timing(EstablishedTiming3::Res1440x900_60HzRb));
        assert!(et3.has_timing(EstablishedTiming3::Res848x480_60Hz));
        assert!(!et3.has_timing(EstablishedTiming3::Res1920x1440_60Hz));
        let descriptor = MonitorDescriptor::EstablishedTimings3(et3);
        block.set_monitor_descriptor(1, &descriptor).unwrap();
        assert_eq!(block.monitor_descriptor(1).unwrap(), Some(descriptor));
        assert_eq!(block.validate(), Ok(()));
    }

    #[test]
    fn cvt_3byte_timing_codes_roundtrips() {
        let mut block = EdidBlock::new_default();
        let cvt1 = Cvt3ByteTimingEntry::Active {
            addressable_lines: 1080,
            aspect_ratio: CvtAspectRatio::Ratio16x9,
            preferred_rate: CvtPreferredRate::Hz60Standard,
            supported_rates: CvtSupportedRates { raw: 0x09 }, // 60Hz standard (bit 3) & 60Hz RB (bit 0)
        };
        let cvt2 = Cvt3ByteTimingEntry::Active {
            addressable_lines: 900,
            aspect_ratio: CvtAspectRatio::Ratio16x10,
            preferred_rate: CvtPreferredRate::Hz75Standard,
            supported_rates: CvtSupportedRates { raw: 0x04 }, // 75Hz standard (bit 2)
        };
        let descriptor = MonitorDescriptor::Cvt3ByteTimings([
            cvt1,
            cvt2,
            Cvt3ByteTimingEntry::Unused,
            Cvt3ByteTimingEntry::Unused,
        ]);
        block.set_monitor_descriptor(0, &descriptor).unwrap();
        assert_eq!(block.monitor_descriptor(0).unwrap(), Some(descriptor));
        assert_eq!(block.validate(), Ok(()));

        if let Some(MonitorDescriptor::Cvt3ByteTimings(slots)) =
            block.monitor_descriptor(0).unwrap()
        {
            if let Cvt3ByteTimingEntry::Active {
                addressable_lines,
                aspect_ratio,
                preferred_rate,
                supported_rates,
            } = slots[0]
            {
                assert_eq!(addressable_lines, 1080);
                assert_eq!(aspect_ratio, CvtAspectRatio::Ratio16x9);
                assert_eq!(preferred_rate, CvtPreferredRate::Hz60Standard);
                assert!(supported_rates.supports_60hz_standard());
                assert!(supported_rates.supports_60hz_reduced_blanking());
                assert!(!supported_rates.supports_50hz_standard());
            } else {
                panic!("expected active CVT slot 0");
            }
        }
    }

    #[test]
    fn cvt_3byte_timing_edid_decode_vector() {
        // Verified against edid-decode: 1080 lines (1080/2 - 1 = 539 = 0x21B), 16:9 (code 01b = 0x04),
        // preferred 60Hz standard (code 001b = 0x20), supported 60Hz std & 60Hz RB (0x08 | 0x01 = 0x09).
        // Byte 0 = 0x1B, Byte 1 = (0x02 << 4) | (0x01 << 2) = 0x24, Byte 2 = 0x20 | 0x09 = 0x29
        let bytes = [0x1B, 0x24, 0x29];
        let entry = Cvt3ByteTimingEntry::decode(bytes);
        if let Cvt3ByteTimingEntry::Active {
            addressable_lines,
            aspect_ratio,
            preferred_rate,
            supported_rates,
        } = entry
        {
            assert_eq!(addressable_lines, 1080);
            assert_eq!(aspect_ratio, CvtAspectRatio::Ratio16x9);
            assert_eq!(preferred_rate, CvtPreferredRate::Hz60Standard);
            assert!(supported_rates.supports_60hz_standard());
            assert!(supported_rates.supports_60hz_reduced_blanking());
            assert!(!supported_rates.supports_50hz_standard());
            assert!(!supported_rates.supports_75hz_standard());
        } else {
            panic!("expected active CVT timing");
        }
        assert_eq!(entry.encode().unwrap(), bytes);
    }

    #[test]
    fn color_management_descriptor_roundtrips() {
        let mut block = EdidBlock::new_default();
        let dcm = ColorManagementDescriptor {
            revision: 3,
            red_a3: 0x1234,
            red_a2: 0x5678,
            green_a3: 0x9ABC,
            green_a2: 0xDEF0,
            blue_a3: 0x1357,
            blue_a2: 0x2468,
        };
        let descriptor = MonitorDescriptor::ColorManagement(dcm);
        block.set_monitor_descriptor(2, &descriptor).unwrap();
        assert_eq!(block.monitor_descriptor(2).unwrap(), Some(descriptor));
        assert_eq!(block.validate(), Ok(()));
    }

    #[test]
    fn range_limits_extensions_secondary_gtf_and_cvt_roundtrip() {
        let mut block = EdidBlock::new_default();

        // 1. Secondary GTF curve with start_horizontal_frequency_khz, parameter_c, slope_m, offset_k, scaling_j
        let gtf_range = MonitorDescriptor::RangeLimits {
            min_vertical_hz: 50,
            max_vertical_hz: 120,
            min_horizontal_khz: 31,
            max_horizontal_khz: 135,
            max_pixel_clock_mhz: 300,
            extension: RangeLimitsExtension::SecondaryGtf(SecondaryGtfParameters {
                start_horizontal_frequency_khz: 80,
                parameter_c: 80, // C = 40%
                slope_m: 600,
                offset_k: 40,
                scaling_j: 20, // J = 10%
            }),
        };
        block.set_monitor_descriptor(1, &gtf_range).unwrap();
        assert_eq!(block.monitor_descriptor(1).unwrap(), Some(gtf_range));
        assert_eq!(block.validate(), Ok(()));

        // 2. CVT range support
        let cvt_range = MonitorDescriptor::RangeLimits {
            min_vertical_hz: 48,
            max_vertical_hz: 165,
            min_horizontal_khz: 30,
            max_horizontal_khz: 250,
            max_pixel_clock_mhz: 650,
            extension: RangeLimitsExtension::Cvt(CvtRangeSupport {
                revision: 0x11,
                max_pixel_clock_precision: 0x10,
                max_active_pixels: 2560,
                supported_aspect_ratios: 0xC0,
                preferred_aspect_ratio_and_flags: 0x28,
                scaling_support: 0x00,
                preferred_vertical_rate_hz: 144,
            }),
        };
        block.set_monitor_descriptor(2, &cvt_range).unwrap();
        assert_eq!(block.monitor_descriptor(2).unwrap(), Some(cvt_range));
        assert_eq!(block.validate(), Ok(()));
    }

    #[test]
    fn descriptor_validation_and_revisions() {
        let mut block = EdidBlock::new_default();

        // Invalid revision for Established Timings III (must be 0x0A)
        let invalid_et3 = MonitorDescriptor::EstablishedTimings3(EstablishedTimings3 {
            revision: 0x09,
            raw: [0; 6],
        });
        assert!(matches!(
            block.set_monitor_descriptor(0, &invalid_et3),
            Err(DescriptorError::InvalidRevision {
                tag: 0xF7,
                revision: 0x09
            })
        ));

        // Invalid revision for DCM (must be 0x03)
        let invalid_dcm = MonitorDescriptor::ColorManagement(ColorManagementDescriptor {
            revision: 0x01,
            red_a3: 0,
            red_a2: 0,
            green_a3: 0,
            green_a2: 0,
            blue_a3: 0,
            blue_a2: 0,
        });
        assert!(matches!(
            block.set_monitor_descriptor(0, &invalid_dcm),
            Err(DescriptorError::InvalidRevision {
                tag: 0xF9,
                revision: 0x01
            })
        ));

        // CVT Range Limits unrepresentable max_active_pixels (not multiple of 8 or > 8184)
        let invalid_cvt_pixels = MonitorDescriptor::RangeLimits {
            min_vertical_hz: 48,
            max_vertical_hz: 144,
            min_horizontal_khz: 30,
            max_horizontal_khz: 200,
            max_pixel_clock_mhz: 500,
            extension: RangeLimitsExtension::Cvt(CvtRangeSupport {
                revision: 0x11,
                max_pixel_clock_precision: 0,
                max_active_pixels: 8193,
                supported_aspect_ratios: 0,
                preferred_aspect_ratio_and_flags: 0,
                scaling_support: 0,
                preferred_vertical_rate_hz: 60,
            }),
        };
        assert_eq!(
            block.set_monitor_descriptor(0, &invalid_cvt_pixels),
            Err(DescriptorError::InvalidRangeExtension)
        );

        // Secondary GTF unrepresentable start frequency (must be even <= 510)
        let invalid_gtf = MonitorDescriptor::RangeLimits {
            min_vertical_hz: 48,
            max_vertical_hz: 144,
            min_horizontal_khz: 30,
            max_horizontal_khz: 200,
            max_pixel_clock_mhz: 500,
            extension: RangeLimitsExtension::SecondaryGtf(SecondaryGtfParameters {
                start_horizontal_frequency_khz: 511, // Odd
                parameter_c: 80,
                slope_m: 600,
                offset_k: 40,
                scaling_j: 20,
            }),
        };
        assert_eq!(
            block.set_monitor_descriptor(0, &invalid_gtf),
            Err(DescriptorError::InvalidRangeExtension)
        );
    }
}
