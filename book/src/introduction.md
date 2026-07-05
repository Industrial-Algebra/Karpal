# Introduction

**Karpal** is a Higher-Kinded Type (HKT) library for the Industrial Algebra ecosystem. It provides:

- HKT encoding via GATs (`trait HKT { type Of<T>; }`)
- A complete functor hierarchy (Functor → Applicative → Monad, plus Alt, Plus, Foldable, Traversable, and more)
- Algebraic typeclasses (Semigroup, Monoid, Group, Ring, Field, Lattice, Module, VectorSpace, HeytingAlgebra)
- Profunctor optics (Lens, Prism, Traversal, Fold, Iso, and more)
- Category/Arrow hierarchy with FnA, KleisliF, CokleisliF
- Free constructions (Free Monad, Cofree Comonad, Coyoneda, Day Convolution, Kan extensions)
- Recursion schemes (cata, ana, hylo, para, apo, histo, futu, zygo, chrono)
- Adjunctions and advanced category theory (ends, coends, dinatural transformations)
- Monad transformers (ExceptT, WriterT, ReaderT, StateT)
- Algebraic law witnesses and proof-carrying code
- External verification (SMT-LIB2, Lean 4, Kani, GPU obligations)
- String diagrams and monoidal category theory
- Schubert intersection type system
- 2-categories, enriched categories, and bicategories

All with `no_std` support and property-based law verification.

## Why Karpal?

Rust has `Option::map`, `Result::and_then`, and `Iterator::collect`. They work great — but they're ad-hoc. Every container re-invents the same patterns with slightly different names, and there's no way to write a function that's generic over "any container that supports mapping" or "any container that supports sequencing effects."

Karpal gives those patterns names and laws, so you can abstract over them.

### What does this buy you that standard Rust doesn't?

**Generic traversals.** `traverse` works over any `Traversable` + `Applicative` pair. You write `validate_batch` once and it works for `Vec`→`Result`, `HashMap`→`Option`, or any other combination:

```rust
// Works for any Traversable container and any Applicative effect
fn validate_all<C: Traversable, F: Applicative>(items: C::Of<Raw>) -> F::Of<C::Item>
```

**Composable lenses.** Instead of hand-writing nested struct accessors, you compose lenses like functions and pass them around as first-class values:

```rust
let street_lens = address_lens.compose(street_name_lens);
let updated = street_lens.over(company, |s| format!("{} (HQ)", s));
```

**Law-guaranteed abstractions.** Every `Monad` instance is property-tested for left identity, right identity, and associativity. If you implement `Monad` for your type and get it wrong, the test suite catches it — before your users do.

### Honest limitations

The GAT-based HKT encoding has real constraints:
- No higher-kinded type inference — you must spell out type constructor markers (`OptionF`, `VecF`)
- Cannot abstract over type constructors with different kind signatures
- Requires nightly Rust (edition 2024 features)
- Static Land style (`OptionF::fmap(...)`) rather than method chaining (`some.fmap(...)`)

These are inherent to encoding HKT in a language without native HKT support. Karpal chooses the GAT encoding because it's zero-dependency, stable since Rust 1.65, and doesn't require proc-macro magic.

## License

Apache-2.0 + CLA. See [CONTRIBUTING.md](https://github.com/Industrial-Algebra/Karpal/blob/develop/CONTRIBUTING.md) for details.
