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

## License

Apache-2.0 + CLA. See [CONTRIBUTING.md](https://github.com/Industrial-Algebra/Karpal/blob/develop/CONTRIBUTING.md) for details.
