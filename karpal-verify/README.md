# karpal-verify

External prover bridge for the Karpal ecosystem.

`karpal-verify` is Karpal's external verification foundation crate. It introduces:

- a backend-agnostic **proof obligation IR**
- reusable **algebraic signatures** for trait-level law generation
- grouped **obligation bundles** for Semigroup / Monoid / Group / Semiring / Lattice laws
- exporters for **SMT-LIB2** and **Lean 4**
- structured Lean module/theorem metadata for a richer Lean bridge
- Lean import/prelude bridging for module imports and symbol aliases
- Lean project/package scaffolding for generated modules
- project-aware Lean execution planning via `lake env lean` or `lake build`
- artifact writers and **dry-run invocation plans** for external tools
- runner abstractions, backend-specific verification policies, and basic SMT result parsing
- reporting types that attach execution outcomes and certificates back to obligations
- session/orchestration helpers for build → run → report flows
- report serialization helpers for CI-friendly JSON / Markdown summaries
- optional **amari-flynn** integration for statistical contracts, Monte Carlo law-bound checks, and probabilistic contract macros
- an explicit **external trust boundary** for importing certificates back into Rust

## What's inside

### Proof obligations

`Obligation` captures a law to be discharged by an external backend:

```rust
use karpal_verify::{Obligation, Origin, Sort};

let assoc = Obligation::associativity(
    "sum_assoc",
    Origin::new("karpal-algebra", "Semigroup for Sum<i32>"),
    Sort::Int,
    "combine",
);
```

### Algebraic signatures

`AlgebraicSignature` lets exporters and higher-level integrations refer to
semantic roles like `combine`, `identity`, `inverse`, `add`, `mul`, `meet`,
and `join` instead of duplicating raw symbol strings:

```rust
use karpal_verify::{AlgebraicSignature, Obligation, Origin, Sort};

let sig = AlgebraicSignature::group(Sort::Int, "combine", "e", "inv");
let obligation = Obligation::left_inverse_in(
    "group_left_inverse",
    Origin::new("karpal-algebra", "Group for i32"),
    &sig,
);
assert_eq!(obligation.property, "left inverse");
```

### Obligation bundles

`ObligationBundle` packages related laws together so callers can generate and
export whole verification batches from one semantic signature:

```rust
use karpal_verify::{AlgebraicSignature, ObligationBundle, Origin, Sort};

let sig = AlgebraicSignature::monoid(Sort::Int, "combine", "e");
let bundle = ObligationBundle::monoid(
    "sum_monoid",
    Origin::new("karpal-core", "Monoid for Sum<i32>"),
    &sig,
);
assert_eq!(bundle.obligations().len(), 3);
```

### SMT-LIB2 export

```rust
use karpal_verify::{export_smt_obligation, Obligation, Origin, Sort};

let obligation = Obligation::commutativity(
    "sum_comm",
    Origin::new("karpal-algebra", "AbelianGroup for i32"),
    Sort::Int,
    "combine",
);

let smt = export_smt_obligation(&obligation);
assert!(smt.contains("(check-sat)"));
```

### Lean 4 export

```rust
use karpal_verify::{export_lean_module, Obligation, Origin, Sort};

let obligation = Obligation::associativity(
    "sum_assoc",
    Origin::new("karpal-algebra", "Semigroup for Sum<i32>"),
    Sort::Int,
    "combine",
);

let lean = export_lean_module("KarpalVerify", &[obligation]);
assert!(lean.contains("namespace KarpalVerify"));

let structured = karpal_verify::Lean4::export("KarpalVerify", &[obligation]);
assert_eq!(structured.theorems[0].witness_ref("KarpalVerify"), "KarpalVerify.sum_assoc");
```

You can also drive the Lean prelude explicitly when a module needs imports or
raw IR symbols need stable Lean-facing aliases:

```rust
use karpal_verify::{export_module_with_prelude, LeanPrelude, Obligation, Origin, Sort};

let obligation = Obligation::associativity(
    "sum_assoc",
    Origin::new("karpal-algebra", "Semigroup for Sum<i32>"),
    Sort::Int,
    "combine-op",
);

let module = export_module_with_prelude(
    "KarpalVerify",
    &[obligation],
    LeanPrelude::new().with_import("Mathlib"),
);
assert!(module.starts_with("import Mathlib"));
assert!(module.contains("abbrev sym_combine_op := «combine-op»"));
```

### Batch export APIs

```rust
use karpal_verify::{
    export_lean_bundle, export_smt_bundle, AlgebraicSignature, ObligationBundle, Origin, Sort,
};

let sig = AlgebraicSignature::group(Sort::Int, "combine", "e", "inv");
let bundle = ObligationBundle::group(
    "sum_group",
    Origin::new("karpal-algebra", "Group for i32"),
    &sig,
);

let smt_scripts = export_smt_bundle(&bundle);
let lean_module = export_lean_bundle("KarpalVerify", &bundle);
let lean_export = karpal_verify::export_lean_bundle_structured("KarpalVerify", &bundle);
assert_eq!(smt_scripts.len(), 5);
assert!(lean_module.contains("theorem left_inverse"));
assert_eq!(lean_export.theorems[0].witness_ref("KarpalVerify"), "KarpalVerify.associativity");
```

### Artifact writing and dry runs

With the `std` feature, `karpal-verify` can write export artifacts to disk and
prepare dry-run command plans for solver / Lean invocation. Lean plans can also
run in project-aware mode through `lake env lean` or whole-package `lake build`
from the generated artifact root:

```rust
use karpal_verify::{
    dry_run_bundle_artifacts, AlgebraicSignature, ArtifactLayout, LeanConfig, ObligationBundle,
    Origin, SmtConfig, Sort,
};

let sig = AlgebraicSignature::monoid(Sort::Int, "combine", "e");
let bundle = ObligationBundle::monoid(
    "sum_monoid",
    Origin::new("karpal-core", "Monoid for Sum<i32>"),
    &sig,
);
let layout = ArtifactLayout::new("target/karpal-verify");
let batch = dry_run_bundle_artifacts(
    &bundle,
    &layout,
    "KarpalVerify",
    &SmtConfig::default(),
    &LeanConfig::default().with_driver(karpal_verify::LeanDriver::LakeBuild),
);
assert_eq!(batch.plans.len(), 4);
assert!(batch.lean_project.is_some());
assert!(batch.plans.iter().any(|plan| {
    plan.kind == karpal_verify::CommandKind::Lean
        && plan.executable == "lake"
        && plan.args == vec!["build", "KarpalVerify"]
}));
```

Use `LeanDriver::LakeEnv` when you want `lake env lean lean/<Module>.lean`, or
`LeanDriver::LakeBuild` when you want package-aware target builds through
`lake build <Module>`.


### Execution model

`VerifierRunner` abstracts over dry-run and local process execution:

```rust
use karpal_verify::{CommandKind, DryRunner, InvocationPlan, VerifierRunner};

let plan = InvocationPlan {
    kind: CommandKind::Smt,
    executable: "z3".into(),
    args: vec!["-smt2".into(), "target/example.smt2".into()],
    working_directory: None,
    input_files: vec!["target/example.smt2".into()],
};

let result = DryRunner.run(&plan);
assert_eq!(result.status, karpal_verify::ExecutionStatus::DryRun);
```

For SMT backends, `parse_smt_status()` recognizes `sat`, `unsat`, and `unknown`,
while `parse_smt_output()` also extracts simple model / `:reason-unknown`
information. Lean runs now also support `parse_lean_output(stdout, stderr)`,
which captures structured diagnostics and theorem-name hits from Lean / lake
messages. Reporting then maps those diagnostics back onto exported Lean theorem
metadata instead of relying only on raw obligation names. Successful results can
be turned into lightweight certificates.

### Reporting layer

`VerificationReport` attaches artifact paths, execution results, and generated
certificates back to each obligation in a bundle:

```rust
use karpal_verify::{
    dry_run_bundle_artifacts, dry_run_report, AlgebraicSignature, ArtifactLayout, LeanConfig,
    ObligationBundle, Origin, SmtConfig, Sort,
};

let sig = AlgebraicSignature::semigroup(Sort::Int, "combine");
let bundle = ObligationBundle::semigroup(
    "sum_semigroup",
    Origin::new("karpal-core", "Semigroup for Sum<i32>"),
    &sig,
);
let artifacts = dry_run_bundle_artifacts(
    &bundle,
    &ArtifactLayout::new("target/karpal-verify"),
    "KarpalVerify",
    &SmtConfig::default(),
    &LeanConfig::default(),
);
let report = dry_run_report(&bundle, &artifacts);
assert_eq!(report.obligation_count(), 1);
assert!(report.to_json().contains("bundle_name"));
assert!(report.to_markdown().contains("Verification Report"));
```

### Verification session orchestration

For a higher-level build → run → report flow, use `VerificationSession` or the
one-shot `verify_bundle(...)` helper:

```rust
use karpal_verify::{
    verify_bundle, AlgebraicSignature, ArtifactLayout, DryRunner, LeanConfig,
    ObligationBundle, Origin, SmtConfig, Sort,
};

let sig = AlgebraicSignature::semigroup(Sort::Int, "combine");
let bundle = ObligationBundle::semigroup(
    "sum_semigroup",
    Origin::new("karpal-core", "Semigroup for Sum<i32>"),
    &sig,
);
let report = verify_bundle(
    &bundle,
    &ArtifactLayout::new("target/karpal-verify-session"),
    "KarpalVerify",
    &SmtConfig::default(),
    &LeanConfig::default(),
    &DryRunner,
)
.expect("verification session should succeed");
assert_eq!(report.obligation_count(), 1);
```

`VerificationSession::verify_with_ci_outputs(...)` also writes JSON / Markdown
summaries directly beside generated artifacts. When a Lean module report is
present it also writes a `*.lean-diagnostics.json` sidecar containing module-
level diagnostics, theorem failure refs, and per-obligation Lean diagnostic
groupings. The main JSON / Markdown summaries now cross-link both that sidecar
and the generated Lean manifest path, and the Lean manifest now links back to
those CI-oriented report files as well. The serialized report JSON, Lean
manifest JSON, and Lean diagnostics sidecar all now include explicit
`schema_version` markers. Lean artifact batches now also carry structured
theorem metadata, prelude/import metadata, generated package metadata, and
write a small typed Lean manifest model alongside the module source plus
`lakefile.lean` / `lean-toolchain` scaffolding at the artifact root.

### amari-flynn statistical integration

Enable the optional `amari` feature when you want to layer statistical
verification on top of the obligation IR:

```toml
karpal-verify = { version = "0.3.0", features = ["amari"] }
```

This exposes:

- `verify_rare_event(...)` for Monte Carlo checking of law-violation predicates
- `StatisticalBound` / `StatisticalVerification` for probability bounds,
  event classification, and Karpal tier mapping
- `precondition_obligation_for(...)`, `postcondition_obligation_for(...)`,
  `concentration_obligation_for(...)`, and `expected_value_obligation_for(...)`
  for bridging Karpal obligations into amari-flynn SMT proof obligations
- re-exported `#[prob_requires(...)]`, `#[prob_ensures(...)]`, and
  `#[ensures_expected(...)]` macros for probabilistic contract helpers

```rust
use karpal_verify::{
    verify_rare_event, Obligation, Origin, Sort, StatisticalBound, VerificationTier,
};

let obligation = Obligation::commutativity(
    "sum_comm",
    Origin::new("karpal-algebra", "AbelianGroup for i32"),
    Sort::Int,
    "combine",
);
let verification = verify_rare_event(
    &obligation,
    &StatisticalBound::new(0.05).with_samples(4096),
    || false,
);
assert_eq!(verification.tier(), VerificationTier::Impossible);
```

This gives `karpal-verify` a concrete bridge for the roadmap's three-tier
story:

- **Impossible** → type-level / zero-probability exclusion
- **Rare** → amari-flynn statistical bounds over violating events
- **Emergent** → behavior that remains possible and therefore observable

For bundle-level summaries, `three_tier_report(...)` combines declared
obligation tiers, amari statistical evidence, and successful external
verification results into a single aggregate report with JSON / Markdown
rendering helpers.

#### Schema compatibility

Current serialized verification artifacts use schema version `1`.

Version `1` guarantees:
- a top-level string `schema_version`
- stable existing field names within the `1.x` line
- additive evolution via new optional fields only
- nested `report_files` objects also include their own `schema_version`

Consumers should accept `schema_version == "1"`, ignore unknown optional
fields, and treat a future schema bump as a breaking parser boundary. See
`docs/dev/verification-schema-versioning.md` for the fuller compatibility
policy.

### Imported trust markers

External evidence does not silently become a Karpal proof witness.
Certificates can also carry provenance like backend version, obligation digest,
and artifact path. Crossing the boundary into `Proven<P, T>` remains explicit
and `unsafe`:

```rust
use karpal_proof::{IsAssociative, Proven};
use karpal_verify::{Certificate, Certified, LeanCertificate};

let cert = Certificate::new("lean4", "sum_assoc", "Sum.assoc");
let externally_checked =
    unsafe { Certified::<LeanCertificate, IsAssociative, i32>::assume(1, cert) };
let _: Proven<IsAssociative, i32> = unsafe { externally_checked.into_proven() };
```

## Current scope

This crate now provides the external-verification foundation for Karpal:

- obligation modeling
- SMT-LIB2 export
- structured Lean 4 export, project scaffolding, and project-aware execution
- CI/report artifact generation with schema-versioned JSON / Markdown outputs
- explicit trust-model types
- optional amari-flynn statistical integration for rare-event bounds and
  probabilistic contract helpers

The core roadmap work for `karpal-verify` is now implemented: obligation IR,
SMT/Lean export, artifact/report/session orchestration, optional amari-flynn
integration, and CI-oriented three-tier summaries. Future work can deepen that
coverage across more derive and trait workflows, but the external verification
foundation is now in place.

## License

MIT OR Apache-2.0
