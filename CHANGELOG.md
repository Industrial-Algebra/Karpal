# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.1] — 2026-07-01

### Added

- `IsNumericallyCorrect` property type in `karpal-verify` for GPU kernel numerical correctness verification
- `GpuObligationBundle::with_numerical_correctness()` builder method (DeepReinforce exact-match protocol)

## [0.6.0] — 2026-06-28

### License Change

**Karpal is now licensed under Apache-2.0 + CLA.** Previous releases (0.5.x)
were licensed under AGPL-3.0-or-later; those versions remain available on
crates.io permanently. Starting with 0.6.0, all code ships under Apache-2.0
to maximize enterprise adoption. See [CONTRIBUTING.md](CONTRIBUTING.md) for
the CLA.

### Added — Phase 15: `karpal-higher`

New crate implementing 2-categories, enriched categories, bicategories, and
higher functors/monads:

- **`TwoCategory`** trait — strict 2-categories with objects, 1-morphisms,
  and 2-morphisms. `Cat` instance (the 2-category of categories).
- **`Bicategory`** trait — weakened 2-categories with associator, left/right
  unitors as isomorphisms.
- **`EnrichedCategory<V>`** trait — categories enriched over a monoidal base.
  `SetEnrichment`/`SetCategory` (ordinary categories), `MonoidEnrichment`.
- **`FFunctor<C1, C2>`** — functor between 2-categories preserving
  1-morphisms and 2-morphisms. `IdentityFFunctor` instance.
- **`FMonad<C>`** — monad in the endofunctor 2-category with `unit` and
  `multiply` 2-morphisms.
- **Coherence witnesses** — `InterchangeIdentity`, `BicategoryPentagonIdentity`,
  `BicategoryTriangleIdentity` as type-level `karpal-proof::Justifies` witnesses.
- **Verification integration** — `HigherCoherenceCertificate` backend and
  `higher_coherence_certificates()` generating `Certificate`s via `ProofBridge`.

### Added — `karpal-index` CLI

New binary for AI-agent library discovery:

- `karpal-index search <query>` — fuzzy search types/traits/functions
- `karpal-index detail <name>` — full signature, docs, methods, implementors
- `karpal-index crates` — workspace crate listing
- `karpal-index hierarchy <trait>` — supertraits, subtraits, implementors

### Changed

- Relicensed entire workspace from AGPL-3.0-or-later to Apache-2.0
- All source files now carry `SPDX-License-Identifier: Apache-2.0` headers
- CONTRIBUTING.md updated with CLA reference
- Publish workflow updated with correct dependency ordering and all crates
- Version bumped 0.5.0 → 0.6.0 across all 17 workspace crates

## [0.5.0] — 2026-05-24

### Added — Phase 12 extensions

- Kani verification backend with harness generation
- GPU compute obligation builders for Metal/MSL kernel contracts
- `karpal-verify-derive` companion proc-macro crate (`#[export_obligations]`)
- `karpal-proof` bridge (`ProofBridge`, `ProofEvidence`)
- Continuous verification CI workflow

### Added — Phase 13: `karpal-diagram`

- Monoidal category traits: `Tensor`, `Braiding`, `Symmetry`, `Trace`
- String-diagram DSL with `Identity`, `Box`, `Sequence`, `Parallel`, `Swap`,
  `Cup`, `Cap` nodes
- Runtime diagram normalization with `NormalizationTrace` and 6 rewrite rules
- Compact-closed cup/cap yanking normalization
- Text and SVG renderers
- Type-level coherence witnesses: `PentagonIdentity`, `TriangleIdentity`,
  `HexagonIdentity`
- Diagrammatic rewriting bridge: `ByNormalization`, `ByYanking`
- Verification integration: `CoherenceCertificate`, `coherence_certificates()`

### Added — Phase 14: `karpal-schubert-types`

- `SchubertType` — Schubert classes in Grassmannians via `amari-enumerative`
- `Intersection` / `IntersectionKind` — LR coefficient intersection
- `SchubertTyped` trait + `SchubertProven<M, T>` proof-carrying values
- `compose_checks()` — chained type-check composition via LR rule
- External verification: `schubert_bundle()` + `verify_schubert()`

### Changed

- Relicensed workspace from `MIT OR Apache-2.0` to `AGPL-3.0-or-later`
- Version bumped 0.4.0 → 0.5.0
