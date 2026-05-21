# Phase 12 Extensions Implementation Plan

> **REQUIRED SUB-SKILL:** Use the executing-plans skill to implement this plan task-by-task.

**Goal:** Complete the revised Phase 12 extension sequence for the 0.5.0 release: Kani backend, trait-to-obligation macro, karpal-proof bridge, continuous verification CI, and GPU compute obligations.

**Architecture:** Keep `karpal-verify` as the normal verification library and add focused extension modules behind `std`/`alloc` gates where appropriate. Add a Rust-idiomatic companion proc-macro crate, `karpal-verify-derive`, then re-export `export_obligations` from `karpal-verify` so downstream code can use `#[karpal_verify::export_obligations]`. Build 12e GPU obligations on top of the generic IR/backend/session work rather than creating a separate verification stack.

**Tech Stack:** Rust nightly edition 2024, workspace Cargo crates, `karpal-verify`, `karpal-proof`, `karpal-proof-derive`, new `karpal-verify-derive`, optional external tools (`kani`, SMT solvers, Lean 4) represented through invocation plans and capability-gated tests.

---

## Phase Order

Implement the roadmap order exactly, because each extension builds on the previous one:

1. **12a — Kani backend**
2. **12b — Trait-to-obligation derive**
3. **12c — karpal-proof bridge**
4. **12d — Continuous verification CI**
5. **12e — GPU compute obligations**

Each numbered task below is intended to become one small commit or PR slice.

---

### Task 1: Add Kani backend model and exports

**Files:**
- Create: `karpal-verify/src/kani.rs`
- Modify: `karpal-verify/src/lib.rs`
- Modify: `karpal-verify/src/trust.rs`
- Test: `karpal-verify/src/kani.rs`
- Test: `karpal-verify/src/trust.rs`

**Steps:**
1. Write failing tests for `KaniCertificate::NAME == "kani"`, `export_kani_harness(&obligation)`, and `export_kani_bundle(&bundle)`.
2. Run `cargo test -p karpal-verify kani --lib` and confirm failure.
3. Implement `Kani`, `KaniHarness`, `export_kani_harness`, `export_kani_bundle`, and a simple IR-to-Kani source renderer using `#[kani::proof]`, `kani::any()`, and `kani::assert(...)`.
4. Add `KaniCertificate` to `trust.rs` and re-export Kani APIs from `lib.rs` behind `std`/`alloc`.
5. Run `cargo fmt --check --all`, `cargo test -p karpal-verify kani --lib`, and `cargo build --no-default-features -p karpal-verify --features alloc`.
6. Commit: `feat: add kani verification backend`.

---

### Task 2: Add Kani invocation planning and reports

**Files:**
- Modify: `karpal-verify/src/command.rs`
- Modify: `karpal-verify/src/runner.rs`
- Modify: `karpal-verify/src/artifact.rs`
- Modify: `karpal-verify/src/report.rs`

**Steps:**
1. Write failing tests for `CommandKind::Kani`, default `KaniConfig`, Kani invocation rendering, Kani verification policy, and dry-run Kani artifacts.
2. Run `cargo test -p karpal-verify kani --lib` and confirm failure.
3. Add `CommandKind::Kani` and `KaniConfig`; default plan should be `cargo kani --harness <harness_name>`.
4. Extend `VerificationPolicy::for_kind` so Kani accepts `ExecutionStatus::Success` with witness suffix `ok`.
5. Write Kani harness artifacts under a stable `kani/` artifact directory, preferably one harness file per obligation for certificate mapping.
6. Ensure report generation can attach Kani results/certificates without disrupting SMT/Lean output.
7. Run `cargo fmt --check --all`, `cargo test -p karpal-verify kani --lib`, and `cargo test -p karpal-verify artifact --lib`.
8. Commit: `feat: plan kani verifier invocations`.

---

### Task 3: Add `karpal-verify-derive` crate and macro re-export

**Files:**
- Modify: root `Cargo.toml`
- Create: `karpal-verify-derive/Cargo.toml`
- Create: `karpal-verify-derive/README.md`
- Create: `karpal-verify-derive/src/lib.rs`
- Modify: `karpal-verify/Cargo.toml`
- Modify: `karpal-verify/src/lib.rs`
- Test: `karpal-verify-derive/tests/export_obligations.rs`

**Steps:**
1. Write a failing integration test using `use karpal_verify::export_obligations;` and an annotated item that generates `Type::karpal_obligation_bundle()`.
2. Run `cargo test -p karpal-verify-derive` and confirm the crate/macro does not exist.
3. Add `karpal-verify-derive` as a proc-macro crate with `syn`, `quote`, and `proc-macro2`, following the `karpal-proof-derive` pattern.
4. Add a `derive = ["std", "dep:karpal-verify-derive"]` feature to `karpal-verify` and include it in default features so `#[karpal_verify::export_obligations]` works by default.
5. Implement `#[export_obligations(...)]` as an attribute macro that leaves the annotated item intact and emits an inherent `karpal_obligation_bundle()` method.
6. Support explicit metadata first: `crate_name`, `item_path`, `carrier`, and `semigroup(...)` / `monoid(...)` forms.
7. Run `cargo fmt --check --all`, `cargo test -p karpal-verify-derive`, and `cargo test -p karpal-verify`.
8. Commit: `feat: add verification obligation derive crate`.

---

### Task 4: Extend derive macro coverage for algebraic signatures

**Files:**
- Modify: `karpal-verify-derive/src/lib.rs`
- Test: `karpal-verify-derive/tests/export_obligations.rs`
- Modify: `karpal-verify-derive/README.md`

**Steps:**
1. Add failing tests for `group`, `semiring`, `ring`, and `lattice` macro attributes.
2. Run `cargo test -p karpal-verify-derive` and confirm missing coverage.
3. Extend parser to support explicit forms such as `group(op = "combine", identity = "empty", inverse = "invert")`, `semiring(add = "add", zero = "zero", mul = "mul", one = "one")`, and `lattice(meet = "meet", join = "join")`.
4. Map attributes to existing `AlgebraicSignature` and `ObligationBundle` constructors.
5. Document macro examples in `karpal-verify-derive/README.md`.
6. Run `cargo fmt --check --all` and `cargo test -p karpal-verify-derive`.
7. Commit: `feat: export algebraic obligation bundles from macros`.

---

### Task 5: Add karpal-proof bridge certificates

**Files:**
- Modify: `karpal-verify/src/trust.rs`
- Create: `karpal-verify/src/proof_bridge.rs`
- Modify: `karpal-verify/src/lib.rs`

**Steps:**
1. Write failing tests showing `ProofEvidence::passed_tests(...)` can generate a `Certificate` with backend `karpal-proof` and a stable witness reference.
2. Run `cargo test -p karpal-verify proof_bridge --lib` and confirm failure.
3. Add `ProofTestCertificate` implementing `VerificationBackend` with `NAME = "karpal-proof"`.
4. Add `ProofEvidence` fields for test path, case/sample count, optional seed, and notes.
5. Add `ProofBridge::certificate(...)` helpers that create certificate metadata from an `Obligation` and `ProofEvidence`.
6. Do not auto-create `Proven<P, T>`; keep `Certified::into_proven()` unsafe and explicit.
7. Run `cargo fmt --check --all`, `cargo test -p karpal-verify proof_bridge --lib`, and `cargo test -p karpal-proof-derive`.
8. Commit: `feat: bridge proof-derived evidence into certificates`.

---

### Task 6: Add CI verification workflow and capability gating

**Files:**
- Create or modify: `.github/workflows/verification.yml`
- Modify: `README.md`
- Modify: `karpal-verify/README.md`
- Test/golden files under: `karpal-verify/tests/`

**Steps:**
1. Add/extend golden tests asserting CI sidecar/report files include Kani paths when Kani plans exist and still include Lean/SMT report links.
2. Run `cargo test -p karpal-verify --test export_golden` and confirm expected failures if golden files need updating.
3. Add a verification workflow that runs `cargo test -p karpal-verify` and capability-gated Lean/Kani smoke steps.
4. Keep external tools optional at first: missing `lake`, SMT solvers, or `cargo-kani` should skip smoke checks rather than fail required CI.
5. Document local verification commands in `README.md` and `karpal-verify/README.md`.
6. Run `cargo fmt --check --all` and `cargo test -p karpal-verify`.
7. Commit: `ci: add continuous verification workflow`.

---

### Task 7: Add GPU compute obligation builders

**Files:**
- Create: `karpal-verify/src/gpu.rs`
- Modify: `karpal-verify/src/lib.rs`
- Modify: `karpal-verify/README.md`
- Test: `karpal-verify/src/gpu.rs`
- Test/golden: `karpal-verify/tests/export_golden.rs`

**Steps:**
1. Write failing tests for a `GpuObligationBundle::metal_kernel(...)` builder with buffer alignment, workgroup divisibility, dispatch limit, and kernel determinism obligations.
2. Run `cargo test -p karpal-verify gpu --lib` and confirm failure.
3. Add GPU-specific property marker names in `karpal-verify::gpu` rather than `karpal-proof`, unless a property is clearly reusable outside GPU verification.
4. Represent GPU concepts using existing IR: `Sort::Named("MTLBuffer")`, `Term::app("aligned_to", ...)`, `Term::app("divisible_by", ...)`, `Term::app("within_dispatch_limit", ...)`, and `Term::app("deterministic_kernel", ...)`.
5. Expose a builder producing a normal `ObligationBundle` with stable names/origins suitable for Borsalino and similar downstream crates.
6. Add tests exporting the GPU bundle through SMT, Lean, and Kani exporters without panics.
7. Run `cargo fmt --check --all`, `cargo test -p karpal-verify gpu --lib`, `cargo test -p karpal-verify --test export_golden`, and `cargo build --no-default-features -p karpal-verify --features alloc`.
8. Commit: `feat: add gpu compute verification obligations`.

---

### Task 8: 0.5.0 release alignment and roadmap status

**Files:**
- Modify: `Cargo.toml`
- Modify: crate `Cargo.toml` files if needed
- Modify: `ROADMAP.md`
- Modify: `README.md`
- Modify: `CONTRIBUTING.md` if release language needs adjustment

**Steps:**
1. Update roadmap status so Phase 12 extensions 12a-12e are marked implemented, and Phase 13 remains in progress unless separately completed.
2. Add 0.5.0 release notes language covering AGPL relicensing and the Phase 12 extension sequence.
3. If this is the actual release-prep branch, bump workspace version from `0.4.0` to `0.5.0`; otherwise leave version bump for a release PR.
4. Run full verification:
   - `cargo fmt --check --all`
   - `cargo clippy --workspace -- -D warnings`
   - `cargo test --workspace`
   - `cargo build --no-default-features -p karpal-core -p karpal-profunctor -p karpal-diagram -p karpal-verify`
   - `cargo build --no-default-features -p karpal-verify --features alloc`
5. Commit: `chore: prepare phase 12 extensions for 0.5.0`.

---

## Execution Notes

- Keep every external tool optional/capability-gated unless the repo explicitly installs it in CI.
- Prefer generated source/string artifacts for Kani over taking a hard dependency on Kani crates.
- Preserve `no_std`/`alloc` behavior for IR/export data structures.
- Keep all trust-boundary crossings explicit; do not auto-create `Proven<P, T>` from external or test evidence without `unsafe`.
- Make the GPU obligation API generic enough for Borsalino while naming Metal concepts (`MTLBuffer`, MSL kernels) in term/sort strings.

## Required Final Verification

Before claiming the sequence is complete, run:

```bash
cargo fmt --check --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo build --no-default-features -p karpal-core -p karpal-profunctor -p karpal-diagram -p karpal-verify
cargo build --no-default-features -p karpal-verify --features alloc
```
