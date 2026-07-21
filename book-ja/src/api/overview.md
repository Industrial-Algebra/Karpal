# Workspace Overview

Karpal consists of 18 crates organized by domain:

| Crate | Purpose |
|-------|---------|
| `karpal-core` | HKT encoding, Functor→Monad, Comonads, Adjunctions, ends/coends |
| `karpal-profunctor` | Profunctor, Strong, Choice, FnP |
| `karpal-optics` | Iso, Lens, Prism, Traversal, Fold, Getter, Setter, Review |
| `karpal-arrow` | Category/Arrow hierarchy, FnA, KleisliF, CokleisliF |
| `karpal-free` | Coyoneda, Free, Cofree, Freer, Day, Kan extensions |
| `karpal-recursion` | Fix, cata, ana, hylo, para, apo, histo, futu, zygo, chrono |
| `karpal-algebra` | Group, Semiring, Ring, Field, Lattice, HeytingAlgebra, Module, VectorSpace |
| `karpal-effect` | ExceptT, WriterT, ReaderT, StateT, MonadTrans |
| `karpal-proof` | Proven<P,T>, Rewrite, refinement types |
| `karpal-proof-derive` | #[derive(VerifySemigroup)] etc. |
| `karpal-verify` | Obligation IR, SMT/Lean 4/Kani, GPU obligations, trust boundary |
| `karpal-verify-derive` | #[export_obligations] macro |
| `karpal-diagram` | Monoidal categories, string diagrams, coherence witnesses |
| `karpal-schubert-types` | Schubert intersection types, SchubertProven, LR enrichment |
| `karpal-higher` | 2-categories, enriched categories, bicategories, FFunctor, FMonad |
| `karpal-index` | AI-agent library discovery CLI |
| `karpal-std` | Prelude re-exports |

See the [HTML reference docs](https://karpal.industrial-algebra.com) for detailed API documentation.
