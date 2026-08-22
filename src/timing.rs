//! Detailed timing model: the [`DetailedTiming`] type, PC/HDTV preset
//! tables, CVT/CVT-RB/CVT-RB2 timing computation, and DTD field limits.

/// A complete progressive display timing, as held in an EDID DTD or
/// computed by the CVT formulas.
#[derive(Clone, Debug, PartialEq)]
pub struct DetailedTiming {
    /// Horizontal active pixels.
    pub h_active: u32,
    /// Vertical active lines.
    pub v_active: u32,
    /// Horizontal front porch (pixels).
    pub h_front: u32,
    /// Horizontal sync pulse width (pixels).
    pub h_sync: u32,
    /// Horizontal back porch (pixels).
    pub h_back: u32,
    /// Vertical front porch (lines).
    pub v_front: u32,
    /// Vertical sync pulse width (lines).
    pub v_sync: u32,
    /// Vertical back porch (lines).
    pub v_back: u32,
    /// Horizontal border in pixels (DTD byte 15); blanking includes 2× this.
    pub h_border: u32,
    /// Vertical border in lines (DTD byte 16); blanking includes 2× this.
    pub v_border: u32,
    /// Pixel clock in kHz.
    pub pixel_clock_khz: u32,
    /// Horizontal sync polarity (`true` = positive).
    pub h_pol: bool,
    /// Vertical sync polarity (`true` = positive).
    pub v_pol: bool,
    /// Refresh rate in Hz, as computed from clock and totals.
    pub v_rate: f64,
}

impl DetailedTiming {
    /// Horizontal active pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.h_active
    }
    /// Vertical active lines.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.v_active
    }

    /// Human-readable label, e.g. `"1920x1080 @ 60Hz"`.
    #[must_use]
    pub fn label(&self) -> String {
        format!("{}x{} @ {:.0}Hz", self.h_active, self.v_active, self.v_rate)
    }

    /// Look up a standard preset timing by resolution and refresh rate
    /// (within 0.5 Hz); returns `None` when no preset matches.
    #[must_use]
    pub fn compute_blanking(width: u32, height: u32, refresh: f64) -> Option<DetailedTiming> {
        let presets = all_presets();
        presets
            .iter()
            .find(|p| {
                p.h_active == width && p.v_active == height && (p.v_rate - refresh).abs() < 0.5
            })
            .cloned()
    }

    /// Look up a standard HDTV/CEA timing without considering PC presets.
    #[must_use]
    pub fn compute_hdtv_blanking(width: u32, height: u32, refresh: f64) -> Option<DetailedTiming> {
        hdtv_presets().into_iter().find(|preset| {
            preset.h_active == width
                && preset.v_active == height
                && (preset.v_rate - refresh).abs() < 0.5
        })
    }
}

/// EDID DTD field limits (E-EDID 1.4 §3.10.2).
///
/// Unlike [`dtd_fits`], this function reports the first invalid field and uses
/// checked arithmetic for blanking totals. Pixel clocks below 10 MHz are
/// rejected because the EDID reader treats them as invalid timing data.
pub fn validate_dtd(t: &DetailedTiming) -> Result<(), crate::error::DtdError> {
    use crate::error::{DtdError, DtdField};

    fn limit(field: DtdField, value: u32, max: u32) -> Result<(), DtdError> {
        if value > max {
            Err(DtdError::FieldOutOfRange { field, value, max })
        } else {
            Ok(())
        }
    }

    fn positive(field: DtdField, value: u32, max: u32) -> Result<(), DtdError> {
        if value == 0 {
            Err(DtdError::InvalidField { field, value })
        } else {
            limit(field, value, max)
        }
    }

    positive(DtdField::HorizontalActive, t.h_active, 4095)?;
    positive(DtdField::VerticalActive, t.v_active, 4095)?;
    limit(DtdField::HorizontalFrontPorch, t.h_front, 1023)?;
    limit(DtdField::HorizontalSync, t.h_sync, 1023)?;
    limit(DtdField::VerticalFrontPorch, t.v_front, 63)?;
    limit(DtdField::VerticalSync, t.v_sync, 63)?;
    limit(DtdField::HorizontalBorder, t.h_border, 255)?;
    limit(DtdField::VerticalBorder, t.v_border, 255)?;
    if t.pixel_clock_khz < 10_000 {
        return Err(DtdError::InvalidField {
            field: DtdField::PixelClockKHz,
            value: t.pixel_clock_khz,
        });
    }
    limit(DtdField::PixelClockKHz, t.pixel_clock_khz, 655_350)?;

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
    limit(DtdField::HorizontalBlanking, h_blank, 4095)?;

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
    limit(DtdField::VerticalBlanking, v_blank, 4095)?;
    Ok(())
}

/// Determine whether a timing fits all EDID DTD fields.
#[must_use]
pub fn dtd_fits(t: &DetailedTiming) -> bool {
    validate_dtd(t).is_ok()
}
fn pc_presets() -> Vec<DetailedTiming> {
    vec![
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
        },
        DetailedTiming {
            h_active: 1920,
            v_active: 1080,
            h_front: 528,
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
            v_rate: 50.0,
        },
        DetailedTiming {
            h_active: 1920,
            v_active: 1080,
            h_front: 638,
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
            v_rate: 48.0,
        },
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
            pixel_clock_khz: 74250,
            h_pol: true,
            v_pol: true,
            v_rate: 30.0,
        },
        DetailedTiming {
            h_active: 1920,
            v_active: 1080,
            h_front: 528,
            h_sync: 44,
            h_back: 148,
            v_front: 4,
            v_sync: 5,
            v_back: 36,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 74250,
            h_pol: true,
            v_pol: true,
            v_rate: 25.0,
        },
        DetailedTiming {
            h_active: 1920,
            v_active: 1080,
            h_front: 638,
            h_sync: 44,
            h_back: 148,
            v_front: 4,
            v_sync: 5,
            v_back: 36,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 74250,
            h_pol: true,
            v_pol: true,
            v_rate: 24.0,
        },
        DetailedTiming {
            h_active: 1600,
            v_active: 900,
            h_front: 24,
            h_sync: 80,
            h_back: 96,
            v_front: 1,
            v_sync: 3,
            v_back: 96,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 108000,
            h_pol: true,
            v_pol: true,
            v_rate: 60.0,
        },
        DetailedTiming {
            h_active: 1366,
            v_active: 768,
            h_front: 70,
            h_sync: 143,
            h_back: 213,
            v_front: 3,
            v_sync: 3,
            v_back: 24,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 85500,
            h_pol: true,
            v_pol: true,
            v_rate: 60.0,
        },
        DetailedTiming {
            h_active: 1360,
            v_active: 768,
            h_front: 64,
            h_sync: 112,
            h_back: 256,
            v_front: 3,
            v_sync: 6,
            v_back: 18,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 85500,
            h_pol: true,
            v_pol: true,
            v_rate: 60.0,
        },
        DetailedTiming {
            h_active: 1280,
            v_active: 720,
            h_front: 110,
            h_sync: 40,
            h_back: 220,
            v_front: 5,
            v_sync: 5,
            v_back: 20,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 74250,
            h_pol: true,
            v_pol: true,
            v_rate: 60.0,
        },
        DetailedTiming {
            h_active: 1280,
            v_active: 720,
            h_front: 440,
            h_sync: 40,
            h_back: 220,
            v_front: 5,
            v_sync: 5,
            v_back: 20,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 74250,
            h_pol: true,
            v_pol: true,
            v_rate: 50.0,
        },
        DetailedTiming {
            h_active: 1280,
            v_active: 720,
            h_front: 960,
            h_sync: 40,
            h_back: 220,
            v_front: 5,
            v_sync: 5,
            v_back: 20,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 90000,
            h_pol: true,
            v_pol: true,
            v_rate: 48.0,
        },
        DetailedTiming {
            h_active: 1280,
            v_active: 720,
            h_front: 1760,
            h_sync: 40,
            h_back: 220,
            v_front: 5,
            v_sync: 5,
            v_back: 20,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 74250,
            h_pol: true,
            v_pol: true,
            v_rate: 30.0,
        },
        DetailedTiming {
            h_active: 1280,
            v_active: 720,
            h_front: 2420,
            h_sync: 40,
            h_back: 220,
            v_front: 5,
            v_sync: 5,
            v_back: 20,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 74250,
            h_pol: true,
            v_pol: true,
            v_rate: 25.0,
        },
        DetailedTiming {
            h_active: 1280,
            v_active: 720,
            h_front: 1760,
            h_sync: 40,
            h_back: 220,
            v_front: 5,
            v_sync: 5,
            v_back: 20,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 59400,
            h_pol: true,
            v_pol: true,
            v_rate: 24.0,
        },
        DetailedTiming {
            h_active: 640,
            v_active: 480,
            h_front: 16,
            h_sync: 96,
            h_back: 48,
            v_front: 10,
            v_sync: 2,
            v_back: 33,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 25200,
            h_pol: false,
            v_pol: false,
            v_rate: 60.0,
        },
    ]
}

/// HDTV preset timings ported from CRU DetailedResolutionClass::AutomaticHDTV[]
/// (subset — 1080p to 4K common rates only; full table in C++ source)
fn hdtv_presets() -> Vec<DetailedTiming> {
    vec![
        // 3840x2160 (120/100 Hz need >655.35 MHz pixel clock — not representable in a DTD)
        DetailedTiming {
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
            pixel_clock_khz: 594000,
            h_pol: true,
            v_pol: true,
            v_rate: 60.0,
        },
        DetailedTiming {
            h_active: 3840,
            v_active: 2160,
            h_front: 1056,
            h_sync: 88,
            h_back: 296,
            v_front: 8,
            v_sync: 10,
            v_back: 72,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 594000,
            h_pol: true,
            v_pol: true,
            v_rate: 50.0,
        },
        DetailedTiming {
            h_active: 3840,
            v_active: 2160,
            h_front: 1276,
            h_sync: 88,
            h_back: 296,
            v_front: 8,
            v_sync: 10,
            v_back: 72,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 594000,
            h_pol: true,
            v_pol: true,
            v_rate: 48.0,
        },
        DetailedTiming {
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
            v_rate: 30.0,
        },
        DetailedTiming {
            h_active: 3840,
            v_active: 2160,
            h_front: 1056,
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
            v_rate: 25.0,
        },
        DetailedTiming {
            h_active: 3840,
            v_active: 2160,
            h_front: 1276,
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
            v_rate: 24.0,
        },
        // 1920x1080
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
            pixel_clock_khz: 297000,
            h_pol: true,
            v_pol: true,
            v_rate: 120.0,
        },
        DetailedTiming {
            h_active: 1920,
            v_active: 1080,
            h_front: 528,
            h_sync: 44,
            h_back: 148,
            v_front: 4,
            v_sync: 5,
            v_back: 36,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 297000,
            h_pol: true,
            v_pol: true,
            v_rate: 100.0,
        },
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
        },
        DetailedTiming {
            h_active: 1920,
            v_active: 1080,
            h_front: 528,
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
            v_rate: 50.0,
        },
        DetailedTiming {
            h_active: 1920,
            v_active: 1080,
            h_front: 638,
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
            v_rate: 48.0,
        },
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
            pixel_clock_khz: 74250,
            h_pol: true,
            v_pol: true,
            v_rate: 30.0,
        },
        DetailedTiming {
            h_active: 1920,
            v_active: 1080,
            h_front: 528,
            h_sync: 44,
            h_back: 148,
            v_front: 4,
            v_sync: 5,
            v_back: 36,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 74250,
            h_pol: true,
            v_pol: true,
            v_rate: 25.0,
        },
        DetailedTiming {
            h_active: 1920,
            v_active: 1080,
            h_front: 638,
            h_sync: 44,
            h_back: 148,
            v_front: 4,
            v_sync: 5,
            v_back: 36,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 74250,
            h_pol: true,
            v_pol: true,
            v_rate: 24.0,
        },
        // 1280x720
        DetailedTiming {
            h_active: 1280,
            v_active: 720,
            h_front: 110,
            h_sync: 40,
            h_back: 220,
            v_front: 5,
            v_sync: 5,
            v_back: 20,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 148500,
            h_pol: true,
            v_pol: true,
            v_rate: 120.0,
        },
        DetailedTiming {
            h_active: 1280,
            v_active: 720,
            h_front: 440,
            h_sync: 40,
            h_back: 220,
            v_front: 5,
            v_sync: 5,
            v_back: 20,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 148500,
            h_pol: true,
            v_pol: true,
            v_rate: 100.0,
        },
        DetailedTiming {
            h_active: 1280,
            v_active: 720,
            h_front: 110,
            h_sync: 40,
            h_back: 220,
            v_front: 5,
            v_sync: 5,
            v_back: 20,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 74250,
            h_pol: true,
            v_pol: true,
            v_rate: 60.0,
        },
        DetailedTiming {
            h_active: 1280,
            v_active: 720,
            h_front: 440,
            h_sync: 40,
            h_back: 220,
            v_front: 5,
            v_sync: 5,
            v_back: 20,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 74250,
            h_pol: true,
            v_pol: true,
            v_rate: 50.0,
        },
        DetailedTiming {
            h_active: 1280,
            v_active: 720,
            h_front: 960,
            h_sync: 40,
            h_back: 220,
            v_front: 5,
            v_sync: 5,
            v_back: 20,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 90000,
            h_pol: true,
            v_pol: true,
            v_rate: 48.0,
        },
        DetailedTiming {
            h_active: 1280,
            v_active: 720,
            h_front: 1760,
            h_sync: 40,
            h_back: 220,
            v_front: 5,
            v_sync: 5,
            v_back: 20,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 74250,
            h_pol: true,
            v_pol: true,
            v_rate: 30.0,
        },
        DetailedTiming {
            h_active: 1280,
            v_active: 720,
            h_front: 2420,
            h_sync: 40,
            h_back: 220,
            v_front: 5,
            v_sync: 5,
            v_back: 20,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 74250,
            h_pol: true,
            v_pol: true,
            v_rate: 25.0,
        },
        DetailedTiming {
            h_active: 1280,
            v_active: 720,
            h_front: 1760,
            h_sync: 40,
            h_back: 220,
            v_front: 5,
            v_sync: 5,
            v_back: 20,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 59400,
            h_pol: true,
            v_pol: true,
            v_rate: 24.0,
        },
    ]
}

fn common_wide_presets() -> Vec<DetailedTiming> {
    vec![
        DetailedTiming {
            h_active: 1280,
            v_active: 800,
            h_front: 48,
            h_sync: 32,
            h_back: 80,
            v_front: 3,
            v_sync: 6,
            v_back: 22,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 71800,
            h_pol: true,
            v_pol: false,
            v_rate: 60.0,
        },
        DetailedTiming {
            h_active: 1920,
            v_active: 1200,
            h_front: 48,
            h_sync: 32,
            h_back: 80,
            v_front: 3,
            v_sync: 6,
            v_back: 26,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 154130,
            h_pol: true,
            v_pol: false,
            v_rate: 60.0,
        },
        DetailedTiming {
            h_active: 2560,
            v_active: 1440,
            h_front: 48,
            h_sync: 32,
            h_back: 80,
            v_front: 3,
            v_sync: 5,
            v_back: 33,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 241700,
            h_pol: true,
            v_pol: false,
            v_rate: 60.0,
        },
        DetailedTiming {
            h_active: 3440,
            v_active: 1440,
            h_front: 48,
            h_sync: 32,
            h_back: 80,
            v_front: 3,
            v_sync: 5,
            v_back: 33,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 319900,
            h_pol: true,
            v_pol: false,
            v_rate: 60.0,
        },
    ]
}

/// All PC and HDTV preset timings, built once and returned as a static slice.
#[must_use]
pub fn all_presets() -> &'static [DetailedTiming] {
    use std::sync::LazyLock;
    static PRESETS: LazyLock<Vec<DetailedTiming>> = LazyLock::new(|| {
        let mut v = pc_presets();
        v.extend(common_wide_presets());
        v.extend(hdtv_presets());
        v
    });
    &PRESETS
}

// ---------------------------------------------------------------------------
// CVT / CVT-RB / CVT-RB2 timing computation
// Ported from cvt12.c: Coordinated Video Timings v1.1 and v1.2
// Reference: https://github.com/kevinlekiller/cvt_modeline_calculator_12
// ---------------------------------------------------------------------------

/// CVT 1.1 constants
const CLOCK_STEP_V1: f64 = 0.250;
const CELL_GRAN_V1: f64 = 8.0;
const MIN_V_BPORCH_V1: f64 = 6.0;
const MIN_V_PORCH: f64 = 3.0;
const H_SYNC_PER: f64 = 8.0;
const M: f64 = 600.0;
const C: f64 = 40.0;
const K: f64 = 128.0;
const J: f64 = 20.0;
const MIN_VSYNC_BP: f64 = 550.0; // µs

// CVT 1.1 Reduced Blanking
const RB_MIN_V_BPORCH: f64 = 6.0;
const RB_MIN_V_BLANK: f64 = 460.0; // µs
const RB_H_SYNC: u32 = 32;
const RB_V1_H_BLANK: f64 = 160.0;
const RB_V1_V_FPORCH: f64 = 3.0;

// CVT 1.2 (RB2) overrides
const CLOCK_STEP_V2: f64 = 0.001;
const CELL_GRAN_V2: f64 = 1.0;
const MIN_V_BPORCH_V2: f64 = 6.0;
const RB_V2_V_FPORCH: f64 = 1.0;
const RB_V2_H_BLANK: f64 = 80.0;
const RB_V2_V_SYNC: f64 = 8.0;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// Which CVT variant to use for timing computation.
pub enum TimingFormula {
    /// Do not compute: used to mark timings entered by hand.
    Manual,
    /// CVT 1.1 with normal blanking.
    CVT,
    /// CVT 1.1 with reduced blanking.
    CVTRB,
    /// CVT 1.2 with reduced blanking v2 (recommended for PC monitors).
    CVTRB2,
}

/// Compute timing via the CVT formula.
///
/// Returns `None` if the requested parameters are out of range
/// (`freq` must be in (0, 1000] Hz, or the formula degenerates).
#[must_use]
pub fn compute_cvt(
    h_pixels: u32,
    v_lines: u32,
    freq: f64,
    formula: TimingFormula,
) -> Option<DetailedTiming> {
    if h_pixels == 0 || v_lines == 0 || !freq.is_finite() || freq <= 0.0 || freq > 1000.0 {
        return None;
    }
    let reduced = match formula {
        TimingFormula::Manual => return None,
        TimingFormula::CVT => 0,
        TimingFormula::CVTRB => 1,
        TimingFormula::CVTRB2 => 2,
    };
    let (clock_step, cell_gran, min_v_bporch, rb_v_fporch, rb_h_blank, v_sync_fixed) =
        if reduced == 2 {
            (
                CLOCK_STEP_V2,
                CELL_GRAN_V2,
                MIN_V_BPORCH_V2,
                RB_V2_V_FPORCH,
                RB_V2_H_BLANK,
                true,
            )
        } else {
            (
                CLOCK_STEP_V1,
                CELL_GRAN_V1,
                MIN_V_BPORCH_V1,
                RB_V1_V_FPORCH,
                RB_V1_H_BLANK,
                false,
            )
        };

    let v_field_rate_rqd = freq;
    let h_pixels_rnd = (h_pixels as f64 / cell_gran).floor() * cell_gran;

    // Determine aspect ratio → vsync lines (per CVT table)
    let v_sync = if v_sync_fixed {
        RB_V2_V_SYNC
    } else {
        vsync_lines(h_pixels_rnd as u32, v_lines, cell_gran)
    };
    let v_sync_rnd = v_sync;

    // Margins (disabled for CRU use case)
    let left_margin = 0.0;
    let right_margin = 0.0;
    let total_active_pixels = h_pixels_rnd + left_margin + right_margin;

    let v_lines_rnd = v_lines as f64;
    let top_margin = 0.0;
    let bot_margin = 0.0;

    let (h_blank, total_v_lines, total_pixels, act_pixel_freq, _h_period_est, v_front_porch) =
        if reduced != 0 {
            let h_blank = rb_h_blank;
            let h_period_est = (1_000_000.0 / v_field_rate_rqd - RB_MIN_V_BLANK)
                / (v_lines_rnd + top_margin + bot_margin);
            let vbi_lines = (RB_MIN_V_BLANK / h_period_est).floor() + 1.0;
            let rb_min_vbi = rb_v_fporch + v_sync_rnd + RB_MIN_V_BPORCH;
            let act_vbi_lines = if vbi_lines < rb_min_vbi {
                rb_min_vbi
            } else {
                vbi_lines
            };
            let total_v_lines = act_vbi_lines + v_lines_rnd + top_margin + bot_margin;
            let total_pixels = total_active_pixels + rb_h_blank;

            let mut act_pixel_freq = v_field_rate_rqd * total_v_lines * total_pixels / 1_000_000.0;
            act_pixel_freq = clock_step * (act_pixel_freq / clock_step).floor();

            let v_front = if reduced == 2 {
                act_vbi_lines - RB_MIN_V_BPORCH - v_sync_rnd
            } else {
                RB_V1_V_FPORCH
            };

            (
                h_blank,
                total_v_lines,
                total_pixels,
                act_pixel_freq,
                h_period_est,
                v_front,
            )
        } else {
            // --- normal blanking (CVT 1.1) ---
            let h_period_est = ((1.0 / v_field_rate_rqd) - MIN_VSYNC_BP / 1_000_000.0)
                / (v_lines_rnd + 2.0 * top_margin + MIN_V_PORCH)
                * 1_000_000.0;
            let v_sync_bp = (MIN_VSYNC_BP / h_period_est).floor() + 1.0;
            let v_sync_bp = if v_sync_bp < v_sync + min_v_bporch {
                v_sync + min_v_bporch
            } else {
                v_sync_bp
            };
            let total_v_lines = v_lines_rnd + top_margin + bot_margin + v_sync_bp + MIN_V_PORCH;

            let c_prime = ((C - J) * K / 256.0) + J;
            let m_prime = M * K / 256.0;
            let ideal_duty = c_prime - m_prime * h_period_est / 1000.0;
            let cur_duty = if ideal_duty < 20.0 { 20.0 } else { ideal_duty };
            let h_blank = (total_active_pixels * cur_duty / (100.0 - cur_duty) / (2.0 * cell_gran))
                .floor()
                * (2.0 * cell_gran);
            let total_pixels = total_active_pixels + h_blank;

            let mut act_pixel_freq = total_pixels / h_period_est;
            act_pixel_freq = clock_step * (act_pixel_freq / clock_step).floor();

            (
                h_blank,
                total_v_lines,
                total_pixels,
                act_pixel_freq,
                h_period_est,
                MIN_V_PORCH,
            )
        };

    // H-sync rounding
    let h_sync_rnd = if reduced != 0 {
        RB_H_SYNC as f64
    } else {
        (H_SYNC_PER / 100.0 * total_pixels / cell_gran).floor() * cell_gran
    };

    let h_back_porch = h_blank / 2.0;
    let h_front_porch = h_blank - h_back_porch - h_sync_rnd;

    let v_back_porch =
        total_v_lines - v_lines_rnd - v_front_porch - v_sync_rnd - top_margin - bot_margin;

    // Polarity: CVT = -H/+V, reduced blanking = +H/-V
    let h_pol = reduced != 0;
    let v_pol = reduced == 0;

    Some(DetailedTiming {
        h_active: h_pixels_rnd as u32,
        v_active: v_lines,
        h_front: h_front_porch as u32,
        h_sync: h_sync_rnd as u32,
        h_back: h_back_porch as u32,
        v_front: v_front_porch as u32,
        v_sync: v_sync_rnd as u32,
        v_back: v_back_porch as u32,
        h_border: 0,
        v_border: 0,
        pixel_clock_khz: (act_pixel_freq * 1000.0) as u32,
        h_pol,
        v_pol,
        v_rate: freq,
    })
}

/// Determine VSync width based on aspect ratio (CVT 1.1 table)
fn vsync_lines(h_px: u32, v_lines: u32, cell_gran: f64) -> f64 {
    let v = v_lines as f64;
    let ratios: [(f64, f64); 5] = [
        (4.0 / 3.0, 4.0),
        (16.0 / 9.0, 5.0),
        (16.0 / 10.0, 6.0),
        (5.0 / 4.0, 7.0),
        (15.0 / 9.0, 7.0),
    ];
    for &(ratio, vsync) in &ratios {
        let expected = cell_gran * (v * ratio / cell_gran).floor();
        if (h_px as f64 - expected).abs() < 0.5 {
            return vsync;
        }
    }
    10.0 // unknown aspect ratio → default
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pc_presets_are_valid() {
        for p in pc_presets() {
            assert!(p.h_active > 0);
            assert!(p.v_active > 0);
            assert!(p.v_rate > 0.0);
        }
    }

    #[test]
    fn hdtv_presets_are_valid() {
        for p in hdtv_presets() {
            assert!(p.h_active > 0);
            assert!(p.v_active > 0);
            assert!(p.v_rate > 0.0);
        }
    }

    /// Every preset's pixel clock must actually produce its labeled refresh rate,
    /// and must be representable in an EDID DTD (12-bit active, <= 655.35 MHz).
    #[test]
    fn preset_clocks_match_rates() {
        for p in all_presets() {
            let h_total = p.h_active + p.h_front + p.h_sync + p.h_back;
            let v_total = p.v_active + p.v_front + p.v_sync + p.v_back;
            let actual = p.pixel_clock_khz as f64 * 1000.0 / (h_total as f64 * v_total as f64);
            assert!(
                (actual - p.v_rate).abs() < 0.5,
                "{}x{}@{}: clock gives {:.2} Hz",
                p.h_active,
                p.v_active,
                p.v_rate,
                actual
            );
            assert!(p.h_active <= 4095 && p.v_active <= 4095);
            assert!(p.pixel_clock_khz <= 655_350);
        }
    }

    #[test]
    fn cvt_1080p60_normal() {
        let t = compute_cvt(1920, 1080, 60.0, TimingFormula::CVT).unwrap();
        assert_eq!(t.h_active, 1920);
        assert_eq!(t.v_active, 1080);
        assert!(!t.h_pol); // CVT = negative hsync
        assert!(t.v_pol); // CVT = positive vsync
        assert!(t.h_sync > 0);
        assert!(t.h_front > 0);
        assert!(t.h_back > 0);
        assert!(t.v_sync > 0);
        assert!(t.v_front > 0);
        assert!(t.v_back > 0);
        let h_total = t.h_active + t.h_front + t.h_sync + t.h_back;

        let v_total = t.v_active + t.v_front + t.v_sync + t.v_back;
        assert!(h_total > t.h_active);
        assert!(v_total > t.v_active);
        assert!((t.v_rate - 60.0).abs() < 0.5);
    }
    #[test]
    fn common_wide_and_high_resolution_presets_are_available() {
        for &(width, height, refresh) in &[
            (1280, 800, 60.0),
            (1920, 1200, 60.0),
            (2560, 1440, 60.0),
            (3440, 1440, 60.0),
        ] {
            assert!(
                DetailedTiming::compute_blanking(width, height, refresh).is_some(),
                "missing preset {width}x{height}@{refresh}"
            );
        }
    }

    #[test]
    fn cvt_1080p60_rb() {
        let t = compute_cvt(1920, 1080, 60.0, TimingFormula::CVTRB).unwrap();
        assert_eq!(t.h_active, 1920);
        assert_eq!(t.v_active, 1080);
        assert!(t.h_pol); // RB = positive hsync
        assert!(!t.v_pol); // RB = negative vsync
        assert_eq!(t.h_sync, 32); // RB always 32
    }

    #[test]
    fn cvt_2560x1440_144_rb2() {
        let t = compute_cvt(2560, 1440, 144.0, TimingFormula::CVTRB2).unwrap();
        assert_eq!(t.h_active, 2560);
        assert_eq!(t.v_active, 1440);
        assert_eq!(t.h_sync, 32); // RB2 sync = 32
        assert!(t.h_pol);
        assert!(!t.v_pol);
        assert!((t.v_rate - 144.0).abs() < 1.0);
    }

    #[test]
    fn cvt_rejects_manual() {
        assert!(compute_cvt(1920, 1080, 60.0, TimingFormula::Manual).is_none());
    }

    #[test]
    fn cvt_rejects_zero() {
        assert!(compute_cvt(0, 1080, 60.0, TimingFormula::CVT).is_none());
        assert!(compute_cvt(1920, 0, 60.0, TimingFormula::CVT).is_none());
        assert!(compute_cvt(1920, 1080, 0.0, TimingFormula::CVT).is_none());
        assert!(compute_cvt(1920, 1080, f64::NAN, TimingFormula::CVT).is_none());
        assert!(compute_cvt(1920, 1080, f64::INFINITY, TimingFormula::CVT).is_none());
        assert!(compute_cvt(1920, 1080, 5000.0, TimingFormula::CVT).is_none());
        assert!(compute_cvt(1920, 1080, 60.0, TimingFormula::Manual).is_none());
    }
    #[test]
    fn dtd_validation_reports_field_limits_without_overflow() {
        let mut timing = DetailedTiming {
            h_active: 1,
            v_active: 1,
            h_front: u32::MAX,
            h_sync: 0,
            h_back: 0,
            v_front: 0,
            v_sync: 0,
            v_back: 0,
            h_border: 0,
            v_border: 0,
            pixel_clock_khz: 1,
            h_pol: false,
            v_pol: false,
            v_rate: 1.0,
        };
        assert!(matches!(
            validate_dtd(&timing),
            Err(crate::error::DtdError::FieldOutOfRange {
                field: crate::error::DtdField::HorizontalFrontPorch,
                ..
            })
        ));
        timing.pixel_clock_khz = 10_000;

        timing.h_front = 1023;
        timing.h_sync = 1023;
        timing.h_back = u32::MAX;
        assert!(matches!(
            validate_dtd(&timing),
            Err(crate::error::DtdError::FieldOutOfRange {
                field: crate::error::DtdField::HorizontalBlanking,
                ..
            }) | Err(crate::error::DtdError::ArithmeticOverflow)
        ));
    }

    #[test]
    fn dtd_validation_accepts_canonical_timing() {
        let timing = DetailedTiming {
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
        };
        assert_eq!(validate_dtd(&timing), Ok(()));
    }
    #[test]
    fn dtd_validation_rejects_zero_active_or_clock() {
        let mut timing = DetailedTiming {
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
        };
        timing.h_active = 0;
        assert!(matches!(
            validate_dtd(&timing),
            Err(crate::error::DtdError::InvalidField {
                field: crate::error::DtdField::HorizontalActive,
                value: 0
            })
        ));

        timing.h_active = 1920;
        timing.pixel_clock_khz = 0;
        assert!(matches!(
            validate_dtd(&timing),
            Err(crate::error::DtdError::InvalidField {
                field: crate::error::DtdField::PixelClockKHz,
                value: 0
            })
        ));
    }
    #[test]
    fn dtd_validation_rejects_unreadable_low_pixel_clock() {
        let timing = DetailedTiming {
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
            pixel_clock_khz: 9,
            h_pol: true,
            v_pol: true,
            v_rate: 60.0,
        };
        assert!(matches!(
            validate_dtd(&timing),
            Err(crate::error::DtdError::InvalidField {
                field: crate::error::DtdField::PixelClockKHz,
                value: 9
            })
        ));
    }
}
