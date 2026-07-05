# Verification Schemas

This page documents the serialized artifact formats emitted by `karpal-verify` and the compatibility contract around their `schema_version` markers.

## Versioned artifacts

- verification report JSON
- Lean manifest JSON
- Lean diagnostics sidecar JSON
- nested `report_files` metadata blocks embedded in report and manifest JSON

## Current version

The current published schema version is `1`. In Rust code this is exposed through constants such as `VERIFICATION_REPORT_SCHEMA_VERSION`, `LEAN_MANIFEST_SCHEMA_VERSION`, and `VERIFICATION_SIDECAR_SCHEMA_VERSION`.

## Version 1 guarantees

- each top-level JSON object includes a string `schema_version`
- existing field names remain stable within the `1.x` line
- new fields are added only as optional, forward-compatible extensions
- nested `report_files` objects also include their own `schema_version`
- string path fields preserve the artifact/session paths written by the current run

## Consumer guidance

External CI tooling, bots, or archive readers should:

1.  accept `schema_version == "1"`
2.  ignore unknown optional fields for forward compatibility
3.  treat a future schema bump as a breaking parser boundary

## Additive vs. breaking changes

The schema version stays at `1` for additive changes such as new optional counters, new optional metadata blocks, or richer cross-linking fields. A future version bump is required if a required field is removed, renamed incompatibly, changes type, or changes meaning.

## Related guides

- [Proof & Verification](proof-verification.md) for the overall model/export/run/trust story
- [Verification CI Workflow](verification-ci.md) for report/layout usage in automation
- [Verification Workflow](../examples/verification-workflow.md) for an end-to-end example


Karpal is licensed under AGPL-3.0-or-later. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).


