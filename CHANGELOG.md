# Changelog

All notable changes to `edid-seria` are documented here.

## [Unreleased]

### Changed

- Documented the serialization boundary: `Edid::to_bytes()` remains an
  unchecked compatibility serializer, while `Edid::to_bytes_checked()` is the
  validated publishing path and returns no partial output on failure.
- Documented the current lossless/raw-preservation contract, the limits of
  canonicalization, and the distinction between raw and typed extension APIs.
- Expanded the parser fuzz target with checked-output reparse/stability oracles,
  CTA typed-view encoding, raw DisplayID encoding, and extension lifecycle
  operations available in the current API.
- CI now builds and smoke-runs the fuzz target and verifies package file/source
  coverage without assuming that fuzz tooling exists on every platform runner.

### API boundaries

- CTA typed views can be encoded through `CtaDataBlockView::to_data_block()`;
  unknown and reserved payloads should use raw `CtaDataBlock` values. DisplayID
  typed views also provide `to_data_block()` and
  `to_data_block_with_tag()`. These encoders emit canonical tags and normalize
  the encoded data-block revision to zero. Embedded CTA typed encoding
  normalizes each nested typed block; representable raw tails are preserved by
  the nested CTA encoder, while unsupported or non-representable tails may be
  rejected rather than silently treated as lossless.
- CTA header/DTD mutation and broader canonicalization policy controls remain
  available only through their existing checked/raw boundaries; typed
  DisplayID editing is no longer inspection-only.
- The crate's MSRV is Rust 1.95 (edition 2024). Public enums are part of the
  API surface: adding variants can make downstream exhaustive `match`
  expressions fail to compile. `ExtensionError` and `ExtensionWriteError` are
  `#[non_exhaustive]`; callers must use a wildcard arm when matching them.

## [0.1.0] - 2026-08-26

### Added

- Strict EDID parsing and checked complete-sequence serialization with
  structured errors, extension-count validation, and checksum validation.
- Base-block detailed timing, metadata, descriptor, modeline, and hex helpers,
  including compatibility and checked serialization APIs.
- Raw CTA-861 and DisplayID extension constructors and replacement APIs with
  automatic bounds, offsets, payload lengths, and checksums.
- CTA typed data-block encoders for the modeled video, audio, speaker,
  extended, and vendor-specific views, with representable raw-tail
  preservation and rejection of unrepresentable edits.
- Extension lifecycle operations for insertion, replacement, and removal.

### Compatibility

- `Edid::to_bytes()` preserves its unchecked compatibility semantics and emits
  the bytes currently stored in the public block fields. Use
  `Edid::to_bytes_checked()` before publishing or writing externally supplied
  state.
- Raw/unknown data is preserved where the raw API carries it, but this release
  does not promise canonical ordering or byte-for-byte output after a typed
  view is edited.

[Unreleased]: https://github.com/kouhe3/edid-seria/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kouhe3/edid-seria/releases/tag/v0.1.0
