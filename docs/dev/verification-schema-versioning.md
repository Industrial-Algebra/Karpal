# Verification Schema Versioning

This document defines the stability contract for serialized verification artifacts produced by `karpal-verify`.

## Scope

The current schema-versioned outputs are:

- verification report JSON
- Lean manifest JSON
- Lean diagnostics sidecar JSON
- nested `report_files` metadata embedded in report / manifest JSON

## Current Versions

Version `1` is exposed in code through:

- `VERIFICATION_REPORT_SCHEMA_VERSION`
- `LEAN_MANIFEST_SCHEMA_VERSION`
- `VERIFICATION_SIDECAR_SCHEMA_VERSION`

## Version `1` Guarantees

Schema version `1` guarantees:

- top-level JSON objects include a string `schema_version`
- existing field names remain stable within the `1.x` line
- optional fields may be absent or `null`, but existing required fields keep their meaning
- nested `report_files` objects also carry their own `schema_version`
- path fields are serialized as strings exactly as written by the current artifact/session layer

In practice, consumers may safely parse version `1` artifacts by:

1. checking `schema_version == "1"`
2. treating unknown additional fields as forward-compatible extensions
3. treating missing optional fields as absent data rather than schema failure

## Compatibility Policy

### Minor-compatible additions

The schema version stays at `1` when changes are additive, such as:

- adding new optional fields
- adding new counts / summaries
- adding new optional nested metadata blocks
- enriching manifest/report cross-links without changing existing field meaning

Consumers should ignore unknown fields to remain compatible with additive `1.x` changes.

### Breaking changes

The schema version must increment to `2` or later when any of the following happen:

- a required field is removed
- a field changes meaning
- a field changes type
- nested object structure changes incompatibly
- success/failure semantics in serialized output are represented differently in a way that breaks existing parsers

## Recommended Consumer Strategy

Consumers should:

- reject unknown major schema versions
- accept known schema version `1`
- log unknown optional fields but ignore them
- prefer constants from `karpal-verify` where they are available instead of hardcoding version strings

## Planned Migration Path

If a future `2` is introduced, `karpal-verify` should aim to:

- document the delta from `1`
- keep `1` artifacts readable in historical CI archives
- note migration expectations in crate docs and reference docs

## Notes

Schema versioning is intentionally independent from crate versioning. A crate release may keep schema version `1` if serialized compatibility is preserved.
