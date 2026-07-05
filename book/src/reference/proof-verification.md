# Proof & Verification

Karpal now has complementary layers for reasoning about laws. `karpal-proof` provides in-Rust witnesses, refinement types, and derive-based law checks. `karpal-verify` extends that story outward with an obligation IR, exporters for SMT-LIB2 and Lean 4, optional amari-flynn statistical verification hooks, artifact generation, execution planning, reporting, three-tier bundle summaries, and an explicit trust boundary for imported certificates. Together these APIs now cover Karpal's full external verification foundation.

## Overview

| Crate                 | Role                                           | Typical use                                                                                                              |
|-----------------------|------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------|
| `karpal-proof`        | Internal law witnesses and refinement evidence | Encode that a value is known to satisfy a property inside Rust                                                           |
| `karpal-proof-derive` | Derive-driven law verification helpers         | Generate tests that check algebraic laws for your types                                                                  |
| `karpal-verify`       | External verification bridge                   | Export obligations to external provers, bridge rare-event checks through amari-flynn, and import certificates explicitly |

## Crate map

| Crate           | Focus                                                                                                |
|-----------------|------------------------------------------------------------------------------------------------------|
| `karpal-proof`  | Law witnesses, rewrite evidence, refinement types, and derive-based law verification                 |
| `karpal-verify` | Obligation IR, exporters, execution/reporting, orchestration, and explicit imported-trust boundaries |

## The `karpal-proof` layer

`karpal-proof` models law evidence as values and phantom markers. The core idea is that a property like associativity or monoid structure can be reflected in the type system without pretending the compiler proved it from first principles.


### Proven\<P, T\>

A value `T` paired with evidence for property marker `P`.


``` rust
use karpal_proof::{IsMonoid, Proven};

let checked: Proven<IsMonoid, i32> = Proven::from_monoid(5);
let value: i32 = checked.into_inner();
```

Property markers such as `IsAssociative`, `IsMonoid`, `IsGroup`, and `IsSemiring` let downstream APIs require evidence rather than a raw trait bound.


### Refinement types

Small runtime-checked wrappers for stronger domain invariants.


``` rust
use karpal_proof::{NonEmpty, Positive};

let xs = NonEmpty::try_new(vec![1, 2, 3]).expect("vector is non-empty");
let p = Positive::new(42).expect("value is positive");
```

These wrappers are useful even when you are not doing external verification: they make illegal states unrepresentable after construction.


### Rewrite witnesses

Composable evidence for algebraic rewriting steps.


Rewrites capture law-guided transformations such as associativity, commutativity, identity elimination, distributivity, and inverse cancellation. They are a good fit for normalization, symbolic simplification, and proof-oriented APIs inside Rust.


### Derive-based law checks

`karpal-proof-derive` provides macros like `VerifySemigroup`, `VerifyMonoid`, `VerifyGroup`, `VerifySemiring`, and `VerifyLattice`. These derive helpers generate tests that exercise the relevant algebraic laws for your type.

``` rust
use karpal_proof::VerifyMonoid;

#[derive(Clone, Debug, PartialEq, Eq, VerifyMonoid)]
struct SumI32(i32);
```

This remains a Rust-native, test-oriented workflow: it is excellent for continuous checking and regression prevention, but it is distinct from importing a theorem prover result.

## The `karpal-verify` layer

`karpal-verify` is Karpal's bridge for external verification. It deliberately separates *modeling*, *export*, *execution*, and *trust* so each step stays inspectable.

### Obligation IR

The core intermediate representation is backend-agnostic and can describe algebraic laws without committing to a specific prover syntax.

``` rust
use karpal_verify::{Obligation, Origin, Sort};

let assoc = Obligation::associativity(
    "sum_assoc",
    Origin::new("karpal-algebra", "Semigroup for Sum<i32>"),
    Sort::Int,
    "combine",
);
```

The IR includes:

- `Obligation` for named proof goals
- `Origin` for provenance
- `Declaration`, `Sort`, and `Term` for signatures and formulas
- `VerificationTier` and `ProofDialect` for classification metadata

### Algebraic signatures and bundles

`AlgebraicSignature` registers semantic roles like `combine`, `identity`, `inverse`, `add`, `mul`, `meet`, and `join`. `ObligationBundle` then groups the relevant laws for a structure such as a semigroup, monoid, group, semiring, or lattice.

``` rust
use karpal_verify::{AlgebraicSignature, ObligationBundle, Origin, Sort};

let sig = AlgebraicSignature::group(Sort::Int, "combine", "e", "inv");
let bundle = ObligationBundle::group(
    "sum_group",
    Origin::new("karpal-algebra", "Group for i32"),
    &sig,
);
assert_eq!(bundle.obligations().len(), 5);
```

### Exporters

The same obligation bundle can be exported to different backends:

- **SMT-LIB2** via `SmtLib2` and `export_smt_bundle(...)`
- **Lean 4** via `Lean4`, `export_lean_bundle(...)`, and the structured Lean export APIs

``` rust
use karpal_verify::{export_smt_bundle, export_lean_bundle};

let smt_scripts = export_smt_bundle(&bundle);
let lean_module = export_lean_bundle("KarpalVerify", &bundle);
```

### Lean integration

The Lean bridge is more than plain text export. Structured Lean metadata tracks theorem identities, declaration spans, module imports, symbol aliases, project/package information, and report cross-links. This lets Karpal preserve a stable connection between exported obligations, generated Lean source, CI artifacts, and parsed Lean diagnostics.

- **Prelude/import bridging** via `LeanPrelude`, `LeanImport`, and `LeanAlias`
- **Structured theorem metadata** via `LeanTheorem` and `LeanExport`
- **Project scaffolding** via `LeanProject` plus generated `lakefile.lean` and `lean-toolchain`
- **Project-aware execution** through `LeanDriver::LakeEnv` and `LeanDriver::LakeBuild`
- **Parsed diagnostics** via `parse_lean_output(...)`, including theorem-name hits and line-aware fallback mapping
- **CI sidecars and manifests** through schema-versioned report JSON, Lean manifest JSON, and Lean diagnostics sidecar JSON

### Artifacts, planning, and execution

With the `std` feature, `karpal-verify` can prepare artifact layouts, write files, build invocation plans, and execute those plans either as dry runs or as local processes.

Those serialized artifacts are schema-versioned as well: report JSON, Lean manifest JSON, and Lean diagnostics sidecars all carry `schema_version` markers so CI tooling can detect compatible vs. breaking format changes explicitly.

| Type                 | Responsibility                                                     |
|----------------------|--------------------------------------------------------------------|
| `ArtifactLayout`     | Directory layout for generated SMT and Lean artifacts              |
| `ArtifactBatch`      | Records plus invocation plans for a verification batch             |
| `InvocationPlan`     | Executable, args, working directory, and tracked input files       |
| `DryRunner`          | Returns shell-rendered dry-run results without spawning processes  |
| `LocalProcessRunner` | Executes local solver or Lean commands via `std::process::Command` |

### Backend-specific verification policies

`karpal-verify` defines explicit backend policies so success is not interpreted uniformly across all tools:

- **SMT**: a verification success means the negated obligation is `unsat`
- **Lean**: a verification success means the process exits successfully and parsed Lean diagnostics report no errors

``` rust
use karpal_verify::{CommandKind, ExecutionStatus, VerificationPolicy};

assert!(VerificationPolicy::for_kind(CommandKind::Smt)
    .accepts(ExecutionStatus::Unsat));
assert!(VerificationPolicy::for_kind(CommandKind::Lean)
    .accepts(ExecutionStatus::Success));
```

SMT output parsing also records richer detail through `SmtOutput`, including the parsed status, simple model text after `sat`, and `:reason-unknown` metadata. Lean parsing records structured diagnostics, theorem hits, and location-aware fallback matching so reports can attach failures back to the correct exported theorem even when Lean emits only source locations.

### Reporting and orchestration

The reporting layer attaches results, artifact paths, and optional certificates back to each obligation. The new session/orchestration layer then offers a higher-level workflow for build → run → report.

``` rust
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
    &ArtifactLayout::new("target/karpal-verify"),
    "KarpalVerify",
    &SmtConfig::default(),
    &LeanConfig::default(),
    &DryRunner,
).expect("verification session should succeed");
assert_eq!(report.obligation_count(), 1);
```

`VerificationSession::verify_with_ci_outputs(...)` additionally writes JSON and Markdown summaries directly beside the generated artifacts, plus a schema-versioned Lean diagnostics sidecar and a typed Lean manifest with cross-links back to the CI report files. See [Verification CI Workflow](verification-ci.md) for CI-specific guidance, artifact layout recommendations, and [Verification Schemas](verification-schemas.md) for the serialized compatibility contract.

## Explicit trust boundary

External certificates do **not** silently become Rust proof witnesses. Imported evidence first becomes `Certified<B, P, T>`, where `B` identifies the backend, `P` is the claimed property, and `T` is the wrapped value. Crossing into `Proven<P, T>` remains an explicit `unsafe` action.

``` rust
use karpal_proof::{IsAssociative, Proven};
use karpal_verify::{Certificate, Certified, LeanCertificate};

let cert = Certificate::new("lean4", "sum_assoc", "Sum.assoc");
let externally_checked =
    unsafe { Certified::<LeanCertificate, IsAssociative, i32>::assume(1, cert) };
let _: Proven<IsAssociative, i32> = unsafe { externally_checked.into_proven() };
```

This policy keeps imported trust searchable, reviewable, and distinct from evidence derived directly from Rust traits or runtime checks. For a design note focused specifically on the trust model, see [Trust Model](../dev/phase-12-trust-model.md).

## Recommended workflow

1.  Model the law as an `Obligation` or `ObligationBundle`.
2.  Export SMT-LIB2 scripts or a Lean module.
3.  Write artifacts and generate invocation plans.
4.  Execute with an explicit backend policy.
5.  Collect a `VerificationReport` and optional CI summaries.
6.  Import external evidence only through `Certified<...>`.
7.  Cross into `Proven<...>` only at carefully audited boundaries.

For a walkthrough-style example, see [Verification Workflow](../examples/verification-workflow.md).


Karpal is licensed under AGPL-3.0-or-later. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).


