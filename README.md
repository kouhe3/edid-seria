# edid-seria
[![CI](https://img.shields.io/github/actions/workflow/status/kouhe3/edid-seria/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/kouhe3/edid-seria/actions/workflows/ci.yml) [![Crates.io](https://img.shields.io/crates/v/edid-seria?style=flat-square)](https://crates.io/crates/edid-seria) [![docs.rs](https://img.shields.io/docsrs/edid-seria?style=flat-square)](https://docs.rs/edid-seria) [![MSRV](https://img.shields.io/badge/MSRV-1.95-blue?style=flat-square)](https://github.com/kouhe3/edid-seria) [![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)](#license)

EDID parsing and serialization in pure Rust, with zero dependencies: the
detailed timing model, CVT / CVT-RB / CVT-RB2 timing computation, EDID
base-block read/write, and the resolution → EDID override pipeline used by
CRU-style display override tools.

## Features

 - **DTD read/write** — field-level encoding and decoding of detailed timing
   descriptors, including borders and sync-type-aware polarity decoding.
- **CVT computation** — CVT 1.1 normal blanking, CVT-RB, and CVT-RB2
  (validated field-by-field against the `cvt12.c` reference implementation).
 - **Preset tables** — PC (VESA), common wide-screen, and HDTV (CEA-861)
   timings; the HDTV table covers common modes, not every CEA VIC.
- **Safe serialization** — rewrites the base block's DTD slots while
  preserving monitor descriptors (name, serial, range limits) and extension
  blocks; rejects timings that cannot be represented in a DTD instead of
  silently truncating them.
 - **Strict parsing** — validated complete EDID sequences with base-header,
   checksum, version, extension-count, and structured error reporting.
 - **Metadata and descriptors** — typed base-block identity, chromaticity
   fixed-point coordinates, Established Timings, Standard Timings, and common
   monitor descriptors (name, serial, alphanumeric string, color point, additional standard timings, range limits, dummy), with unknown descriptor payload preservation.
 - **Manual and interlaced DTD access** — strict manual timing serialization
   plus raw DTD flag round-trips; the legacy timing view remains progressive-only.
- **Extension views** — typed read-only CTA video/audio/VSDB (HDMI 1.4b, HDMI Forum 2.0+)/HDR Static Metadata and
  Adaptive-Sync data-block views, DisplayID headers, Product Identification,
  Display Parameters, Type I/Type VII Detailed Timing, embedded CTA, and raw
  unknown-block views. Extension generation/reordering remains out of scope.
- **Modeline and Hex interoperability** — X11/xrandr Modeline string formatting and
  parsing, and flexible EDID hex string (compact, formatted, C-array) import/export.
- **`#![deny(unsafe_code)]`**, no panics on arbitrary EDID input, MSRV 1.95.

## Quick start

```toml
[dependencies]
edid-seria = "0.1"
```

```rust
use edid_seria::{EdidBlock, ResolutionSpec, TimingKind, serialize_resolutions};

// Read a display's detailed timings.
let block = EdidBlock::from_bytes(&edid_bytes).unwrap();
for t in block.detailed_timings() {
    println!("{}", t.label()); // e.g. "1920x1080 @ 60Hz"
}

// Build an EDID override: keep the display's existing EDID, replace the
// base-block timings with two computed modes.
let res = serialize_resolutions(
    Some(&edid_bytes),
    &[
        ResolutionSpec { width: 1920, height: 1080, refresh: 60.0, kind: TimingKind::Pc },
        ResolutionSpec { width: 3840, height: 2160, refresh: 60.0, kind: TimingKind::Hdtv },
    ],
);
assert_eq!(res.skipped, 0); // 0 = every requested mode was written
fs::write("override.bin", &res.bytes)?;
```

## Core-library boundaries

The library is platform-independent. It does not enumerate displays, access the
Windows registry, request elevation, or apply driver overrides. CTA-861 and
DisplayID blocks are currently inspected read-only; their data is preserved but
not generated or rearranged.

`serialize_resolutions` retains its compatibility behavior: malformed or
partial input may fall back or be skipped. New code should use
`serialize_resolutions_checked` or `serialize_timings`, which reject malformed
input, invalid timings, and unavailable DTD slots with structured errors.

The strict DTD API is byte-precise for representable fields. New writes
normalize flags to digital separate sync unless the explicit flagged writer is
used; analog/composite semantics are not inferred by the progressive-only view.

## Commands

| Command | Description |
|---------|-------------|
| `cargo test` | Run the test suite (bit-level golden bytes, round-trips, CVT cases) |
| `cargo clippy --all-targets -- -D warnings` | Lint gate used by CI |
| `cargo doc --no-deps` | Build documentation (`missing_docs` is a warning) |
| `cargo package` | Verify the publishable package |

The optional fuzz target is under `fuzz/` and can be run with
`cargo-fuzz` after installing that tool.

## Architecture

| Module | Responsibility |
|--------|----------------|
| [`extensions`](src/extensions.rs) | CTA-861 data-block views, DisplayID section/block parsing and typed views, and extension-kind detection |
| [`serialize`](src/serialize.rs) | Resolution/manual timing → EDID pipeline with strict and compatibility APIs |
| [`error`](src/error.rs) | Structured parsing, DTD, metadata, descriptor, and serialization errors |

Key design decisions are recorded in [docs/decisions](docs/decisions/).

## Contributing

PRs welcome. CI runs `fmt`, `clippy -D warnings`, tests, rustdoc, package, and
library checks on Linux, Windows, and macOS; the same gates must pass locally:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

The corpus/property smoke tests live in `tests/core_regression.rs`; the
libFuzzer parser target lives in `fuzz/fuzz_targets/parse_edid.rs`.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE),
at your option.
