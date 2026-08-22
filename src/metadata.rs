//! Base-block metadata and monitor descriptor access.

use crate::edid::{DETAILED_START, EdidBlock};
use crate::error::{DescriptorError, MetadataError};

const EDID_HEADER: [u8; 8] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];
const DESCRIPTOR_LEN: usize = 18;
const DESCRIPTOR_SLOTS: usize = 4;
const TEXT_PAYLOAD_LEN: usize = 13;

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
    use super::{BaseMetadata, MonitorDescriptor};
    use crate::edid::EdidBlock;

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
}
