# karpal-topos

Topos-theoretic constructions for the Industrial Algebra ecosystem.

`karpal-topos` provides the categorical infrastructure underlying structured
emptiness: small categories, presheaves (contravariant functors `C^op → Set`),
representable presheaves, sieves, and the Yoneda lemma.

## Status

**Phase 16 (complete):** the full topos stack — Heyting algebras (16A),
presheaves/sieves/Yoneda (16B), subobject classifier & finite limits (16C),
Grothendieck topologies & sheaves (16D).

## What's here

| Module | Contents |
|--------|----------|
| `small_category` | `SmallCategory` trait, `ChainCat` (finite poset), `DiscreteCat` |
| `presheaf` | `Presheaf<C>` trait, `ConstantPresheaf`, `InitialSegmentPresheaf` |
| `representable` | `Representable<c>` — the hom-presheaf `Hom(-, c)` |
| `sieve` | `Sieve` trait, `FiniteSieve` (precomposition-closed families) |
| `classifier` | `Omega` subobject classifier, `Terminal`, `TruthValue` Heyting lattice |
| `limits` | `pullback_fiber`, `equalizer_fiber`, `characteristic_at` |
| `topology` | `GrothendieckTopology` (trivial, dense), `LawvereTierneyTopology` |
| `sheaf` | `is_separated_at`, `is_sheaf_at`, sheafification adjunction interface |
| `yoneda` | `yoneda_apply`, `yoneda_extract` — the Yoneda bijection in action form |

## Encoding

Objects of a small category are phantom marker types; morphisms `Mor<A, B>`
are values carrying runtime data. Identity is constructed per-concrete-category
(Rust cannot extract object identity generically from phantom types). See
[`docs/plans/2026-07-19-phase-16b-presheaves.md`](../docs/plans/2026-07-19-phase-16b-presheaves.md)
for the full design rationale.

## Example

```rust
use karpal_topos::{Presheaf, Representable, SmallCategory};
use karpal_topos::small_category::{ChainCat, ChainObj, ChainMor};

// Object markers for the chain 0 ≤ 1 ≤ 2.
struct C0; struct C1; struct C2;
impl ChainObj for C0 { const IDX: usize = 0; }
impl ChainObj for C1 { const IDX: usize = 1; }
impl ChainObj for C2 { const IDX: usize = 2; }

// The representable presheaf Hom(-, C2).
// At<C1> = Hom(C1, C2) = ChainMor<C1, C2>.
let m: ChainMor<C1, C2> = ChainCat::<2>::morphism::<C1, C2>().unwrap();
let f: ChainMor<C0, C1> = ChainCat::<2>::morphism::<C0, C1>().unwrap();

// Restriction along f precomposes: m ∘ f : C0 → C2.
let composed: ChainMor<C0, C2> =
    <Representable<C2> as Presheaf<ChainCat<2>>>::restrict(f, m);
assert_eq!((composed.from(), composed.to()), (0, 2));
```

## License

Apache-2.0 (with CLA; see `CONTRIBUTING.md`).
