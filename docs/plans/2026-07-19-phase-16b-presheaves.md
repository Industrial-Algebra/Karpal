# Phase 16B — Presheaves & Sieves: Design

**Date:** 2026-07-19
**Sub-phase:** 16B (creates `karpal-topos`)
**Dependencies:** karpal-core 0.7.0 (HKT, PhantomData patterns), karpal-proof 0.7.0
**Status:** design, pre-implementation

## Goal

Create the `karpal-topos` crate with the first topos-theoretic building
blocks: small categories, presheaves (contravariant functors `C^op → Set`),
representable presheaves, sieves, and the Yoneda lemma as a computable
bijection. This is the foundation for 16C (subobject classifier) and 16D
(sheaves).

## The encoding problem

A presheaf `P: C^op → Set` consists of:

1. For each object `c` in a small category `C`, a set `P(c)`.
2. For each morphism `f: d → c` in `C`, a restriction map
   `P(f): P(c) → P(d)` — note the **contravariance** (direction reverses).
3. Functoriality: `P(id_c) = id`, `P(g ∘ f) = P(f) ∘ P(g)`.

The hard part is representing "the base category `C`" and "objects /
morphisms of `C`" at the type level, and handling contravariance without
re-igniting the type-system friction documented in issues #93, #95, #98
and the closure-barrier design doc.

## Chosen encoding

### Base category: morphism-as-data, not morphism-as-function

The existing `karpal-arrow::Category` trait is biased toward *computable*
morphisms (`compose`/`id` return morphism values that are function-like).
Presheaves are defined over *arbitrary small categories*, where morphisms
are often finite data (e.g. the simplex category Δ, poset categories),
not functions.

We therefore introduce a dedicated `SmallCategory` trait where morphisms
are **values** (data), decoupled from the arrow-computation hierarchy:

```rust
/// A small category encoded at the type level.
/// Objects are phantom types; morphisms A→B are values of `Mor<A, B>`.
pub trait SmallCategory {
    /// The type of morphisms from A to B.
    type Mor<A, B>;

    /// Identity morphism on A.
    fn identity<A>() -> Self::Mor<A, A>;

    /// Compose: g: B→C after f: A→B gives g∘f: A→C.
    fn compose<A, B, C>(g: Self::Mor<B, C>, f: Self::Mor<A, B>) -> Self::Mor<A, C>;
}
```

This is deliberately separate from `karpal-arrow::Category`. They serve
different purposes: arrow-`Category` is for endofunctor-style computable
composition; `SmallCategory` is the substrate for (pre)sheaf theory where
morphisms are indexing data. A bridge (treating an arrow-`Category` as a
`SmallCategory`) is possible later but out of scope for 16B.

### Presheaf

```rust
/// A presheaf on C: a contravariant functor C^op → Set.
///
/// Laws:
/// - Identity:     restrict(id, x) == x
/// - Composition:  restrict(g∘f, x) == restrict(f, restrict(g, x))
pub trait Presheaf<C: SmallCategory> {
    /// The set P(Obj): the value of the presheaf at object Obj.
    type At<Obj>;

    /// Restriction along a morphism f: Dom → Cod in C.
    /// Maps P(Cod) → P(Dom). Contravariant: the direction of f reverses.
    fn restrict<Dom, Cod>(
        cat: &C,
        f: C::Mor<Dom, Cod>,
        x: Self::At<Cod>,
    ) -> Self::At<Dom>;
}
```

Contravariance is handled structurally: `restrict` consumes
`Mor<Dom, Cod>` and produces `At<Dom>` from `At<Cod>`. No lifetime-GAT
machinery is needed for the first slice because presheaf values are owned
(see Limitations).

### Representable presheaf

The representable `Hom_C(-, c)` maps `d ↦ Mor(d, c)`; restriction along
`f: d → c` is precomposition:

```rust
pub struct Representable<Cod>(PhantomData<Cod>);

impl<C: SmallCategory, Cod> Presheaf<C> for Representable<Cod> {
    type At<Dom> = C::Mor<Dom, Cod>;
    fn restrict<Dom, Mid, Cod2>(cat, f: Mor<Dom, Mid>, m: Mor<Mid, Cod2>)
        -> Mor<Dom, Cod2>
    {
        C::compose(cat, m, f)  // precompose: m ∘ f
    }
}
```

### Sieve

A sieve on `c` is a **subfunctor** of `Hom(-, c)`: a collection of
morphisms into `c` closed under precomposition (if `f ∈ S` and `g` is
composable, then `f ∘ g ∈ S`). For a finite base category, a sieve is a
finite set of morphisms. For generality, it is a predicate on `Mor(d, c)`.

```rust
/// A sieve on object Cod: a precomposition-closed family of morphisms into Cod.
pub struct Sieve<C: SmallCategory, Cod> { /* membership predicate or finite set */ }

impl<C, Cod> Sieve<C, Cod> {
    /// Is f: Dom → Cod in the sieve?
    fn contains<Dom>(&self, cat: &C, f: &C::Mor<Dom, Cod>) -> bool;
    /// Closure: if f ∈ S then for all composable g, f∘g ∈ S.
    fn is_closed(&self, cat: &C) -> bool;
}
```

Two encodings ship: a finite `FiniteSieve` (morphisms stored in a `Vec`)
for testable small categories, and a predicate-based form for generality.

### Yoneda lemma (the testable centerpiece)

For any presheaf `P` and object `c`:

```
Nat(Hom(-, c), P)  ≅  P(c)
       α            ↦   α_c(id_c)          (evaluate at c, apply to id)
```

We make both directions computable:

- **`yoneda_from_element`**: given `x ∈ P(c)`, build the natural
  transformation `α^x` whose component at `d` sends `f: d → c` to
  `P(f)(x) ∈ P(d)`.
- **`yoneda_to_element`**: given a natural transformation `α`, return
  `α_c(id_c) ∈ P(c)`.

These are mutual inverses — tested directly on concrete presheaves over a
concrete small category.

Natural transformations between presheaves:

```rust
pub trait PresheafNat<P: Presheaf<C>, Q: Presheaf<C>, C: SmallCategory> {
    fn component<Obj>(cat: &C, p_at_obj: P::At<Obj>) -> Q::At<Obj>;
}
```

## Concrete instances (for testing)

To make all of the above testable, 16B ships two concrete small categories:

1. **`ChainCat`** — the poset category of a finite chain `0 ≤ 1 ≤ ... ≤ n`,
   where a morphism `i → j` exists iff `i ≤ j` (unique witness). This is the
   simplest non-trivial small category and exercises precomposition closure
   concretely.

2. **`DiscreteCat`** — only identity morphisms. Presheaves over it are
   trivial (any assignment); useful as a degenerate baseline.

And two concrete presheaves:

1. **Constant presheaf** — `P(c) = A` for a fixed set `A`, restriction is
   identity. Tests functoriality trivially.

2. **A "size" presheaf over `ChainCat`** — e.g. `P(i) = {0..=i}`,
   restriction `P(j) → P(i)` truncates. Tests non-trivial restriction.

## Test plan

Every trait ships with law tests (proptest where applicable):

| Entity | Laws tested |
|--------|-------------|
| `SmallCategory` | associativity + identity of compose |
| `Presheaf` | identity (`restrict(id, x) == x`), composition (`restrict(g∘f, x) == restrict(f, restrict(g,x))`) |
| `Representable` | presheaf laws via precomposition |
| `Sieve` | precomposition closure |
| `Yoneda` | `yoneda_to_element(yoneda_from_element(x)) == x` and inverse |

## Documented limitations (proactive, from the audit lessons)

1. **No full functor category `[C^op, Set]`.** Rust cannot enumerate all
   presheaves as a single type. We provide the `Presheaf<C>` trait and
   natural transformations between *specific* instances. The Yoneda
   *embedding* `Y: C → PSh(C)` is expressed as the construction
   `c ↦ Hom(-, c)` (i.e. `Representable<c>`), not as a value-level
   functor into an encoded functor-category type. This is honest: the
   embedding is a *construction recipe*, not a first-class functor object.

2. **`'static` on presheaf values.** `type At<Obj>` is used concretely
   with owned data (`Vec`, `u32`, etc.) in the first slice. Presheaves
   valued in borrowed sets would require the `HKTLt` lifetime-GAT
   machinery (cf. issue #93). Deferred.

3. **`SmallCategory` vs `karpal-arrow::Category`.** These are distinct
   traits serving distinct purposes. A bridge is possible but out of
   scope. Documenting this separation up front prevents the "silent
   incompatibility" pattern that bit issue #97.

4. **Yoneda here vs `karpal-free::Yoneda`.** `karpal-free::Yoneda` is
   the Yoneda *lemma as a fusion optimization for functors* (CPS-encoded
   `F<A>` with deferred maps). The Yoneda lemma in 16B is the *category-
   theoretic* bijection `Nat(Hom(-,c), P) ≅ P(c)` for presheaves. They
   are the same lemma in two roles; the connection is noted in docs but
   they are separate code because their type-level encodings differ.

## Deliverables

- `karpal-topos/` crate (workspace member, std/alloc gated like karpal-higher)
- `src/small_category.rs` — `SmallCategory`, `ChainCat`, `DiscreteCat`
- `src/presheaf.rs` — `Presheaf<C>`, constant + chain presheaves
- `src/representable.rs` — `Representable<Cod>`
- `src/sieve.rs` — `Sieve`, `FiniteSieve`
- `src/yoneda.rs` — `PresheafNat`, `yoneda_from_element`, `yoneda_to_element`
- `src/lib.rs` — module wiring + crate docs
- Law tests for every trait; Yoneda bijection round-trip tests
- README.md + ROADMAP update (mark 16B complete)

## Out of scope (16C / 16D)

- `SubobjectClassifier`, `Pullback`, `Equalizer` (16C)
- `GrothendieckTopology`, `Sheaf`, sheafification (16D)
- Bridge from `karpal-arrow::Category` to `SmallCategory`
- Lifetime-parameterized presheaves (`PresheafLt`)
