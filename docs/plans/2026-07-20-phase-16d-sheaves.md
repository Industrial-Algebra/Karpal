# Phase 16D — Grothendieck Topologies & Sheaves: Design

**Date:** 2026-07-20
**Sub-phase:** 16D (completes `karpal-topos` and Phase 16)
**Dependencies:** 16B (`Sieve`, `Presheaf`, `SmallCategory`), 16C (`Omega`, `TruthValue`, `equalizer_fiber`), karpal-core (`Adjunction`)
**Status:** design, pre-implementation

## Goal

Add Grothendieck topologies, sheaves, the Lawvere-Tierney correspondence, and
the sheafification concept to `karpal-topos`. This completes the topos
machinery: a topos is a category with finite limits (16C) and a subobject
classifier (16C) over which one can define topologies and sheaves (16D).

## What 16D delivers

### Grothendieck topology

A Grothendieck topology `J` assigns to each object `c` a collection `J(c)` of
**covering sieves** satisfying three axioms:

1. **Maximality**: the maximal sieve on `c` is covering.
2. **Stability (base change)**: if `S ∈ J(c)` and `f: d → c`, then the pullback
   sieve `f*S ∈ J(d)`.
3. **Transitivity**: if `S ∈ J(c)` and `R` is a sieve such that `f*R ∈ J(d)`
   for every `f: d → c` in `S`, then `R ∈ J(c)`.

Encoding: a trait over `ChainCat<N>` with `is_covering(i, rank) -> bool`. Two
concrete instances:

- **Trivial topology**: only the maximal sieve covers (`rank == i+1`).
- **Dense topology**: any non-empty sieve covers (`rank >= 1`).

Both are verified to satisfy all three axioms by test.

### Lawvere-Tierney topology

A Lawvere-Tierney topology is a closure operator `j: Ω → Ω` satisfying:

- `j(true) = true` (top is closed)
- `j ∘ j = j` (idempotence)
- `j(a ∧ b) = j(a) ∧ j(b)` (meet-preserving)

For `ChainCat`, this is a function `j_i: {0..=i+1} → {0..=i+1}` on `TruthValue`
ranks. Concrete instances correspond to the Grothendieck topologies above:

- **Trivial (LT)**: `j_i(r) = i+1` for all `r` (everything closes to top).
- **Dense (LT)**: `j_i(r) = max(r, 1)` (the empty sieve closes to rank 1).

Tested for all three axioms.

### Sheaf

A presheaf `P` is a **sheaf** for topology `J` if, for every covering sieve
`S ∈ J(c)`, every compatible family of elements (one per object in `S`,
agreeing on overlaps) glues uniquely to an element of `P(c)`.

The sheaf condition is an **equalizer** condition — 16C's `equalizer_fiber` is
the building block. Encoding: a function `is_sheaf_at` that, given the
presheaf's restriction action and a covering sieve, checks the unique-gluing
property by enumerating compatible families.

Tested on concrete presheaves (`ConstantPresheaf`, `InitialSegmentPresheaf`)
against both the trivial and dense topologies.

### Sheafification

The sheafification `a: PSh(C) → Sh(C, J)` is the left adjoint to the inclusion
`Sh(C,J) ↪ PSh(C)`. Full algorithmic sheafification (the plus-construction,
applied twice) is genuinely complex; 16D provides:

- The **adjunction interface** connecting to `karpal-core::Adjunction` (unit,
  counit, triangle identities) — documented, not fully algorithmic.
- The **separated-presheaf** concept (the first step of sheafification:
  quotient by elements that are locally equal) as an honest, partial
  construction.

This is an honest scope: the *concept* of sheafification as an adjunction is
provided; the full plus-construction algorithm is flagged as future work.

## The Lawvere-Tierney correspondence

There is a bijection between Grothendieck topologies and Lawvere-Tierney
topologies on a topos: a sieve `S` is `J`-covering iff `j(S)` is the maximal
sieve. 16D documents this correspondence and tests it on the concrete
instances where it is unambiguous (trivial: only max covers, j(max)=max;
dense: non-empty covers, j(0)≠max but j(r)=max for r≥1... checked case by
case).

## Deliverables

- `karpal-topos/src/topology.rs` — `GrothendieckTopology` trait, `TrivialTopology`,
  `DenseTopology`, `LawvereTierneyTopology` trait + instances, axiom tests
- `karpal-topos/src/sheaf.rs` — `is_sheaf_at` sheaf-condition checker,
  separated-presheaf concept, sheafification adjunction interface
- Tests for all topology axioms, the sheaf condition on concrete presheaves
- ROADMAP update (mark 16D complete — Phase 16 fully done)

## Connections

- **16B**: topologies are collections of sieves; sheaves are presheaves.
- **16C**: the sheaf condition is an equalizer; Lawvere-Tierney is `j: Ω → Ω`
  on `TruthValue`.
- **karpal-core**: sheafification is an `Adjunction`.
- **Structured emptiness**: a Grothendieck topology axiomatizes *which* kinds
  of emptiness (sieves) count as "covering" — making the reason-for-emptiness
  first-class in the covering structure.

## Out of scope (future)

- Full plus-construction sheafification algorithm
- Stalks and the espace étalé
- Cohomology of sheaves
