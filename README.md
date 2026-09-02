# edid-seria
[![CI](https://img.shields.io/github/actions/workflow/status/kouhe3/edid-seria/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/kouhe3/edid-seria/actions/workflows/ci.yml) [![Crates.io](https://img.shields.io/crates/v/edid-seria?style=flat-square)](https://crates.io/crates/edid-seria) [![docs.rs](https://img.shields.io/docsrs/edid-seria?style=flat-square)](https://docs.rs/edid-seria) [![MSRV](https://img.shields.io/badge/MSRV-1.95-blue?style=flat-square)](https://github.com/kouhe3/edid-seria) [![License](https://img.shields.io/badge/license-Unlicense-blue?style=flat-square)](#license)

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
 - **Safe serialization** — rewrites Base Block DTD slots while preserving
   monitor descriptors and extension blocks. Checked extension constructors
   provide raw CTA-861 and DisplayID data-block collection layout, payload
   limits, offsets, and checksums.
 - **Strict parsing** — validated complete EDID sequences with base-header,
   checksum, version, extension-count, and structured error reporting.
 - **Metadata and descriptors** — typed base-block identity, chromaticity
   fixed-point coordinates, Established Timings, Standard Timings, and complete
   EDID 1.4 standard monitor descriptors (name, serial, alphanumeric string, color point, additional standard timings, Established Timings III, CVT 3-byte timing codes, Display Color Management, and extended Range Limits with Secondary GTF & CVT support), with unknown descriptor payload preservation.
 - **Manual and interlaced DTD access** — strict manual timing serialization
   plus raw DTD flag round-trips; the legacy timing view remains progressive-only.
 - **Extension writers** — `CtaDataBlockView::to_data_block` encodes the
   modeled CTA typed views (including representable vendor tails), while raw
   CTA blocks remain available for unknown or reserved data. CTA capability
   flags/header and DTD collections can be edited with checked mutation APIs.
   `DisplayIdDataBlockView::to_data_block` and
   `to_data_block_with_tag` encode Product Identification, Display Parameters,
   Type I/Type VII detailed timings, embedded CTA, and unknown raw blocks;
   `DisplayIdDataBlock::encode` remains the raw block writer.
 - **High-level display inspection** — convenient `Edid` helpers (`monitor_name()`, `serial_number()`, `preferred_timing()`, `all_detailed_timings()`) that aggregate Base and CTA extension descriptors.
 - **Modeline and Hex interoperability** — X11/xrandr Modeline string formatting and
   parsing, and flexible EDID hex string (compact, formatted, C-array) import/export.
 - **`#![deny(unsafe_code)]`**, no panics on arbitrary EDID input, MSRV 1.95.

## Quick start

```toml
[dependencies]
edid-seria = "0.1"
```

```rust,no_run
use std::fs;
use edid_seria::{EdidBlock, ResolutionSpec, TimingKind, serialize_resolutions};

fn main() -> std::io::Result<()> {
    // Read a display's detailed timings.
    let edid_bytes = fs::read("input.bin")?;
    let block = EdidBlock::from_bytes(&edid_bytes).expect("valid EDID base block");
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
    fs::write("override.bin", &res.bytes)
}
```

## Core-library boundaries

The library is platform-independent. It does not enumerate displays, access the
Windows registry, request elevation, or apply driver overrides.

`Edid::to_bytes()` is the unchecked compatibility serializer: it concatenates
the public base and extension block bytes exactly as stored, without repairing
the extension count, checksums, offsets, or payload lengths. Use
`Edid::to_bytes_checked()` (or `validate_for_serialization()` first) when output
must be validated. Checked output is atomic: invalid state returns an error
instead of partially serialized bytes.

The parse → `to_bytes()` path is lossless for the bytes held by an `Edid`.
Raw CTA and DisplayID data-block APIs preserve unknown/reserved payload bytes
and source order when those blocks are passed through unchanged. For CTA
offset-zero collections, an all-zero suffix is interpreted as padding; an
otherwise empty tag-0 block at that boundary is therefore ambiguous and is
not treated as a distinct block. The CTA typed writer,
`CtaDataBlockView::to_data_block`, encodes the fields represented by its
view and retains representable vendor/raw tails; it may normalize fields that
the typed view cannot represent, and it rejects unrepresentable edits. CTA
header/capability and DTD mutation are checked and preserve unrelated layout
or data-block content as documented by their APIs.

DisplayID typed views have encoders for Product Identification, Display
Parameters, Type I/Type VII detailed timing, embedded CTA, and unknown blocks.
`DisplayIdDataBlockView::to_data_block_with_tag` selects the 1.x/2.x tag when
that distinction matters; the shorthand `to_data_block` chooses a canonical
tag. `DisplayIdDataBlock::encode` and raw collection constructors remain
available when the caller owns the exact raw block fields.

No implicit canonicalization is promised. Canonical ordering, flag policy, and
other multi-strategy normalization remain deferred; typed writers may choose a
canonical tag or normalize fields not represented by their view. Use raw APIs
when exact payload preservation is required.


The strict DTD API is byte-precise for representable fields. New writes
normalize flags to digital separate sync unless the explicit flagged writer is
used; analog/composite semantics are not inferred by the progressive-only view.

## Commands

| Command | Description |
|---------|-------------|
| `cargo test` | Run the test suite (bit-level golden bytes, round-trips, CVT cases) |
| `cargo clippy --all-targets -- -D warnings` | Lint gate used by CI |
| `cargo doc --no-deps` | Build documentation (`missing_docs` is a warning) |
| `cargo package --list` | Inspect the files that will enter the source package |
| `cargo package` | Verify the publishable package and its packaged source |

The optional fuzz target is under `fuzz/`. After installing `cargo-fuzz`, run
`cargo fuzz build parse-edid` to compile it and
`cargo fuzz run parse-edid -- -runs=64` for a bounded smoke run. The target
checks that successful `to_bytes_checked()` output reparses and is stable,
and exercises raw and currently available typed extension paths.
## Architecture

| Module | Responsibility |
|--------|----------------|
| [`extensions`](src/extensions.rs) | CTA-861 data-block views, DisplayID section/block parsing and typed views, and extension-kind detection |
| [`serialize`](src/serialize.rs) | Resolution/manual timing → EDID pipeline with strict and compatibility APIs |
| [`error`](src/error.rs) | Structured parsing, DTD, metadata, descriptor, and serialization errors |

Key design decisions are recorded in [docs/decisions](docs/decisions/).

## Contributing

PRs welcome. CI runs formatting, Clippy, tests, rustdoc, package-list/source
checks, and a bounded libFuzzer build/smoke job. It also runs library and
integration tests on Ubuntu, Windows, and macOS, plus MSRV 1.95 tests on
Ubuntu. The fuzz job is intentionally Ubuntu-only and does not add fuzzing
assumptions to the platform matrix. To reproduce the quality job locally:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo package --list
cargo package
```

The corpus/property smoke tests live in `tests/core_regression.rs`; the
libFuzzer parser target lives in `fuzz/fuzz_targets/parse_edid.rs`.

## License

This is free and unencumbered software released into the public domain. See
[UNLICENSE](UNLICENSE) for the complete dedication and warranty disclaimer.
