# Karpal Roadmap

Karpal aims to be the definitive algebraic structures and category theory
library for Rust — a comprehensive "Rust Static Land" covering everything
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
- **Structured emptiness**: zeros carry provenance — *why* something is empty
  matters as much as *that* it is empty. See [Structured Emptiness](#structured-emptiness-zero-intersection-semantics)
  below.

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

### Phase 15 — `karpal-schubert-types`: Schubert Intersection Type System

**Crate**: `karpal-schubert-types` (new, experimental)

**Origin**: Discovered via ShaperOS sasm dual-domain execution model — cross-domain
type checking between rewrite rules and register VM requires verifying that
declarative patterns satisfy imperative signatures. This maps directly to
Schubert intersection in a Grassmannian.

**Core idea**: types are Schubert classes σ_λ in Gr(k, n), and type checking
is computing intersection numbers via Littlewood-Richardson coefficients.
Subtyping becomes `σ_A · σ_B ≠ 0`, with the LR coefficient giving the
*multiplicity* — the number of distinct coercion paths.

| Concept | Description |
|---------|-------------|
| `SchubertType` | A class (or union of classes) in Gr(k, n) |
| `Intersection` | Result with multiplicity, per-class decomposition, computation path, structured classification |
| `IntersectionKind` | Structural zero / geometric zero / positive / underdetermined — the concrete realization of [structured emptiness](#structured-emptiness-zero-intersection-semantics) |
| `SchubertTyped` trait | Associate a Schubert class with a Rust type |
| `SchubertProven<λ, T>` | Proof-carrying type assertion verified by intersection computation |
| Operadic composition | `compose_checks()` — chained type checks compose via the LR rule |

**Sub-phases**:

| Sub-phase | Description | Dependencies |
|-----------|-------------|--------------|
| **A — Core engine** | `SchubertType`, `Intersection`, `check_intersection()` backed by amari-enumerative | karpal-core, karpal-algebra, amari-enumerative |
| **B — Proof integration** | `SchubertProven<λ, T>` witness type, composition of proofs via LR | Phase 11 (karpal-proof) |
| **C — External verification** | SMT-LIB2 export of intersection queries, Lean 4 export of LR rule | Phase 12 (karpal-verify) |
| **D — Enriched formalization** | Schubert intersection as category enriched over LR coefficient ring | Phase 14 (karpal-higher) |

**Benefits beyond ShaperOS**:
- **Multiplicity-aware compatibility**: formalizes what newtypes (`Sum<T>`, `Product<T>`) do informally — multiplicity > 1 means multiple valid instances
- **Capability-based access control**: capabilities as Schubert classes, nonzero intersection = access granted
- **Interface compatibility scoring**: quantitative API evolution analysis (0 = breaking, 1 = seamless, >1 = ambiguous)
- **Enriched error messages**: "Schubert intersection = 0 in Gr(3,6)" vs "3 coercion paths available, please disambiguate"

See [docs/dev/schubert-intersection-types.md](docs/dev/schubert-intersection-types.md)
for the full synopsis.

**Extensions** (building on existing and planned phases):

| Extension | Description | Dependencies |
|-----------|-------------|--------------|
| **E — Topos-theoretic grounding** | Schubert intersection as pullback in a presheaf topos; IntersectionKind as Heyting-valued subobject classifier; enables internal logic over Schubert classes | Phase 16 (karpal-topos) |
| **F — K-theoretic refinement** | Replace cohomology classes with K-theory classes; Grothendieck ring structure on type lattice; quantum deformation parameter for refined multiplicity | karpal-algebra (Ring), Phase 14 |
| **G — Equivariant Schubert calculus** | Torus-equivariant intersection theory; localization formulas (Atiyah-Bott) for efficient computation; polynomial representatives (Schubert polynomials) via karpal-algebra | karpal-algebra (Ring, Module), amari-enumerative |
| **H — Motivic measures** | Motivic integration over Schubert varieties; connects structured emptiness to measure-theoretic "weight of emptiness"; virtual motives as universal additive invariant | Phase 16 (karpal-topos), karpal-algebra |

Extensions E and F are the most immediately meaningful:
- **E** gives Phase 15 a rigorous categorical home — Schubert intersection *is*
  pullback in the right topos, and `IntersectionKind` *is* the subobject
  classifier of that topos. This collapses the gap between "structured emptiness
  as a design pattern" and "structured emptiness as categorical truth".
- **F** refines the coarse intersection number into a polynomial invariant. Where
  Phase 15A gives `σ_λ · σ_μ = 2`, K-theoretic refinement gives
  `[O_λ] · [O_μ] = q + q²` — the *two* solutions are distinguished by a
  deformation parameter, which maps to distinct coercion paths at the type level.

### Phase 16 — `karpal-topos`: Topos-Theoretic Constructions

**Crate**: `karpal-topos` (new)

Topos theory unifies logic, geometry, and category theory. A topos is a
category that "behaves like Set" — it has all finite limits, exponentials,
and a subobject classifier Ω. This phase provides the categorical
infrastructure that Phase 15 and structured emptiness ultimately rest on.

| Concept | Description |
|---------|-------------|
| `SubobjectClassifier` trait | Ω with `true: 1 → Ω` and characteristic morphism `χ: Sub(A) ↔ Hom(A, Ω)` |
| `HeytingAlgebra` | Lattice + implication; internal logic of any topos. Extends `BoundedLattice` from Phase 8 |
| `Presheaf<C>` | Functor `C^op → Set`; the free cocompletion. Built on karpal-core's `Functor` + `NaturalTransformation` |
| `Sieve<C>` | Subfunctor of a representable presheaf; the "covering" concept underlying Grothendieck topologies |
| `GrothendieckTopology` | Coverage on a category; axiomatizes which sieves count as "covers" |
| `Sheaf<C, J>` | Presheaf satisfying the gluing condition for topology J; sheafification functor |
| `Topos` trait | Category with finite limits, exponentials, and subobject classifier |
| `InternalHom` | Exponential objects A^B in a topos; generalizes function types |
| `PowerObject` | P(A) = Ω^A; internal powerset; the "type of subtypes of A" |
| `Pullback` / `Equalizer` | Finite limit constructions; Schubert intersection *is* pullback in the flag variety topos |

**Sub-phases**:

| Sub-phase | Description | Dependencies |
|-----------|-------------|--------------|
| **A — Heyting algebras & internal logic** | `HeytingAlgebra` extending `BoundedLattice`, internal implication `→`, negation `¬`, propositional connectives | karpal-algebra (BoundedLattice) |
| **B — Presheaves & sieves** | `Presheaf<C>`, `Sieve<C>`, Yoneda embedding (connecting to karpal-free's Yoneda), representable presheaves | karpal-core (Functor, NaturalTransformation), karpal-free (Yoneda) |
| **C — Subobject classifier & finite limits** | `SubobjectClassifier`, `Pullback`, `Equalizer`, characteristic morphism construction | Sub-phase A, Sub-phase B |
| **D — Grothendieck topologies & sheaves** | `GrothendieckTopology`, `Sheaf`, sheafification adjunction (connecting to karpal-core adjunctions), Lawvere-Tierney topologies | Sub-phase C, karpal-core (Adjunction) |

**Connections to existing phases**:
- **Phase 8 (Abstract Algebra)**: `BoundedLattice` → `HeytingAlgebra` is a direct extension; the structured emptiness lattice `Ω = { Denied, Granted(0), Granted(n), Granted(∞) }` becomes a concrete subobject classifier
- **Phase 9 (Adjunctions)**: Sheafification is a left adjoint to the inclusion of sheaves into presheaves; this is a new `Adjunction` instance with deep computational content
- **Phase 5 (Free Constructions)**: Presheaves *are* the free cocompletion; the Yoneda lemma (karpal-free's `Yoneda<F, A>`) is the embedding theorem for presheaf topoi
- **Phase 14 (Enriched Categories)**: Enriched topoi generalize to categories enriched over the subobject classifier — connecting directly to Phase 15's LR-enriched categories
- **Phase 15 (Schubert Types)**: `IntersectionKind` is literally a subobject classifier; Schubert intersection is pullback; structured emptiness is the internal logic of a non-Boolean topos

---

## Structured Emptiness: Zero-Intersection Semantics

A cross-cutting design insight discovered during
[ShaperOS](https://github.com/Industrial-Algebra/ShaperOS) development.

### The Problem

Standard algebraic libraries treat zero as a single concept — `Monoid::empty()`
returns one value and that's that. But in geometric computation (and many other
domains), there are fundamentally different *kinds* of emptiness:

| Kind | Example (Schubert calculus) | Meaning |
|------|---------------------------|---------|
| **Structural zero** | codim 10 > dim 4 | The question cannot even be posed |
| **Geometric zero** | codim 4 = dim 4, but LR coeff = 0 | The question is well-posed; the answer is zero |
| **Positive** | codim 4 = dim 4, LR coeff = 2 | Two solutions exist |
| **Underdetermined** | codim 2 < dim 4 | Infinitely many solutions |

ShaperOS encodes this as a type-level distinction:

```rust
enum EnumerationResult {
    Empty,                  // structural impossibility
    Finite(0),              // geometric zero — well-posed but unsatisfiable
    Finite(n),              // n solutions
    PositiveDimensional,    // infinite solutions
}
```

This distinction propagates through the entire system: capability grants,
memory recall, fallback strategies, and audit logging all behave differently
for structural vs geometric zeros.

### Why This Matters for Karpal

This pattern is not specific to Schubert calculus. It appears anywhere
"why is this empty?" carries information:

- **Type inference**: unification failure (structural) vs inhabitation
  failure (the type exists but has no values)
- **Constraint solving**: inconsistent constraints vs consistent but
  unsatisfiable
- **Database queries**: malformed query vs valid query with zero results
- **Effect systems**: impossible effect combination vs permitted but
  vacuous effect

No widely-used programming language or algebraic library formalizes this
distinction. Karpal can be the first.

**Phase 15 (`karpal-schubert-types`)** is the concrete realization of this
vision: types as Schubert classes, type checking as intersection computation,
and `IntersectionKind` as the structured emptiness lattice made computable.
The `BoundedLattice` from Phase 8 isn't just a library curiosity — it's the
truth value of the type system.

### Categorical Foundations

**Heyting-valued truth (Phase 8).** The enumeration result forms a
bounded lattice that serves as a multi-valued subobject classifier — a
generalization of `bool` from topos theory. Where standard logic has
`{true, false}`, structured emptiness has:

```
Ω = { Denied, Granted(0), Granted(n), Granted(∞) }
```

This is a Heyting algebra with `join`, `meet`, and an implication
operator, encodable as a `BoundedLattice` in Phase 8.

**Graded monoids and graded monads (Phases 8, 10).** When zeros carry
provenance, monoidal operations must preserve that provenance. A
`GradedMonoid<G>` where `G` is a grade lattice tracks *how* a value
became empty through `combine` operations. The fidelity-tracking
contraction fallback in ShaperOS is a graded writer monad:

```rust
// Conceptually:
type Recall<A> = Graded<FidelityGrade, Option<A>>
// where FidelityGrade: Monoid (worst degradation wins)
```

**Enriched categories (Phase 14).** The deepest connection. Standard
categories have hom-*sets* — morphisms either exist or don't. But when
hom-objects carry richer structure (the enumeration lattice), you get
a category *enriched* over that lattice. Composition must respect the
lattice structure: composing a geometric zero with a positive result
follows the Littlewood-Richardson rule, not simple boolean AND.

This means Karpal's enriched category encoding isn't just an abstract
exercise — it's the formalism that makes structured emptiness compose
correctly through chains of operations.

### Impact on Roadmap Phases

| Phase | Impact |
|-------|--------|
| **8 — Abstract Algebra** | `BoundedLattice` for enumeration results; `GradedMonoid` trait; Heyting algebra |
| **9 — Adjunctions** | Galois connections between lattices of structured truth values |
| **10 — Effects** | Graded monads tracking fidelity/degradation through monad transformer stacks |
| **11 — Proof** | Witnesses distinguishing structural impossibility from geometric vanishing |
| **13 — Diagrams** | String diagrams where wires carry lattice-valued annotations |
| **14 — Enriched categories** | Categories enriched over enumeration lattices; composition via LR-style rules |
| **15 — Schubert types** | Concrete realization: types as Schubert classes, type checking as intersection computation, `IntersectionKind` as the structured emptiness lattice |
| **16 — Topos** | Subobject classifier *is* the structured emptiness lattice; Schubert intersection *is* pullback; sheafification gives "local-to-global" composition of structured truth values |

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
├── karpal-higher/         # Phase 14: 2-categories, enriched categories
├── karpal-schubert-types/ # Phase 15: Schubert intersection type system (experimental)
└── karpal-topos/          # Phase 16: Topos theory, subobject classifiers, sheaves
```

## Syntax & Ergonomics

Karpal uses Haskell/PureScript typeclass semantics (Static Land: associated
functions on marker types), which is more natural for categorical reasoning
than Rust's method-on-value convention. Three macro families provide
ergonomic sugar on top of this foundation:

### Monadic sugar — `do!`, `ado!`, `free!` (Phases 2, 5, 10)

```rust
// do! — monadic bind (PureScript-style do notation)
let result = do_! {
    u <- get_user(id);
    a <- get_address(u);
    get_city(a)
};

// ado! — applicative do (PureScript's ado)
let area = ado! {
    w <- get_width();
    h <- get_height();
    yield w * h
};

// free! — free monad DSL
let program = free! {
    x <- GetLine;
    PutLine(format!("You said: {x}"));
    Pure(x)
};
```

### Compositional sugar — `.then()`, `>>>`, `diagram!` (Phases 4, 7, 13)

```rust
// Optic composition (already converging with Cliffy/Orlando)
let city_name = person_lens.then(address_lens).then(city_lens);

// Arrow pipelines
let pipeline = arr(parse) >>> arr(validate) >>> arr(save);

// String diagram DSL
let circuit = diagram! {
    f: A -> B,
    g: B -> C,
    h: A -> D,
    ---
    (f >>> g) *** h  // parallel + sequential composition
};
```

### Declarative sugar — `cata!`, `#[enriched]`, `prove!` (Phases 6, 8, 11, 14)

```rust
// Recursion scheme sugar
let eval = cata!(ExprF, |node| match node {
    Lit(n) => n,
    Add(a, b) => a + b,
});

// Enriched category declaration
#[enriched(over = "EnumerationLattice")]
trait MyCategory { /* hom(A, B) carries lattice structure */ }

// Proof witnesses
let v: Proven<Associative, MyOp> = prove!(MyOp is Associative);
```

### Syntax review checkpoints

These macro designs are provisional. Each phase should pause to
reconsider syntax when the actual type constraints become concrete —
decisions are best made when staring at real trait bounds, not in the
abstract.

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

## Research References

### Specifications & Textbooks

- [Fantasy Land Specification](https://github.com/fantasyland/fantasy-land)
- [Static Land Specification](https://github.com/fantasyland/static-land)
- [Haskell Typeclassopedia](https://wiki.haskell.org/Typeclassopedia)
- [Scala Typeclassopedia](https://github.com/lemastero/scala_typeclassopedia)
- [Edward Kmett's Haskell ecosystem](https://github.com/ekmett) (profunctors, kan-extensions, recursion-schemes, free, adjunctions)
- [Fantastic Morphisms and Where to Find Them](https://yangzhixuan.github.io/pdf/fantastic-morphisms.pdf)
- [Category Theory for Programmers](https://bartoszmilewski.com/2014/10/28/category-theory-for-programmers-the-preface/) — Bartosz Milewski
- Conal Elliott, "Compiling to Categories" (2018) — categorical semantics as compilation targets

### Papers — Rust Formal Verification (Phases 11-12)

- **REM2.0: Refactoring and Equivalence in Rust** — Britton, Pak, Potanin
  [arXiv:2601.19207](https://arxiv.org/abs/2601.19207v1).
  Rust → CHARON/AENEAS → Coq equivalence proofs. Directly applicable to
  verifying that Karpal's algebraic law implementations preserve semantics
  across refactorings.

- **RustyDL: A Program Logic for Rust**
  [arXiv:2602.22075](https://arxiv.org/abs/2602.22075v1).
  Source-level deductive verification for Rust without translation to
  intermediate languages. Could prove algebraic laws directly on Karpal
  trait implementations.

### Papers — Type Theory & HKT Foundations (Phases 1-3, 9, 14)

- **The ∞-category of ∞-categories in simplicial type theory** —
  Gratzer, Weinberger, Buchholtz
  [arXiv:2602.02218](https://arxiv.org/abs/2602.02218v1).
  Higher category theory done purely type-theoretically. Informs
  2-category and enriched category encoding in Phase 14.

- **The Leibniz adjunction in HoTT** — de Jong, Kraus, Ljungstrom
  [arXiv:2601.21843](https://arxiv.org/abs/2601.21843v1).
  Adjunctions formalized in Cubical Agda. Directly relevant to Phase 9
  adjunction design.

- **For Generalised Algebraic Theories, Two Sorts Are Enough** —
  Avrillon, Kaposi, Lafont, Najmaei, Rosain
  [arXiv:2601.19426](https://arxiv.org/abs/2601.19426v1).
  Any GAT reduces to two sorts via section-retraction. Theoretical
  validation of our GAT-based HKT encoding approach.

- **Impredicativity in Linear Dependent Type Theory** — Speight, van der Weide
  [arXiv:2602.08846](https://arxiv.org/abs/2602.08846v1).
  Linear + dependent types with cartesian and linear decodings. Informs
  ownership-aware algebraic abstractions in Rust.

- **Generalized Decidability via Brouwer Trees** — de Jong, Kraus et al.
  [arXiv:2602.10844](https://arxiv.org/abs/2602.10844v1).
  Decidability framework in HoTT using Brouwer ordinals. Relevant to
  constructive foundations underlying algebraic structure design.

### Papers — String Diagrams & Monoidal Categories (Phase 13)

- **Towards Term-based Verification of Diagrammatic Equivalence** —
  Cailler, Delorme, Perdrix, Tourret
  [arXiv:2602.11035](https://arxiv.org/abs/2602.11035v1).
  Automated reasoning about string diagram equivalence. Core to
  `karpal-diagram` design.

- **Simpler Presentations for Fragments of Quantum Circuits** — Blake
  [arXiv:2602.09874](https://arxiv.org/abs/2602.09874v1).
  Quantum circuits as symmetric monoidal categories (PROPs). Demonstrates
  the monoidal category abstractions Karpal should encode.

### Papers — Optics, Arrows & Functional Patterns (Phases 4, 7)

- **Lenses for Agent Introspection** — Hutton, Gibbons, Mehl
  [arXiv:2601.31746](https://arxiv.org/abs/2601.31746).
  Formalizes self-modification safety via lens laws. Validates the exact
  optic patterns Karpal encodes.

- **Programming Backpropagation with Reverse Handlers for Arrows** —
  Sanada, Hirai, Hoshino
  [arXiv:2602.18090](https://arxiv.org/abs/2602.18090v1).
  Arrows + reverse handlers for backpropagation. Validates arrow
  abstractions in Phase 4.

### Papers — Proof Theory & Recursion (Phase 6)

- **Making progress: Cut Elimination in the Ill-founded Realm** —
  Curzi, Leigh
  [arXiv:2602.01299](https://arxiv.org/abs/2602.01299v1).
  Cut elimination for mu-MALL with recursive/corecursive types. Directly
  relevant to recursion schemes (Fix/Nu) in Phase 6.

- **Proof Complexity of Linear Logics** — Tabatabai, Jalali
  [arXiv:2601.22393](https://arxiv.org/abs/2601.22393v1).
  Linear logic proof-size bounds. Informs resource-aware type reasoning.

### Papers — Schubert Calculus & Intersection Types (Phase 15)

- Fulton, *Young Tableaux* — Schubert calculus foundations (Littlewood-Richardson
  rule, cohomology of Grassmannians)
- Vakil, "Schubert Induction" — algorithmic intersection theory
- ShaperOS sasm spec (`docs/spec/shaper_asm.md`) — dual-domain execution model
  motivating Schubert intersection as cross-domain type checking

### Papers — Formal Verification at Scale (Phase 12)

- **AMBER: Construction-Verification Benchmark for Lean 4** — Yang et al.
  [arXiv:2602.01291](https://arxiv.org/abs/2602.01291v1).
  Lean 4 benchmark spanning convex analysis, optimization, numerical
  algebra. Relevant to Lean bridge design.

- **M2F: Automated Formalization at Scale** — Wang et al.
  [arXiv:2602.17016](https://arxiv.org/abs/2602.17016v1).
  Agentic framework converting textbooks → Lean with 96% proof success.
  Could automate formalization of algebraic properties.

### Papers — Effect Systems & Program Logic (Phase 10)

- **Handling Scope Checks (Extended)** — Lee, Xie, Kiselyov, Yallop
  [arXiv:2601.18793](https://arxiv.org/abs/2601.18793v1).
  Lambda-op calculus with effect handlers. Relevant to how HKT
  abstractions compose with effectful computations.

- **A Program Logic for Abstract (Hyper)Properties** — Baldan et al.
  [arXiv:2601.20370](https://arxiv.org/abs/2601.20370v1).
  Unifying Hoare-style logic for program correctness verification.

## License

MIT OR Apache-2.0
