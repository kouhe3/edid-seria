# ADR-001: Serialization scope, flag normalization, and edid-decode alignment

## Status
Accepted

## Date
2026-08-14

## Context

`edid-seria` exists to feed a CRU-style display override tool: take a user's
requested resolutions, rewrite the EDID base block's detailed timing
descriptors, and produce a binary the OS/driver will accept. Three decisions
shape the whole serializer and were made explicitly during verification
against 48,960 real monitor EDIDs and the `edid-decode` reference decoder.

## Decision

### 1. Base-block DTD rewriting only — no CTA-861 / DisplayID generation

New timings are written exclusively into the four 18-byte DTD slots of the
base block; extension blocks are preserved but never extended.

Alternatives considered:

- **CTA-861 Video Data Blocks** — the standard way TVs advertise rates, but
  requires growing/editing the extension block and interleaving with existing
  data blocks; overkill for PC-monitor overrides, which is what CRU's own
  "detailed resolutions" do.
- **DisplayID** — needed for modes beyond DTD limits (e.g. v_front > 63 at
  high refresh), but not readable by all older drivers.

Consequence: modes whose computed timing does not fit a DTD (see
`timing::dtd_fits`) are rejected with `skipped += 1` rather than emitted
incorrectly. E.g. CVT-RB2 2560×1440@144 has v_front = 89 > 63 and cannot be
serialized; supporting it would require DisplayID generation, deliberately
out of scope.

### 2. Flags normalized to digital separate sync on write

`write_detailed` always emits sync type 11 (digital separate) with H/V
polarity in bits 1/2 of DTD byte 17. Analog serrate/sync-on-green bits are
not preserved.

Alternatives: bit-preserving round-trip of byte 17. Rejected: the target
hardware is digital, and byte-17 semantics differ per sync type (E-EDID 1.4
§3.10.2) — preserving analog flag bits while relabeling the mode as digital
would be misleading. `read_detailed` decodes polarity per sync type so
analog EDIDs still parse correctly.

### 3. Heuristics aligned with edid-decode

Three validity heuristics match `edid-decode` exactly, so the library's view
of a block agrees with the reference decoder used to validate it:

- all-`0x01` descriptors are padding, not timings;
- pixel clock < 10 MHz is invalid data, not a timing;
- borders (bytes 15/16) are subtracted from blanking when splitting porches.

Alternatives: strict spec-only parsing. Rejected: real EDIDs contain all
three patterns (padding slots, junk descriptors, border DTDs); treating them
as timings yields garbage modes that would then be rewritten into user
output.

### 4. CVT-RB2 default for PC modes, CEA presets for HDTV modes

`TimingKind::Pc` computes CVT 1.2 reduced blanking; `TimingKind::Hdtv` looks
up CEA-861 preset timings with CVT 1.1 as fallback. Mirrors CRU's
AutomaticPC/AutomaticHDTV behavior; the CVT port is verified against
`cvt12.c` and the preset tables against CRU's source.

### 5. Strict APIs are additive and lossless by default

The compatibility serializer remains available for existing callers, but new
code should use `serialize_resolutions_checked` or `serialize_timings`. These
APIs reject malformed block sequences, bad checksums, unsupported base-block
versions, invalid DTD fields, and unavailable slots with structured errors.

The strict parser aggregates the base block and extensions, identifies CTA-861,
DisplayID, and unknown extension tags, and preserves unknown raw bytes. CTA and
DisplayID generation remains out of scope.

## Consequences

- Strict serialization never emits a timing it cannot represent and never
  silently drops malformed existing input.
- Round-trips are byte-exact for digital separate sync DTDs (bytes 0-11,
  15-16 and polarity bits), and flag-normalized for analog/composite ones.
- `DetailedTiming` gained `h_border`/`v_border` fields to make border
  handling lossless.
- Interlaced DTD flags are available through the flagged DTD API; the legacy
  `read_detailed` view remains progressive-only.
