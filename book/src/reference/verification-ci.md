# Verification CI Workflow

This guide shows how to use the `karpal-verify` stack in continuous integration. The goal is to make external verification runs inspectable and archivable: generate artifacts, execute plans with explicit backend policies, and persist JSON / Markdown summaries beside those artifacts.

## Workflow overview

1.  Construct an `ObligationBundle` from an `AlgebraicSignature`.
2.  Choose an `ArtifactLayout` under your CI workspace.
3.  Run a `VerificationSession` or `verify_bundle_with_ci_outputs(...)`.
4.  Publish the generated SMT / Lean artifacts and the report files as CI artifacts.
5.  Review `VerificationReport` and imported certificates at explicit trust boundaries.

## Directory layout

`karpal-verify` uses a predictable on-disk layout. Given a root like `target/karpal-verify`:

``` text
target/karpal-verify/
├── smt/
│   ├── associativity.smt2
│   ├── left_identity.smt2
│   └── right_identity.smt2
├── lean/
│   ├── KarpalVerify.lean
│   └── KarpalVerify.manifest.json
├── lakefile.lean
├── lean-toolchain
├── verification-report.json
├── verification-report.md
└── verification-report.lean-diagnostics.json
```

This layout is useful in CI because a single directory can be attached as an artifact bundle for later inspection.

## One-shot helper

For simple CI jobs, the easiest entry point is `verify_bundle_with_ci_outputs(...)`:

``` rust
use karpal_verify::{
    verify_bundle_with_ci_outputs, AlgebraicSignature, ArtifactLayout, DryRunner,
    LeanConfig, ObligationBundle, Origin, SmtConfig, Sort,
};

let sig = AlgebraicSignature::monoid(Sort::Int, "combine", "e");
let bundle = ObligationBundle::monoid(
    "sum_monoid",
    Origin::new("karpal-core", "Monoid for Sum<i32>"),
    &sig,
);

let output = verify_bundle_with_ci_outputs(
    &bundle,
    &ArtifactLayout::new("target/karpal-verify"),
    "KarpalVerify",
    &SmtConfig::default(),
    &LeanConfig::default(),
    &DryRunner,
).expect("verification run should succeed");

assert!(output.report_files.json_path.ends_with("verification-report.json"));
assert!(output.report_files.markdown_path.ends_with("verification-report.md"));
```

This function builds artifacts, runs plans with the supplied runner, and writes CI-oriented summaries directly beside the generated files. When Lean artifacts are present, the output set also includes a typed Lean manifest and a Lean diagnostics sidecar so CI systems can archive both the source-level proof context and the parsed failure surface.

## Session API

For more control, use `VerificationSession`. It lets you customize solver binaries, Lean arguments, Lean execution drivers such as direct `lean`, `lake env lean`, or `lake build`, and the report file stem.

``` rust
use karpal_verify::{
    AlgebraicSignature, ArtifactLayout, LeanConfig, ObligationBundle, Origin,
    SmtConfig, Sort, VerificationSession,
};

let sig = AlgebraicSignature::semiring(Sort::Int, "add", "zero", "mul", "one");
let bundle = ObligationBundle::semiring(
    "wrap_ring",
    Origin::new("karpal-algebra", "Semiring for WrapRing"),
    &sig,
);

let session = VerificationSession::new(
    bundle,
    ArtifactLayout::new("target/verify-semiring"),
    "KarpalVerify",
)
.with_smt_config(SmtConfig::new("z3").with_arg("-smt2"))
.with_lean_config(LeanConfig::new("lean"))
.with_report_stem("ci-summary");
```

### Dry-run validation in CI

A dry run is useful when you want to validate export and path generation without requiring external tools to be installed on every CI job:

``` rust
let report = session.dry_run_report();
assert_eq!(report.obligation_count(), 6);
assert!(report.obligations.iter().all(|o| o.status().is_some()));
```

Because `DryRunner` returns shell-rendered commands, it is a good fit for preview jobs and artifact smoke tests.

### Real execution in CI

When the CI image includes your solver and Lean, use:

``` rust
let output = session
    .verify_local_with_ci_outputs()
    .expect("local verification should run");

if !output.report.is_success() {
    panic!("verification failed; inspect generated report artifacts");
}
```

This path uses `LocalProcessRunner`, backend-specific `VerificationPolicy` interpretation, and report serialization.

## Backend policy behavior

CI should not guess what “success” means. In `karpal-verify`, success is interpreted explicitly:

| Backend | Success condition          | Why                                                                            |
|---------|----------------------------|--------------------------------------------------------------------------------|
| SMT     | `ExecutionStatus::Unsat`   | Karpal exports the negation of the obligation, so `unsat` means the law holds. |
| Lean    | `ExecutionStatus::Success` | The module is accepted by Lean without process failure.                        |

This distinction matters in CI dashboards and failure triage: a solver returning `sat` or `unknown` should not be treated the same way as a successful Lean process.

## Report files

The JSON and Markdown outputs are intentionally lightweight:

- **JSON** is useful for CI bots, artifact scraping, or post-processing.
- **Markdown** is useful for human inspection in uploaded artifacts or job summaries.

The report includes bundle name, root directory, success/failure counts, per-obligation status, artifact paths, and certificate summaries where available.

### Schema versioning

The verification report JSON, generated Lean manifest JSON, and Lean diagnostics sidecar each include a top-level `schema_version` marker. Nested `report_files` metadata blocks are versioned too.

Current schema version is `1`. Within the `1.x` line, existing fields are expected to remain stable and new data should be added only through optional fields. Consumers should therefore:

- accept `schema_version == "1"`
- ignore unknown optional fields for forward compatibility
- treat a future schema bump as a breaking parser boundary

For the fuller compatibility policy and migration expectations, see [Verification Schemas](verification-schemas.md).

## Sample CI shape

A typical CI pipeline might split external verification into two jobs:

1.  **Export / dry-run job**
    - Runs on every PR
    - Generates artifacts and dry-run reports
    - Publishes export files for review
2.  **Solver-backed verification job**
    - Runs where Z3 / Lean are available
    - Executes local verification
    - Uploads the same artifact directory plus final report files

## Recommended practices

- Use deterministic artifact roots so CI artifacts are easy to find.
- Archive the whole layout root, not just the JSON report.
- Keep report file names stable via `DEFAULT_REPORT_STEM` unless you need multiple outputs.
- Review imported certificates separately from exporter correctness.
- Treat `Certified<...>` as an explicit trust handoff, not as an automatic theorem import.

For an end-to-end example with SMT and Lean-oriented snippets, see the [Verification Workflow example](../examples/verification-workflow.md). For the higher-level API overview, return to [Proof & Verification](proof-verification.md). For the serialized schema contract, see [Verification Schemas](verification-schemas.md).


Karpal is licensed under Apache-2.0 + CLA. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).


