# Phase 16C — Subobject Classifier & Finite Limits: Design

**Date:** 2026-07-20
**Sub-phase:** 16C (extends `karpal-topos`)
**Dependencies:** 16B (`SmallCategory`, `Presheaf`, `Representable`, `Sieve`), 16A (`HeytingAlgebra`)
**Status:** design, pre-implementation

## Goal

Add the subobject classifier Ω and finite-limit constructions (pullback,
equalizer) to `karpal-topos`. These are the remaining ingredients that make a
category a topos (finite limits + subobject classifier), completing the
categorical home for structured emptiness.

## The key fact

In a presheaf topos `[C^op, Set]`, the subobject classifier Ω is the presheaf:

```
Ω(c) = { sieves on c }
true_c = the maximal sieve on c
```

16B built sieves. 16C assembles them into Ω. This is not a coincidence — it is
*why* sieves exist in topos theory. The characteristic morphism χ: Sub(A) →
Hom(A, Ω) sends a subobject to its "support sieve."

For `ChainCat<N>`, a sieve on object `i` is a downward-closed subset of
`{0,…,i}`, which is determined by its rank `r ∈ {0,…,i+1}`. So Ω(i) is a
**chain lattice** of `i+2` truth values — a concrete instance of the
structured-emptiness lattice (16A's `HeytingAlgebra`), where rank 0 = empty
sieve = "Denied" and rank `i+1` = maximal sieve = "top".

## Encoding

### `Omega` presheaf

```rust
/// A truth value: a sieve represented by its rank (size of the down-set).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct TruthValue { pub rank: usize }

/// The subobject classifier Ω of the presheaf topos [ChainCat<N>^op, Set].
pub struct Omega;

impl<const N: usize> Presheaf<ChainCat<N>> for Omega {
    type At<Obj> = TruthValue;
    fn restrict<Dom, Cod>(f: ChainMor<Dom, Cod>, tv: TruthValue) -> TruthValue {
        // Pull back the sieve along f: Dom → Cod.
        // rank becomes min(rank, Dom::IDX + 1).
        TruthValue { rank: tv.rank.min(f.from() + 1) }
    }
}
```

Restriction is sieve pullback: pulling back a rank-`r` sieve along
`f: j → i` yields the rank-`min(r, j+1)` sieve on `j` (the largest
downward-closed subset that composes into the original).

### Lattice operations on `TruthValue`

Ω(i) is a chain Heyting algebra. Provide `meet` (min), `join` (max),
`top_at(i)` (rank i+1), `bottom` (rank 0), and `implies` (the Heyting
implication on a chain: `r → s = if r ≤ s then top else s`). This connects
16C to 16A's `HeytingAlgebra` — Ω(i) *is* a concrete Heyting algebra.

### `Terminal` presheaf

```rust
pub struct Terminal;
impl<C: SmallCategory> Presheaf<C> for Terminal {
    type At<Obj> = ();
    fn restrict<Dom, Cod>(_f, _x: ()) -> () { () }
}
```

The `truth` map `1 → Ω` sends each object `i` to `top_at(i)` (the maximal
sieve). Exposed as `truth_at(i) -> TruthValue`.

### Finite limits (pointwise)

Limits in a presheaf topos are computed **object-by-object**. Because presheaf
morphisms (natural transformations) cannot be first-class values in Rust (the
rank-N wall from 16B), we expose finite-limit *fibers* as functions that take
the presheaf values and morphism actions at a single object:

```rust
/// Pullback fiber at one object: {(p, q) | f(p) == g(q)}.
pub fn pullback_fiber<P, Q, R, F, G>(ps: &[P], qs: &[Q], f: F, g: G)
    -> Vec<(P, Q)> where ...;

/// Equalizer fiber at one object: {p | f(p) == g(p)}.
pub fn equalizer_fiber<P, R, F, G>(ps: &[P], f: F, g: G) -> Vec<P> where ...;
```

The caller enumerates objects (0..=N for `ChainCat`) and calls these per
object. This is honest: it makes the pointwise nature of presheaf-topos
limits explicit rather than hiding it behind a leaky abstraction.

### Characteristic morphism

```rust
/// For a subobject S ↪ P, compute χ(p) ∈ Ω(i): the support sieve of p.
/// χ(p) = largest sieve R such that p restricted along any m ∈ R stays in S.
pub fn characteristic_at<P, C, F>(... predicate: F, ...) -> TruthValue;
```

The defining property — **a subobject is the pullback of `truth` along χ** —
is the headline test: `p ∈ S  iff  χ(p) == truth` is *almost* the statement;
more precisely, `p ∈ S(i)` iff `χ(p)` is the maximal sieve on `i`.

## Deliverables

- `karpal-topos/src/classifier.rs` — `Omega`, `Terminal`, `TruthValue`, lattice ops, `truth_at`
- `karpal-topos/src/limits.rs` — `pullback_fiber`, `equalizer_fiber`, `characteristic_at`
- Tests: Ω restriction laws, lattice Heyting laws, the "subobject = pullback of truth" theorem, pullback/equalizer on concrete presheaves
- ROADMAP update (mark 16C complete)

## Connections

- **16A**: `TruthValue` lattice operations realize `HeytingAlgebra` on a chain.
- **16B**: Ω is built from 16B's sieves; restriction is sieve pullback.
- **Structured emptiness**: Ω(i) is the lattice of "how covered" object i is — rank 0 = no coverage (empty), rank i+1 = fully covered (top). The *reason* a position is uncovered carries the sieve structure.

## Out of scope (16D)

- `GrothendieckTopology` (a collection of sieves satisfying axioms)
- `Sheaf` (a presheaf satisfying the gluing condition for a topology)
- Sheafification adjunction
