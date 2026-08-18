# edid-seria
[![CI](https://img.shields.io/github/actions/workflow/status/kouhe3/edid-seria/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/kouhe3/edid-seria/actions/workflows/ci.yml) [![Crates.io](https://img.shields.io/crates/v/edid-seria?style=flat-square)](https://crates.io/crates/edid-seria) [![docs.rs](https://img.shields.io/docsrs/edid-seria?style=flat-square)](https://docs.rs/edid-seria) [![MSRV](https://img.shields.io/badge/MSRV-1.95-blue?style=flat-square)](https://github.com/kouhe3/edid-seria) [![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)](#license)

EDID parsing and serialization in pure Rust, with zero dependencies: the
detailed timing model, CVT / CVT-RB / CVT-RB2 timing computation, EDID
base-block read/write, and the resolution → EDID override pipeline used by
CRU-style display override tools.

## Features

- **DTD read/write** — byte-exact round-trips of detailed timing descriptors,
  including borders and sync-type-aware polarity decoding (verified against
  edid-decode over ~49,000 real monitor EDIDs).
- **CVT computation** — CVT 1.1 normal blanking, CVT-RB, and CVT-RB2
  (validated field-by-field against the `cvt12.c` reference implementation).
- **Preset tables** — PC (VESA) and HDTV (CEA-861) standard timings matching
  CRU's automatic tables.
- **Safe serialization** — rewrites the base block's DTD slots while
  preserving monitor descriptors (name, serial, range limits) and extension
  blocks; rejects timings that cannot be represented in a DTD instead of
  silently truncating them.
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

## Commands

| Command | Description |
|---------|-------------|
| `cargo test` | Run the test suite (bit-level golden bytes, round-trips, CVT cases) |
| `cargo clippy --all-targets -- -D warnings` | Lint gate used by CI |
| `cargo doc --no-deps` | Build documentation (`missing_docs` is a warning) |
| `cargo package` | Verify the publishable package |

## Architecture

| Module | Responsibility |
|--------|----------------|
| [`timing`](src/timing.rs) | `DetailedTiming` model, preset tables, CVT computation, `dtd_fits` field-limit check |
| [`edid`](src/edid.rs) | 18-byte DTD bit-packing, slot classification, base-block read/write, checksum |
| [`serialize`](src/serialize.rs) | Resolution → EDID pipeline: compute DTDs, rewrite slots, preserve descriptors, fix checksums |

Key design decisions are recorded in [docs/decisions](docs/decisions/).

## Contributing

PRs welcome. CI runs `fmt`, `clippy -D warnings`, tests, and rustdoc on every
push and pull request; the same gates must pass locally:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE),
at your option.
