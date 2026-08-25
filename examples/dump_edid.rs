//! Dump information from an EDID binary or hex string.
//!
//! Run with:
//! ```bash
//! cargo run --example dump_edid
//! ```

use edid_seria::{
    CtaDataBlockView, CtaExtendedDataBlockView, CtaVendorSpecificBlock, Edid, EdidBlock,
    EstablishedTiming3, EstablishedTimings3, MonitorDescriptor,
};

fn main() {
    // A sample 2-block EDID (Base block + CTA-861 extension)
    let mut base = EdidBlock::new_default();
    base.raw[126] = 1; // 1 extension
    base.set_monitor_descriptor(1, &MonitorDescriptor::ProductName("SAM-27QHD".to_owned()))
        .unwrap();
    let mut et3 = EstablishedTimings3::default();
    et3.set_timing(EstablishedTiming3::Res1920x1200_60Hz, true);
    base.set_monitor_descriptor(2, &MonitorDescriptor::EstablishedTimings3(et3))
        .unwrap();
    base.update_checksum();
    let mut cta = EdidBlock::new_default();
    cta.raw[0] = 0x02; // CTA-861 extension tag
    cta.raw[1] = 0x03; // Revision 3
    cta.raw[2] = 0x07; // DTD offset at byte 7
    cta.raw[3] = 0xF1; // Underscan, Basic Audio, YCbCr 444/422, Native DTD count = 1

    // Data Block Collection (byte 4..7):
    // 1. Tag 2 Video block (len 1): VIC 16 (1080p60) -> header 0x41, vic 0x10
    cta.raw[4] = 0x41;
    cta.raw[5] = 0x10;
    // 2. Tag 4 Speaker Allocation (len 1): FL/FR -> header 0x81, mask 0x01
    cta.raw[6] = 0x81;
    cta.raw[7] = 0x01;
    cta.raw[2] = 8; // DTDs at offset 8

    // DTD at byte 8..26: 1080p60 (1920x1080 @ 60Hz)
    cta.raw[8] = 0x02;
    cta.raw[9] = 0x3A;
    cta.raw[10] = 0x80;
    cta.raw[11] = 0x18;
    cta.raw[12] = 0x71;
    cta.raw[13] = 0x38;
    cta.raw[14] = 0x2D;
    cta.raw[15] = 0x40;
    cta.raw[16] = 0x58;
    cta.raw[17] = 0x2C;
    cta.raw[18] = 0x45;
    cta.raw[25] = 0x1E;
    cta.update_checksum();

    let mut edid_bytes = base.as_bytes().to_vec();
    edid_bytes.extend_from_slice(cta.as_bytes());

    let edid = Edid::from_bytes(&edid_bytes).expect("valid EDID");

    println!("================ EDID Summary ================");
    println!("Total Blocks: {}", 1 + edid.extensions.len());
    if let Some(name) = edid.monitor_name() {
        println!("Monitor Name: {name}");
    }
    if let Some(serial) = edid.serial_number() {
        println!("Serial Number: {serial}");
    }
    if let Some(preferred) = edid.preferred_timing() {
        println!("Preferred Timing: {}", preferred.label());
    }

    println!("\nAll Detailed Timings (Base + CTA-861):");
    for (i, timing) in edid.all_detailed_timings().iter().enumerate() {
        println!(
            "  [{}] {} (clock: {:.2} MHz, H: {}/{}/{}/{}, V: {}/{}/{}/{})",
            i + 1,
            timing.label(),
            timing.pixel_clock_khz as f64 / 1000.0,
            timing.h_active,
            timing.h_front,
            timing.h_sync,
            timing.h_back,
            timing.v_active,
            timing.v_front,
            timing.v_sync,
            timing.v_back,
        );
    }

    println!("\nBase Block Monitor Descriptors:");
    for slot in 0..4 {
        match edid.base.monitor_descriptor(slot) {
            Ok(Some(desc)) => match desc {
                MonitorDescriptor::ProductName(name) => {
                    println!("  [Slot {slot}] Product Name: \"{name}\"");
                }
                MonitorDescriptor::SerialNumber(serial) => {
                    println!("  [Slot {slot}] Serial Number: \"{serial}\"");
                }
                MonitorDescriptor::AlphanumericString(text) => {
                    println!("  [Slot {slot}] Alphanumeric String: \"{text}\"");
                }
                MonitorDescriptor::EstablishedTimings3(et3) => {
                    println!(
                        "  [Slot {slot}] Established Timings III (rev {:#04X}): 1920x1200@60Hz={}",
                        et3.revision,
                        et3.has_timing(EstablishedTiming3::Res1920x1200_60Hz)
                    );
                }
                MonitorDescriptor::Cvt3ByteTimings(cvts) => {
                    println!(
                        "  [Slot {slot}] CVT 3-Byte Timing Codes: {} slots",
                        cvts.len()
                    );
                }
                MonitorDescriptor::ColorManagement(dcm) => {
                    println!(
                        "  [Slot {slot}] Color Management (rev {:#04X})",
                        dcm.revision
                    );
                }
                MonitorDescriptor::RangeLimits {
                    min_vertical_hz,
                    max_vertical_hz,
                    min_horizontal_khz,
                    max_horizontal_khz,
                    max_pixel_clock_mhz,
                    extension,
                } => {
                    println!(
                        "  [Slot {slot}] Range Limits: V: {min_vertical_hz}-{max_vertical_hz}Hz, H: {min_horizontal_khz}-{max_horizontal_khz}kHz, Clock: {max_pixel_clock_mhz}MHz, Ext: {extension:?}"
                    );
                }
                MonitorDescriptor::AdditionalColorPoint { point1, point2 } => {
                    println!(
                        "  [Slot {slot}] Additional Color Point: #{} (gamma: {:?}), second: {}",
                        point1.index,
                        point1.gamma,
                        point2.is_some()
                    );
                }
                MonitorDescriptor::AdditionalStandardTimings(timings) => {
                    println!(
                        "  [Slot {slot}] Additional Standard Timings: {} entries",
                        timings.len()
                    );
                }
                MonitorDescriptor::Dummy => {
                    println!("  [Slot {slot}] Dummy Descriptor");
                }
                MonitorDescriptor::Unknown { tag, payload } => {
                    println!(
                        "  [Slot {slot}] Unknown Descriptor: Tag {tag:#04X}, len {}",
                        payload.len()
                    );
                }
            },
            Ok(None) => {
                println!("  [Slot {slot}] <Timing Slot or Unused>");
            }
            Err(err) => {
                println!("  [Slot {slot}] Error: {err}");
            }
        }
    }

    for (idx, ext) in edid.extensions.iter().enumerate() {
        println!("\nExtension Block #{}: {:?}", idx + 1, ext.extension_kind());
        if let Ok(cta_hdr) = ext.cta_header() {
            println!("  CTA Revision: {}", cta_hdr.revision);
            println!("  Native DTDs: {}", cta_hdr.native_dtd_count);
            println!("  Underscan: {}", cta_hdr.underscan);
            println!("  Basic Audio: {}", cta_hdr.basic_audio);
            println!("  YCbCr 4:4:4: {}", cta_hdr.ycbcr_444);
            println!("  YCbCr 4:2:2: {}", cta_hdr.ycbcr_422);
        }
        if let Ok(blocks) = ext.cta_data_blocks() {
            println!("  CTA Data Blocks:");
            for (b_idx, db) in blocks.iter().enumerate() {
                match db.view() {
                    Ok(CtaDataBlockView::Video { modes }) => {
                        println!(
                            "    [{}] Video Data Block: {} mode(s)",
                            b_idx + 1,
                            modes.len()
                        );
                    }
                    Ok(CtaDataBlockView::Audio { descriptors }) => {
                        println!(
                            "    [{}] Audio Data Block: {} descriptor(s)",
                            b_idx + 1,
                            descriptors.len()
                        );
                    }
                    Ok(CtaDataBlockView::SpeakerAllocation(spk)) => {
                        println!(
                            "    [{}] Speaker Allocation: FL/FR={}, LFE={}, FC={}",
                            b_idx + 1,
                            spk.front_left_right(),
                            spk.lfe(),
                            spk.front_center()
                        );
                    }
                    Ok(CtaDataBlockView::VendorSpecific(CtaVendorSpecificBlock::AmdFreeSync {
                        min_refresh_hz,
                        max_refresh_hz,
                        ..
                    })) => {
                        println!(
                            "    [{}] AMD FreeSync VSDB: {:?}-{:?} Hz",
                            b_idx + 1,
                            min_refresh_hz,
                            max_refresh_hz
                        );
                    }
                    Ok(CtaDataBlockView::VendorSpecific(vsdb)) => {
                        println!("    [{}] Vendor-Specific: {vsdb:?}", b_idx + 1);
                    }
                    Ok(CtaDataBlockView::Extended(CtaExtendedDataBlockView::Colorimetry(c))) => {
                        println!(
                            "    [{}] Colorimetry: BT2020_RGB={}",
                            b_idx + 1,
                            c.bt2020_rgb
                        );
                    }
                    Ok(CtaDataBlockView::Extended(ext_view)) => {
                        println!("    [{}] Extended CTA: {ext_view:?}", b_idx + 1);
                    }
                    Ok(CtaDataBlockView::Unknown { tag, payload }) => {
                        println!(
                            "    [{}] Unknown CTA tag {tag}: {} bytes",
                            b_idx + 1,
                            payload.len()
                        );
                    }
                    Err(e) => {
                        println!("    [{}] Parsing error: {e}", b_idx + 1);
                    }
                }
            }
        }
        if let Ok(cta_timings) = ext.cta_detailed_timings() {
            for (t_idx, t) in cta_timings.iter().enumerate() {
                if t_idx == 0 {
                    println!("  CTA Detailed Timings:");
                }
                println!("    [{}] {}", t_idx + 1, t.label());
            }
        }
    }
}
