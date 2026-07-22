# Topos Theory

The `karpal-topos` crate realizes the categorical infrastructure underlying [structured emptiness](../concepts/structured-emptiness.md): small categories, presheaves, sieves, the subobject classifier Ω, finite limits, Grothendieck topologies, sheaves, and the Yoneda lemma.

This is the Phase 16 stack — the most abstract layer of Karpal, where "zero has geometry" becomes formal: the reason for emptiness matters as much as the emptiness itself, and topos theory provides the language (Ω is the sieve lattice, sheafification is local-to-global gluing).

## Overview

| Module           | Contents                                                         | Feature gate         |
|------------------|------------------------------------------------------------------|----------------------|
| `small_category` | `SmallCategory`, `ChainCat<N>` (finite poset), `DiscreteCat`     | `no_std`             |
| `presheaf`       | `Presheaf<C>`, `ConstantPresheaf`, `InitialSegmentPresheaf`      | core; presheaf values `alloc` |
| `representable`  | `Representable<c>` — the hom-presheaf `Hom(-, c)`                | `no_std`             |
| `sieve`          | `Sieve`, `FiniteSieve` (precomposition-closed families)          | `alloc`              |
| `classifier`     | `Omega` (subobject classifier), `Terminal`, `TruthValue` lattice | `no_std`             |
| `limits`         | `pullback_fiber`, `equalizer_fiber`, `characteristic_at`        | `alloc`              |
| `topology`       | `GrothendieckTopology`, `LawvereTierneyTopology`                 | `no_std`             |
| `sheaf`          | `is_separated_at`, `is_sheaf_at`, sheafification interface       | `alloc`              |
| `yoneda`         | `yoneda_apply`, `yoneda_extract` — the Yoneda bijection          | `no_std`             |

The crate builds in three configurations: `std`, `no_std + alloc`, and pure `no_std` (the `sieve`, `limits`, and `sheaf` modules are `alloc`-gated).

## Small Categories

### SmallCategory

A small category where **objects are phantom marker types** and **morphisms are values carrying runtime data**.

``` rust
pub trait SmallCategory {
    /// The type of morphisms from A to B.
    type Mor<A, B>;

    /// Compose g: B → C after f: A → B, yielding g ∘ f: A → C.
    fn compose<A, B, C>(g: Self::Mor<B, C>, f: Self::Mor<A, B>) -> Self::Mor<A, C>;
}
```

**Law:** associativity — `compose(h, compose(g, f)) == compose(compose(h, g), f)`.

#### Why not `karpal_arrow::Category`?

`karpal_arrow::Category` is biased toward *computable* morphisms (`compose`/`id` return function-like values). Presheaves are defined over arbitrary small categories where morphisms are often finite data (the simplex category Δ, poset categories). This `SmallCategory` is deliberately separate: morphisms are indexing data.

#### Identity is per-concrete-category

Rust cannot extract object identity from phantom type parameters, so `SmallCategory` provides only `compose`. Each concrete category supplies `identity` as an inherent method bound to an object-index trait. This is an honest limitation, not an omission.

### ChainCat\<N\>

The poset category of a finite chain `0 ≤ 1 ≤ … ≤ N`. A morphism `i → j` exists iff `i ≤ j` (unique witness). This is the simplest non-trivial small category.

``` rust
use karpal_topos::{ChainCat, ChainMor, ChainObj, SmallCategory};

// Object markers, each exposing its position at compile time.
struct C0; struct C1; struct C2;
impl ChainObj for C0 { const IDX: usize = 0; }
impl ChainObj for C1 { const IDX: usize = 1; }
impl ChainObj for C2 { const IDX: usize = 2; }

// Identity is an inherent method:
let id: ChainMor<C1, C1> = ChainCat::<2>::identity::<C1>();

// A morphism exists only when the source ≤ target:
let f: ChainMor<C0, C2> = ChainCat::<2>::morphism::<C0, C2>().unwrap();
assert!(ChainCat::<2>::morphism::<C2, C0>().is_none()); // 2 > 0, no morphism

// Composition:
let g: ChainMor<C1, C2> = ChainCat::<2>::morphism::<C1, C2>().unwrap();
let gf: ChainMor<C0, C2> = ChainCat::<2>::compose(g, f);
assert_eq!((gf.from(), gf.to()), (0, 2));
```

`DiscreteCat` is the degenerate case: only identity morphisms exist.

## Presheaves

### Presheaf\<C\>

A contravariant functor `C^op → Set`. For each object it assigns a set; for each morphism `f: Dom → Cod` it assigns a restriction map `restrict(f): P(Cod) → P(Dom)`.

``` rust
pub trait Presheaf<C: SmallCategory> {
    /// The set P(Obj): the value of the presheaf at object Obj.
    type At<Obj>;

    /// Restriction along f: Dom → Cod. Maps P(Cod) → P(Dom).
    fn restrict<Dom, Cod>(f: C::Mor<Dom, Cod>, x: Self::At<Cod>) -> Self::At<Dom>;
}
```

**Laws:**
- Identity: `restrict(id, x) == x`
- Composition: `restrict(g ∘ f, x) == restrict(f, restrict(g, x))`

Note the **contravariance**: restriction along `f: Dom → Cod` maps values at `Cod` to values at `Dom`, and composition order reverses.

#### Instances

| Presheaf                 | `P(i)`                       | Restriction                                   |
|--------------------------|------------------------------|-----------------------------------------------|
| `ConstantPresheaf<T>`    | `T` (same everywhere)        | identity (returns `x` unchanged)              |
| `InitialSegmentPresheaf` | `{0, 1, …, i}` (`SegmentSet`)| truncates to the first `Dom::IDX + 1` elements|
| `Representable<c>`       | `Hom(i, c)` (morphisms)      | precomposition: `m ↦ m ∘ f`                   |
| `Omega`                  | `TruthValue` (sieve rank)    | `min(rank, Dom::IDX + 1)`                     |
| `Terminal`               | `()`                         | identity                                      |

### Representable\<c\>

The hom-presheaf `Hom_C(-, c)`. For each object `d`, `At<d> = Hom_C(d, c)`. Restriction along `f: Dom → Cod` is precomposition: `Hom(Cod, c) → Hom(Dom, c)` sends `m` to `m ∘ f`. This is the anchor of the Yoneda lemma.

## Sieves

A **sieve** on an object `c` is a precomposition-closed family of morphisms into `c`: whenever `f: d → c` is in the sieve and `g: e → d` is any morphism, the composite `f ∘ g` is also in the sieve. Sieves are the "covering" concept underlying Grothendieck topologies.

``` rust
use karpal_topos::{FiniteSieve, Sieve, ChainCat, ChainObj};
# struct C0; struct C2; struct C3;
# impl ChainObj for C0 { const IDX: usize = 0; }
# impl ChainObj for C2 { const IDX: usize = 2; }
# impl ChainObj for C3 { const IDX: usize = 3; }

// {2} alone is NOT closed: precomposition with 0→2, 1→2 requires 0 and 1.
let unclosed: FiniteSieve<C3> = FiniteSieve::new([2]);
assert!(!Sieve::<ChainCat<3>, C3>::is_closed(&unclosed));

// close() enforces downward closure: {2} becomes {0, 1, 2}.
let closed = unclosed.close();
assert!(Sieve::<ChainCat<3>, C3>::is_closed(&closed));

// The maximal sieve contains all sources [0, Cod::IDX].
let max: FiniteSieve<C3> = FiniteSieve::maximal();
```

## The Subobject Classifier Ω

In a presheaf topos `[C^op, Set]`, the subobject classifier Ω is the presheaf assigning to each object `c` the set of **sieves on `c`**. Over `ChainCat<N>`, sieves are downward-closed subsets representable by a **rank** — a chain Heyting algebra.

### TruthValue

``` rust
pub struct TruthValue { pub rank: usize }
```

For object `i`, `Ω(i)` contains ranks `0..=i+1`:
- rank `0` = the empty sieve (**bottom** — "nothing is covered")
- rank `k` = the sieve `{0, …, k-1}`
- rank `i+1` = the maximal sieve (**top** — "everything is covered")

This forms a **Heyting algebra** (intuitionistic logic), the foundation of structured emptiness:

``` rust
use karpal_topos::TruthValue;

let a = TruthValue { rank: 2 };
let b = TruthValue { rank: 4 };

a.meet(b);                          // lattice meet (sieve intersection)
a.join(b);                          // lattice join (sieve union)
a.implies_at(b, 4);                 // Heyting implication at object 4
a.neg_at(3);                        // Heyting negation: ¬a = a → bottom
```

Note: `¬¬a ≠ a` in general — this is intuitionistic, not classical, logic. The missing middle is itself a kind of structured emptiness.

### Terminal and the truth map

`Terminal` is the terminal presheaf (sends every object to `()`). The truth map `true: 1 → Ω` selects the maximal sieve:

``` rust
use karpal_topos::truth_at;
let max_sieve_on_2 = truth_at(2); // TruthValue { rank: 3 }
```

A subobject `S ↪ A` corresponds to the unique characteristic morphism `χ: A → Ω` whose pullback along `true` recovers `S`.

## Finite Limits

Limits in a presheaf topos are computed **pointwise**. Because natural transformations cannot be first-class values in Rust (the rank-N wall), these are exposed as **fiber functions** that take presheaf values and morphism actions at a single object:

``` rust
use karpal_topos::{pullback_fiber, equalizer_fiber, characteristic_at};

// Pullback fiber at one object: pairs (p, q) with f(p) == g(q).
let pb = pullback_fiber(&[1,2,3], &[10,20,30], |p| p % 2, |q| (q/10) % 2);

// Equalizer fiber: elements p with f(p) == g(p).
let eq = equalizer_fiber(&[1,2,3,4], |p| *p, |p| p + (p % 2));

// Characteristic morphism χ at object i: the largest sieve rank such that
// p restricted into the subobject stays in S.
let chi = characteristic_at(2, &42, |_p, j| j < 2); // rank 2
```

The defining theorem: **`p ∈ S(i)` iff `χ(p)` is the maximal sieve on `i`** — a subobject is the pullback of `truth` along χ.

## Grothendieck Topologies

A **Grothendieck topology** `J` assigns to each object a collection of covering sieves.

``` rust
pub trait GrothendieckTopology {
    fn is_covering(i: usize, rank: usize) -> bool;
}
```

**Laws** (verified by the axiom checkers):
1. **Maximality** — the maximal sieve (rank `i+1`) always covers.
2. **Stability** — if rank `r` covers `i`, then `min(r, j+1)` covers `j`.
3. **Transitivity** — sieves that are "locally covering" are covering.

| Topology           | What covers                                          |
|--------------------|------------------------------------------------------|
| `TrivialTopology`  | only the maximal sieve (rank `i+1`)                 |
| `DenseTopology`    | any non-empty sieve (rank ≥ 1)                       |

### Lawvere-Tierney topologies

The equivalent notion as a closure operator `j: Ω → Ω` on truth values:

``` rust
pub trait LawvereTierneyTopology {
    fn j(i: usize, rank: usize) -> usize;
}
```

**Laws:** `j(top) = top`, `j(j(r)) = j(r)` (idempotence), `j(min(r,s)) = min(j(r), j(s))` (meet-preserving). There is a bijection between Grothendieck and Lawvere-Tierney topologies; `TrivialTopology` and `DenseTopology` implement both.

## Sheaves

A presheaf `P` is a **sheaf** for a topology `J` if, for every covering sieve, every compatible family of local sections glues uniquely to a global section.

``` rust
use karpal_topos::{is_separated_at, is_sheaf_at};

// Separated (unique gluing): distinct elements have distinct restriction profiles.
let separated = is_separated_at(2, 3, &[1, 2, 3], |x, _k| *x);

// Full sheaf condition: every compatible family glues uniquely.
let is_sheaf = is_sheaf_at(
    2, 1, &[7, 8],
    |_k| vec![7, 8],
    |x, _k| *x,
);
```

### Sheafification

Sheafification `a: PSh(C) → Sh(C, J)` is the left adjoint to inclusion — it sends a presheaf to its "best sheaf approximation." The full plus-construction is genuinely complex and is **not** implemented; the interface documents the adjunction shape (unit/counit/triangle identities) and its connection to `karpal_core::Adjunction`. This is honest about the boundary, not a stub masquerading as complete.

## The Yoneda Lemma

For any presheaf `P` and object `c`:

``` text
Nat(Hom(-, c), P)  ≅  P(c)
```

Rust cannot represent a natural transformation as a first-class value (it is rank-N polymorphic over the object index — the same wall as `FreeAp::fold_map`). So the bijection is exposed by its **computable action**:

``` rust
use karpal_topos::{yoneda_apply, yoneda_extract};

// Forward: x ∈ P(c) induces a natural transformation.
// Given f: Dom → Cod, yoneda_apply computes restrict(f, x) ∈ P(Dom).
let applied = yoneda_apply::<P, C, Dom, Cod>(f, x);

// Inverse: evaluate the transformation at c on the identity morphism.
let x = yoneda_extract::<P, C, Cod, _>(id_c, |f| action(f));
```

- `yoneda_apply(f, x)` = `restrict(f, x)` — the component of the induced transformation.
- `yoneda_extract(id_c, action)` = `action(id_c)` — recovering the generating element.

The round-trip identity is directly testable: extract after apply recovers `x`, because `restrict(id, x) == x` by the presheaf identity law.

---

Karpal is licensed under Apache-2.0 + CLA. [View on GitHub](https://github.com/Industrial-Algebra/Karpal).
