# Schubert Intersection Type System — Karpal Experimental Extension

**Status:** Synopsis / research direction
**Date:** March 2026
**Context:** Discovered via ShaperOS sasm dual-domain execution model (cross-domain type checking between rewrite rules and register VM). See ShaperOS `docs/spec/shaper_asm.md` § Cross-Domain Programs, § Open Questions #7.

---

## Summary

A type system where **types are Schubert classes in a Grassmannian** and **type checking is computing intersection numbers**. This is not a metaphor — it's a literal formalization arising from the sasm bytecode's need to verify that declarative rewrite-rule patterns satisfy imperative function signatures across execution domain boundaries.

Karpal is the natural testbed: it already has the type class hierarchy (HKT, Functor, Monad), optics (Lens, Prism), and planned proof infrastructure (Phases 11-12). A `karpal-schubert-types` crate would implement this against `amari-enumerative`'s Schubert calculus backends, validating the theory before it's built into the Shaper compiler (Lang-2.4).

Beyond ShaperOS, this has potential as a **general Rust development tool** — Schubert intersection types formalize concepts that existing type systems approximate: subtyping with multiplicity, compositional capability checking, and a mathematically grounded answer to "how compatible are these two interfaces?"

---

## Core Idea

### Types as Schubert Classes

In a Grassmannian Gr(k, n), Schubert classes σ_λ (indexed by Young partitions λ) form a basis for the cohomology ring. The product of two classes decomposes via Littlewood-Richardson coefficients:

```
σ_λ · σ_μ = Σ c^ν_{λμ} σ_ν
```

A Schubert intersection type system maps this directly:

| Type Theory | Schubert Calculus |
|---|---|
| Type | Schubert class σ_λ in Gr(k, n) |
| Subtype relation `A <: B` | Intersection `σ_A · σ_B ≠ 0` |
| Compatibility | LR coefficient c^ν_{AB} > 0 |
| Number of coercion paths | LR coefficient c^ν_{AB} (multiplicity) |
| Union type `A \| B` | Sum of Schubert classes |
| Grade constraint `Blade[k]` | Fixing the Schubert cell dimension |
| Function type `A → B` | Morphism between Schubert cells |
| Capability scope | Schubert class intersection in the Grassmannian |

### What Makes This Different from Existing Type Systems

**Multiplicity.** Standard subtyping is boolean: A <: B or not. Schubert intersection gives an integer — the number of coercion paths. When c^ν_{AB} = 3, there are three distinct ways to view an A as a B. The compiler can:
- Require disambiguation when multiplicity > 1
- Use multiplicity as a measure of "how compatible" two types are
- Report multiplicity in error messages: "types incompatible (intersection = 0)" vs "3 coercions available, please disambiguate"

**Computation paths.** amari-enumerative provides four independent algorithms for computing intersection numbers: Littlewood-Richardson, equivariant localization, tropical geometry, and matroid theory. The type checker can use the fastest path for each query, with cross-verification in debug builds.

**Compositional.** Schubert calculus is inherently compositional — operadic composition of intersection queries follows from the algebra. Checking a chain of function calls is a sequence of intersection computations that compose via the LR rule, not ad-hoc unification.

**Graded.** Types carry grade information naturally. A `Blade[2]` type lives in a specific Schubert cell; a `Blade[*]` type is a union over cells. Grade inference is just tracking which cells are occupied through a computation.

---

## Benefits to General Rust Development

While the primary motivation is ShaperOS, the underlying concepts address real pain points in Rust's type system:

### 1. Trait Compatibility with Multiplicity

Rust's trait system is boolean: a type implements a trait or it doesn't. But real-world APIs often have *multiple valid implementations* (e.g., `Semigroup` via `Sum` or `Product`). Schubert types formalize this as multiplicity > 1, with the newtype markers (`Sum<T>`, `Product<T>`) being explicit disambiguation of a multiplicity-2 intersection.

A `karpal-schubert-types` crate could provide:

```rust
/// How many ways does T satisfy trait bound B?
fn compatibility<T, B>() -> Multiplicity;

/// Which specific coercion path does this newtype select?
fn coercion_path<Sum<T>, Semigroup>() -> CoercionWitness;
```

This is already what Karpal does informally with newtype markers — Schubert types formalize the "why" and make it computable.

### 2. Capability-Based Access Control

Any system using capability tokens (API keys, permission sets, RBAC roles) can model capabilities as Schubert classes. Two principals can interact iff their capability classes have nonzero intersection. The multiplicity tells you how many interaction modes exist.

```rust
/// Capability intersection: can principal A access resource B?
fn check_access(a: &Capability, b: &Capability) -> AccessResult {
    match intersection(a.class(), b.class()) {
        0 => AccessResult::Denied,            // structural impossibility
        n => AccessResult::Granted { paths: n }, // n interaction modes
    }
}
```

This generalizes beyond ShaperOS to any Rust service with fine-grained access control.

### 3. Interface Compatibility Scoring

When refactoring or evolving APIs, the question isn't just "does this compile?" but "how compatible is the new interface with the old one?" Schubert intersection gives a quantitative answer:

- Intersection = 0: breaking change (no coercion possible)
- Intersection = 1: seamless migration (unique coercion)
- Intersection > 1: compatible but ambiguous (migration needs guidance)
- Intersection structure: which parts of the interface are compatible and which break

This could power tooling for API evolution analysis in any Rust project.

### 4. Enriched Error Messages

Type errors become geometrically informative:

```
error[E0308]: mismatched types
  --> src/main.rs:42:5
   |
42 |     process(data)
   |     ^^^^^^^ expected `Blade[2]`, found `Blade[1|3]`
   |
   = note: Schubert intersection σ[2] · σ[1,3] = 0 in Gr(3,6)
   = note: types occupy disjoint Schubert cells — no coercion exists
   = help: grade-project to Blade[2] first: `⟨data⟩₂`
```

vs the positive case:

```
warning: ambiguous coercion
  --> src/main.rs:42:5
   |
42 |     process(data)
   |     ^^^^^^^ 3 coercion paths from `A` to `B`
   |
   = note: σ_A · σ_B = 3 (LR coefficient via tropical path)
   = help: disambiguate with explicit projection
```

### 5. Proof-Carrying Type Assertions

Combined with Phase 11 (`karpal-proof`), Schubert type assertions become proof witnesses:

```rust
/// A value whose type membership has been verified by intersection computation.
struct SchubertProven<λ: Partition, T> {
    value: T,
    _class: PhantomData<λ>,
}

impl<λ: Partition, T> SchubertProven<λ, T> {
    /// Construct only if T's Schubert class intersects σ_λ.
    fn verify(value: T) -> Result<Self, IntersectionZero> {
        // ... computed via amari-enumerative
    }
}
```

Combined with Phase 12 (`karpal-verify`), the intersection computation itself can be exported to Z3/Lean for independent verification.

---

## Proposed Crate: `karpal-schubert-types`

### Position in Karpal Roadmap

This could be:
- **Phase 12.5** (branching from Phase 12's external prover work), or
- **Phase 15** (after the enriched categories foundation in Phase 14), or
- A **standalone experimental crate** developed in parallel with Phases 11-12

The enriched categories connection (Phase 14) is deep — Schubert intersection types are a category enriched over the LR coefficient ring — but the core type-checking machinery can be built independently.

### Dependencies

```
karpal-schubert-types/
  depends on:
    karpal-core       — HKT, Functor, Monad (trait hierarchy foundation)
    karpal-proof      — Proven<P, T> witnesses (Phase 11)
    karpal-algebra    — BoundedLattice, Heyting algebra (Phase 8)
    amari-enumerative — SchubertClass, LR coefficients, 4 computation paths
    amari-core        — Multivector<P,Q,R> (the values being typed)
  optional:
    karpal-verify     — SMT-LIB2 export of intersection queries (Phase 12)
    karpal-optics     — Lens/Prism over Schubert-typed values
    karpal-higher     — Enriched category formalization (Phase 14)
```

### Core API Sketch

```rust
/// A Schubert type: a class (or union of classes) in Gr(k, n).
pub struct SchubertType {
    grassmannian: (usize, usize),   // Gr(k, n)
    classes: Vec<SchubertClass>,     // union of Schubert cells
}

/// Intersection result with full diagnostic information.
pub struct Intersection {
    /// Total multiplicity (sum of LR coefficients).
    pub multiplicity: u64,
    /// Per-class decomposition.
    pub components: Vec<(SchubertClass, u64)>,
    /// Which computation path was used.
    pub path: ComputationPath,
    /// Structured emptiness classification.
    pub classification: IntersectionKind,
}

pub enum IntersectionKind {
    /// Structural impossibility: codimensions don't fit.
    Structural,
    /// Well-posed but zero: LR coefficient = 0.
    GeometricZero,
    /// Positive: multiplicity coercion paths exist.
    Positive(u64),
    /// Underdetermined: infinite-dimensional intersection.
    Underdetermined,
}

/// The core type-checking operation.
pub fn check_intersection(a: &SchubertType, b: &SchubertType) -> Intersection;

/// Subtyping as nonzero intersection.
pub fn is_subtype(sub: &SchubertType, sup: &SchubertType) -> bool {
    check_intersection(sub, sup).multiplicity > 0
}

/// Compose type checks (operadic composition of intersection queries).
pub fn compose_checks(chain: &[SchubertType]) -> Intersection;
```

### Trait Integration

```rust
/// A type that has an associated Schubert class.
pub trait SchubertTyped {
    /// The Grassmannian this type lives in.
    fn grassmannian() -> (usize, usize);
    /// The Schubert class (or union) representing this type.
    fn schubert_class() -> SchubertType;
}

/// Blanket impl for amari-core multivectors.
impl<const P: usize, const Q: usize, const R: usize> SchubertTyped
    for Multivector<P, Q, R>
{
    fn grassmannian() -> (usize, usize) {
        // Derived from signature
    }
    fn schubert_class() -> SchubertType {
        // Full algebra = union of all Schubert cells
    }
}

/// Grade-constrained blades have more specific Schubert classes.
impl<const P: usize, const Q: usize, const R: usize, const K: usize> SchubertTyped
    for GradeBlade<K, P, Q, R>
{
    fn schubert_class() -> SchubertType {
        // Single Schubert cell for grade K
    }
}
```

### Connection to Structured Emptiness

Schubert intersection types are the **computation-level realization** of Karpal's structured emptiness concept. The `IntersectionKind` enum is exactly the enumeration lattice from the Structured Emptiness section of the roadmap:

| Structured Emptiness | IntersectionKind | Type System Meaning |
|---|---|---|
| Structural zero | `Structural` | Types can't even be compared (wrong Grassmannian, grade mismatch) |
| Geometric zero | `GeometricZero` | Types are comparable but incompatible (LR = 0) |
| Positive | `Positive(n)` | n coercion paths exist |
| Underdetermined | `Underdetermined` | Infinite coercions (e.g., `Blade[*]` to `Blade[*]`) |

This means the Heyting algebra / bounded lattice from Phase 8 isn't just a library curiosity — it's the **truth value of the type system**.

### Connection to karpal-proof (Phase 11)

`Proven<SchubertCompatible<A, B>, T>` is a value of type T carrying a phantom-typed witness that A and B have nonzero Schubert intersection. The proof is the LR coefficient computation itself. This connects to ShaperOS's Constitution Kernel concept: constitutional guards are type-level proofs of access rights, verified by intersection computation.

### Connection to karpal-verify (Phase 12)

Intersection queries can be exported as SMT-LIB2 problems:

```smt2
(declare-const lr_coeff Int)
(assert (= lr_coeff (lr-coefficient (partition 2 1) (partition 1 1) (grassmannian 2 4))))
(assert (> lr_coeff 0))
(check-sat)
```

This enables independent verification of type judgments by Z3/CVC5, and the Lean 4 bridge can formalize the LR rule itself.

### Connection to karpal-higher (Phase 14)

The deepest connection: a category where hom-objects are Schubert intersection numbers is a category **enriched over the LR coefficient ring**. This means:
- Composition of morphisms follows the LR rule (not simple function composition)
- The identity morphism has multiplicity 1 (σ_∅ · σ_λ = σ_λ)
- Associativity of composition follows from associativity of the LR rule

This is a concrete, non-trivial example of an enriched category — possibly the most natural one for computational mathematics. It would validate and motivate the Phase 14 enriched category abstractions.

---

## Implementation Sketch

### Phase A: Core Intersection Engine

- `SchubertType`, `Intersection`, `IntersectionKind` types
- `check_intersection()` backed by amari-enumerative (LR path initially, add others later)
- `SchubertTyped` trait with blanket impls for `Multivector` and `GradeBlade`
- Property tests: intersection is commutative, associative, respects grade constraints
- Benchmark against direct amari-enumerative calls (should be thin wrapper, <5% overhead)

### Phase B: Proof Integration

- `SchubertProven<λ, T>` witness type (depends on karpal-proof Phase 11)
- `verify()` constructor that computes intersection and returns proof or error
- Composition: `SchubertProven<λ, T>` + `SchubertProven<μ, U>` → `SchubertProven<ν, V>` via LR
- Connect to structured emptiness: proof failures carry `IntersectionKind` diagnostics

### Phase C: External Verification

- SMT-LIB2 export for intersection queries (depends on karpal-verify Phase 12)
- Lean 4 export for the LR rule itself
- Three-tier classification: structural impossibility (type-level), geometric zero (SMT-verified), positive (computed)

### Phase D: Enriched Category Formalization

- Schubert intersection as enrichment (depends on karpal-higher Phase 14)
- LR coefficient ring as the enriching category
- Demonstrate composition respects LR rule
- String diagram visualization of Schubert type derivations (depends on karpal-diagram Phase 13)

---

## Open Questions

1. **Grassmannian selection.** Which Gr(k, n) does a given Rust type live in? For amari multivectors, the signature determines this. For general Rust types, this needs a mapping — possibly via trait annotations or derivation.

2. **Performance.** LR coefficient computation is polynomial but not free. Is it fast enough for interactive type checking in a compiler? Tropical path is fastest; caching helps. The sasm validator only needs it at cross-domain boundaries, not for every expression.

3. **Expressiveness.** Can all useful type relationships be expressed as Schubert intersections? Some type constraints (lifetime bounds, trait object safety) may not have natural Schubert representations. The system may need to compose with conventional type checking rather than replacing it.

4. **Inference.** Hindley-Milner inference works by unification. Schubert intersection is not unification — it's a richer operation (multiplicity, computation paths, structured emptiness). What does inference look like in this setting? Possibly constraint propagation over LR coefficients.

5. **Ergonomics.** How does a Rust developer interact with Schubert types without needing to understand algebraic geometry? The error messages help, but the type annotations may need sugar that hides the Grassmannian structure.

---

## References

- ShaperOS sasm spec: `docs/spec/shaper_asm.md` (dual-domain execution, cross-domain type checking)
- ShaperOS kernel research: `docs/research/kernel_architecture.md` (Constitution Kernel verification overlay)
- Karpal ROADMAP.md Phases 11-12 (proof witnesses, external prover bridge)
- Karpal ROADMAP.md § Structured Emptiness (Heyting algebra, enriched categories)
- amari-enumerative: SchubertClass, LR coefficients, localization, tropical, matroid paths
- Constitution Kernel doc: `IA-documents/IA/CONSTITUTION_KERNEL_VERIFICATION.md`
- Fulton, *Young Tableaux* — Schubert calculus foundations
- Vakil, "Schubert Induction" — algorithmic intersection theory

---

*Synopsis — March 2026*
