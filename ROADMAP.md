# Karpal Roadmap

Karpal aims to be the definitive algebraic structures and category theory
library for Rust — a comprehensive "Rust Fantasyland" covering everything
from basic typeclasses through profunctor optics, recursion schemes, and
formal verification.

## Design Principles

- **GAT-based HKT encoding**: `trait HKT { type Of<T>; }` — clean, zero-dependency,
  leverages nightly Rust features that stable-only libraries cannot use
- **Static Land over Fantasy Land**: traits with associated functions (not methods
  on values) map naturally to Rust's static dispatch
- **Law verification built in**: every trait ships with proptest-based law tests;
  user-defined instances get the same verification for free
- **`no_std` first**: core and profunctor crates work without an allocator;
  `std`/`alloc` features gate heap-dependent instances
- **Newtype markers for multiple instances**: `Sum<T>`, `Product<T>`, `Min<T>`,
  `Max<T>` following Haskell convention — no orphan rule conflicts
- **Composition over completeness**: each phase delivers a usable, self-contained
  layer before the next begins

## Current State

### Phase 1 — Algebraic Structures Scaffold (complete)

| Crate | Contents |
|-------|----------|
| `karpal-core` | HKT, HKT2, Functor, Semigroup, Monoid |
| `karpal-profunctor` | Profunctor, Strong, Choice, FnP |
| `karpal-optics` | Optic marker, Lens (getter/setter + profunctor transform) |
| `karpal-std` | Stub for prelude re-exports |

---

## Near-term Phases

### Phase 2 — Complete the Functor Hierarchy

**Crate**: `karpal-core`

The Functor → Applicative → Monad chain is the backbone everything
else depends on.

| Trait | Depends On | Key Operations |
|-------|-----------|----------------|
| Apply | Functor | `ap(tf, ta) -> T<B>` |
| Applicative | Apply | `pure(a) -> T<A>` |
| Chain (FlatMap) | Apply | `chain(f, t) -> T<B>` |
| Monad | Applicative + Chain | *(combined)* |
| Alt | Functor | `alt(a, b) -> T<A>` |
| Plus | Alt | `zero() -> T<A>` |
| Alternative | Applicative + Plus | *(combined)* |
| Contravariant | — | `contramap(f, t) -> T<A>` |
| Bifunctor | Functor | `bimap(f, g, t) -> T<B, D>` |
| Selective | Applicative | `select(f_either, f_fn) -> T<B>` |
| Foldable | — | `fold_map(f, t) -> M` |
| Traversable | Functor + Foldable | `traverse(f, t) -> G<T<B>>` |
| FunctorFilter | Functor | `filter_map(f, t) -> T<B>` |
| Natural Transformation | — | `type FunctionK<F, G>` — `forall A. F<A> -> G<A>` |

Laws: identity, composition, homomorphism, interchange (Applicative);
left/right identity, associativity (Monad); naturality, identity,
composition (Traversable).

### Phase 3 — Comonads & Dual Structures

**Crate**: `karpal-core`

| Trait | Depends On | Key Operations |
|-------|-----------|----------------|
| Extend (CoflatMap) | Functor | `extend(f, w) -> T<B>`, `duplicate(w)` |
| Comonad | Extend | `extract(w) -> A` |
| ComonadEnv | Comonad | `ask`, `local` |
| ComonadStore | Comonad | `pos`, `peek` |
| ComonadTraced | Comonad | `trace` |
| Invariant | — | `invmap(f, g, t)` |
| Divide | Contravariant | `divide(f, a, b)` |
| Divisible | Divide | `conquer` |
| Decide | Contravariant | `choose(f, a, b)` |
| Conclude | Decide | `conclude(f)` |

### Phase 4 — Category / Arrow Hierarchy

**Crate**: `karpal-arrow` (new)

```
Category
  └→ Arrow
       ├→ ArrowZero → ArrowPlus
       ├→ ArrowChoice → ArrowApply (≅ Monad)
       └→ ArrowLoop
```

| Trait | Key Operations |
|-------|----------------|
| Semigroupoid | `compose(f, g)` |
| Category | Semigroupoid + `id()` |
| Arrow | `arr(f)`, `first`, `second`, `***`, `&&&` |
| ArrowChoice | `left`, `right`, `+++`, `\|\|\|` |
| ArrowApply | `app` |
| ArrowLoop | `loop_` |
| ArrowZero | `zero_arrow` |
| ArrowPlus | `plus(a, b)` |
| Kleisli | Arrow from Monad |
| Cokleisli | Arrow from Comonad |

### Phase 5 — Free Constructions & Kan Extensions

**Crate**: `karpal-free` (new)

| Construction | Description |
|-------------|-------------|
| Coyoneda | Free functor — makes any type constructor a Functor |
| Yoneda | `forall B. (A -> B) -> F<B>` — optimization via fusion |
| Free Monad | `Pure(A) \| Roll(F<Free<F, A>>)` — interpreter pattern |
| Freer Monad | No Functor constraint; uses Kan extension internally |
| Free Applicative | Static analysis of effects |
| Free Alternative | Free alternative functor |
| Cofree Comonad | `(A, F<Cofree<F, A>>)` — annotated trees, streams |
| Right Kan Extension | `Ran g h a = forall b. (a -> g b) -> h b` |
| Left Kan Extension | `Lan g h a = exists b. (g b -> a, h b)` |
| Codensity | `Ran m m` — performance optimization for free monads |
| Density | `Lan f f` — comonad from any functor |
| Day Convolution | Combines two functors; gives Applicative |

### Phase 6 — Recursion Schemes

**Crate**: `karpal-recursion` (new)

| Scheme | Type Pattern | Description |
|--------|-------------|-------------|
| F-Algebra | `F<A> -> A` | Evaluator for functor F |
| F-Coalgebra | `A -> F<A>` | Generator / unfolder |
| Fix / Mu | `F<Fix<F>> ≅ Fix<F>` | Least fixed point (recursive types) |
| Nu | Greatest fixed point | Corecursive / infinite structures |
| Catamorphism | `(F<A> -> A) -> Fix<F> -> A` | Fold (tear down bottom-up) |
| Anamorphism | `(A -> F<A>) -> A -> Fix<F>` | Unfold (build up top-down) |
| Hylomorphism | `ana ; cata` | Unfold then fold |
| Paramorphism | Fold with access to original subterms | |
| Apomorphism | Unfold with early termination | |
| Histomorphism | Fold with access to all previous results | |
| Futumorphism | Unfold with multiple steps | |
| Zygomorphism | Fold with auxiliary fold | |
| Chronomorphism | `futu ; histo` — most general scheme | |

### Phase 7 — Complete Profunctor Optics

**Crate**: `karpal-optics` (extend existing)

| Optic | Constraint | Focuses On |
|-------|-----------|------------|
| Lens | Strong | Single field (product) |
| Prism | Choice | Single variant (sum) |
| Iso | Profunctor | Isomorphism |
| Traversal | Traversing | Zero or more targets |
| Getter | Forget | Read-only access |
| Setter | Mapping | Write-only modification |
| Fold | Forget + Monoid | Read-only, multiple targets |
| Review | Tagged | Construct from focus |

Additional profunctor classes:

| Trait | Description |
|-------|-------------|
| Closed | Functions pass through |
| Mapping | Functors pass through |
| Traversing | Traversables pass through |
| Profunctor Functor | Functor in the profunctor category |
| Profunctor Monad | Monad in the profunctor category |
| Profunctor Comonad | Comonad in the profunctor category |

Optic composition: `Lens . Lens = Lens`, `Lens . Prism = Traversal`, etc.

### Phase 8 — Abstract Algebra

**Crate**: `karpal-algebra` (new)

| Structure | Description |
|-----------|-------------|
| Group | Monoid + `invert` |
| Abelian Group | Commutative group |
| Semiring | Add (commutative monoid) + Mul (monoid) + distribution |
| Ring | Semiring where addition forms a group |
| Field | Ring where multiplication forms a group |
| Lattice | `join` (supremum) + `meet` (infimum) |
| Bounded Lattice | Lattice + top + bottom |
| Module | Vector-scalar multiplication over a ring |
| Vector Space | Module over a field |

Newtype markers: `Sum<T>`, `Product<T>`, `Min<T>`, `Max<T>`, `First<T>`,
`Last<T>` for selecting between multiple valid instances.

---

## Mid-term Phases

### Phase 9 — Adjunctions & Advanced Category Theory

**Crate**: `karpal-core` (extend)

| Concept | Description |
|---------|-------------|
| Adjunction (F ⊣ U) | `unit: A -> U<F<A>>`, `counit: F<U<B>> -> B` |
| Triangle identities | `counit . F(unit) = id`, `U(counit) . unit = id` |
| Monad from adjunction | Every `F ⊣ U` gives monad `U . F` |
| Comonad from adjunction | Every `F ⊣ U` gives comonad `F . U` |
| Contravariant adjunction | Adjunction between contravariant functors |
| Profunctor adjunction | Adjunction in the profunctor category |
| Ends | `forall A. P<A, A>` for profunctor P |
| Coends | `exists A. P<A, A>` for profunctor P |
| Dinatural transformation | Transformation between profunctors |

### Phase 10 — Effect System & Monad Transformers

**Crate**: `karpal-effect` (new)

| Abstraction | Description |
|-------------|-------------|
| MonadTrans | `lift: M<A> -> T<M, A>` |
| ReaderT | Environment-passing transformer |
| WriterT | Log-accumulating transformer |
| StateT | Stateful computation transformer |
| ExceptT | Error-handling transformer |
| MonadReader | `ask`, `local`, `reader` |
| MonadWriter | `tell`, `listen`, `pass` |
| MonadState | `get`, `put`, `modify` |
| MonadError | `throw_error`, `catch_error` |

---

## Far-future Phases

### Phase 11 — `karpal-proof`: Algebraic Law Witnesses & Refinement Types

Type-level proof encoding — making illegal states unrepresentable and
providing a vocabulary for algebraic reasoning within Rust's type system.

| Concept | Description |
|---------|-------------|
| Property witnesses | `Proven<P: Property, T>(T)` — phantom-typed proofs that a value satisfies algebraic laws; constructing the type *is* the proof |
| Equational reasoning | `Rewrite<Lhs, Rhs, Via>` types for encoding algebraic identities verified by the compiler |
| Refinement newtypes | `NonEmpty<Vec<T>>`, `Associative<Op>`, `Commutative<Op>` composable with all Karpal traits |
| Auto-derive law checks | Proc macros generating proptest + Monte Carlo law verification for user-defined instances |

### Phase 12 — `karpal-verify`: External Prover Bridge

Formal verification via external tools, following the architecture
pioneered in [amari-flynn](https://github.com/Industrial-Algebra/amari-flynn).

| Capability | Description |
|-----------|-------------|
| SMT-LIB2 export | Generate proof obligations for algebraic laws as SMT-LIB2 (Z3, CVC5). Inspired by amari-flynn's `smt.rs` backend |
| Lean 4 bridge | Export Karpal typeclass hierarchies as Lean 4 structures; verify laws in Lean; import trust markers back as Rust phantom types |
| amari-flynn integration | Reuse contract macros (`#[prob_requires]`, `#[prob_ensures]`) on Karpal trait impls for statistical guarantees on law compliance |
| Three-tier verification | Following amari-flynn's philosophy: **Impossible** (type-level — unrepresentable states), **Rare** (statistical — Monte Carlo + Hoeffding bounds), **Emergent** (runtime — property-test discovery) |

### Phase 13 — `karpal-diagram`: Monoidal Categories & String Diagrams

| Concept | Description |
|---------|-------------|
| Monoidal category traits | `Tensor`, `Braiding`, `Symmetry` with coherence laws (pentagon, triangle, hexagon identities) |
| String diagram DSL | Compose morphisms and render corresponding string diagrams (SVG/text) for debugging optic compositions and arrow pipelines |
| Diagrammatic rewriting | Encode diagram equivalences as type-level rewrite rules; two compositions producing the same diagram type are proven equivalent |
| Compact closed categories | Trace / duality structures for quantum-inspired computation patterns |

### Phase 14 — `karpal-higher`: 2-Categories & Enriched Categories

| Concept | Description |
|---------|-------------|
| 2-category encoding | Objects = types, 1-morphisms = functions/traits, 2-morphisms = natural transformations. GAT encoding supports 2 levels deep. |
| Enriched categories | Categories where hom-sets carry algebraic structure (monoids, lattices, metric spaces) |
| Bicategories | Weakened 2-categories where composition is associative only up to isomorphism — relevant for profunctor composition |
| FFunctor / FMonad | Functor and Monad at the functor-category level; maps natural transformations |

---

## Workspace Evolution

```
karpal/
├── karpal-core/           # Phases 1-3, 9: HKT, functors, comonads, adjunctions
├── karpal-profunctor/     # Phase 1: Profunctor hierarchy
├── karpal-optics/         # Phases 1, 7: Profunctor optics
├── karpal-std/            # Prelude re-exports (grows with each phase)
├── karpal-arrow/          # Phase 4: Category/Arrow hierarchy
├── karpal-free/           # Phase 5: Free/Cofree, Kan extensions
├── karpal-recursion/      # Phase 6: Recursion schemes
├── karpal-algebra/        # Phase 8: Groups, rings, fields, lattices
├── karpal-effect/         # Phase 10: Monad transformers, effect system
├── karpal-proof/          # Phase 11: Type-level witnesses, refinements
├── karpal-verify/         # Phase 12: SMT/Lean bridge, amari-flynn integration
├── karpal-diagram/        # Phase 13: Monoidal categories, string diagrams
└── karpal-higher/         # Phase 14: 2-categories, enriched categories
```

## Rust-Specific Design Challenges

| Challenge | Strategy |
|-----------|----------|
| No native HKT | GAT encoding — accept some ergonomic cost, leverage nightly features |
| Orphan rules | Newtype wrappers + blanket impls where possible |
| Ownership in bind | Per-type semantics; `fmap` takes `impl Fn`, `bind` consumes `Self::Of<A>` |
| Multiple instances per type | Newtype markers (`Sum<T>`, `Product<T>`) following Haskell convention |
| No do-notation | Macro-based `do!` block, lean on `?` for Result/Option |
| Compile-time cost | Feature-gate advanced modules, keep core lean |
| Proofs beyond Rust's type system | Export to external provers (SMT, Lean 4), import trust via phantom types |

## References

- [Fantasy Land Specification](https://github.com/fantasyland/fantasy-land)
- [Static Land Specification](https://github.com/fantasyland/static-land)
- [Haskell Typeclassopedia](https://wiki.haskell.org/Typeclassopedia)
- [Scala Typeclassopedia](https://github.com/lemastero/scala_typeclassopedia)
- [Edward Kmett's Haskell ecosystem](https://github.com/ekmett) (profunctors, kan-extensions, recursion-schemes, free, adjunctions)
- [Fantastic Morphisms and Where to Find Them](https://yangzhixuan.github.io/pdf/fantastic-morphisms.pdf)
- [Category Theory for Programmers](https://bartoszmilewski.com/2014/10/28/category-theory-for-programmers-the-preface/) — Bartosz Milewski

## License

MIT OR Apache-2.0
