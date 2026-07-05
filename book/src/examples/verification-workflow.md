# Verification Workflow

This example walks through a realistic `karpal-verify` workflow: define a bundle of algebraic obligations, export SMT and Lean artifacts, preview commands with a dry run, produce CI-oriented summaries, and finally import an external certificate through the explicit trust boundary.

## Scenario

Suppose you have a type that behaves like an additive monoid and you want three things:

- a machine-readable description of its laws,
- export artifacts for SMT and Lean, and
- a reviewable path from external verification back into Rust.

The `karpal-verify` stack is designed exactly for this flow.

## 1. Build an obligation bundle

``` rust
use karpal_std::prelude::*;

let sig = AlgebraicSignature::monoid(Sort::Int, "combine", "e");
let bundle = ObligationBundle::monoid(
    "sum_monoid",
    Origin::new("karpal-core", "Monoid for Sum<i32>"),
    &sig,
);

assert_eq!(bundle.obligations().len(), 3);
```

The resulting bundle contains associativity, left identity, and right identity. The bundle becomes the shared source for every downstream step.

## 2. Export SMT and Lean artifacts

``` rust
let smt_scripts = export_smt_bundle(&bundle);
let lean_module = export_lean_bundle("KarpalVerify", &bundle);

assert_eq!(smt_scripts.len(), 3);
assert!(lean_module.contains("namespace KarpalVerify"));
```

At this stage you still have plain strings in memory. This is useful when integrating with other tools or building higher-level export pipelines.

## 3. Write artifacts and inspect invocation plans

Once you choose a root layout, `karpal-verify` can materialize files and the command plans needed to run them.

``` rust
let layout = ArtifactLayout::new("target/karpal-verify-example");
let batch = dry_run_bundle_artifacts(
    &bundle,
    &layout,
    "KarpalVerify",
    &SmtConfig::default(),
    &LeanConfig::default(),
);

for plan in &batch.plans {
    println!("{}", plan.render_shell());
}
```

A dry-run batch is especially useful while wiring CI, because it validates paths and exporter output without requiring solver binaries to be available. The batch also carries structured Lean export metadata, generated Lean project data, and a typed Lean manifest model that will later be serialized beside the generated module.

## 4. Orchestrate build → run → report

The orchestration layer wraps the lower-level pieces into one cohesive flow. For example, here is a dry-run CI-style session:

``` rust
let output = verify_bundle_with_ci_outputs(
    &bundle,
    &ArtifactLayout::new("target/karpal-verify-example"),
    "KarpalVerify",
    &SmtConfig::default(),
    &LeanConfig::default(),
    &DryRunner,
).expect("verification session should succeed");

assert_eq!(output.report.obligation_count(), 3);
assert!(output.report_files.json_path.ends_with("verification-report.json"));
assert!(output.report_files.markdown_path.ends_with("verification-report.md"));
```

This one call does all of the following:

- writes SMT and Lean artifacts,
- creates invocation plans,
- runs them with the supplied runner,
- builds a `VerificationReport`,
- writes JSON / Markdown summaries beside the generated artifacts,
- writes a schema-versioned Lean diagnostics sidecar, and
- cross-links a schema-versioned Lean manifest back to those report files.

## 5. Understand backend semantics

The same word “success” means different things depending on the backend:

``` rust
assert!(VerificationPolicy::for_kind(CommandKind::Smt)
    .accepts(ExecutionStatus::Unsat));
assert!(VerificationPolicy::for_kind(CommandKind::Lean)
    .accepts(ExecutionStatus::Success));
```

For SMT backends, Karpal exports the negation of the law, so `unsat` is the success case. For Lean, success is an accepted module together with parsed diagnostics that report no errors. Lean diagnostics are then mapped back to exported theorem identities, using source-line spans as a fallback when the diagnostic message does not name the theorem directly.

## 6. Use the session builder for more control

If you need to configure tool names, extra arguments, or custom report names, use `VerificationSession` directly:

``` rust
let session = VerificationSession::new(
    bundle.clone(),
    ArtifactLayout::new("target/karpal-verify-example-2"),
    "KarpalVerify",
)
.with_smt_config(SmtConfig::new("z3").with_arg("-smt2"))
.with_lean_config(
    LeanConfig::new("lean")
        .with_driver(LeanDriver::LakeBuild)
)
.with_report_stem("nightly-summary");

let dry_report = session.dry_run_report();
assert!(dry_report.obligations.iter().all(|o| o.status().is_some()));
```

## 7. Import external evidence explicitly

The final step is intentionally explicit. External evidence first becomes a certificate and a `Certified<...>` wrapper, not a `Proven<...>` value.

``` rust
use karpal_proof::{IsAssociative, Proven};
use karpal_verify::{Certificate, Certified, SmtCertificate};

let cert = Certificate::new("smtlib2", "sum_assoc", "z3:unsat");
let imported =
    unsafe { Certified::<SmtCertificate, IsAssociative, i32>::assume(1, cert) };
let _: Proven<IsAssociative, i32> = unsafe { imported.into_proven() };
```

This is the deliberate trust handoff in `karpal-verify`: external evidence is useful, but it is not silently conflated with Rust-native proof evidence.

## Where to go next

- [Proof & Verification](../reference/proof-verification.md) for the full API overview.
- [Verification CI Workflow](../reference/verification-ci.md) for CI-focused layout and reporting guidance.
- [Verification Schemas](../reference/verification-schemas.md) for report/manifest/sidecar compatibility details.
- [Trust Model](../dev/phase-12-trust-model.md) for the imported-proof boundary design note.


Karpal is licensed under AGPL-3.0-or-later. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).


