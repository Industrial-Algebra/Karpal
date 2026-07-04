# Structured Emptiness: Zero-Intersection Semantics

**Status:** Design document (preliminary)
**Date:** 2026-07-03
**Authors:** Justin Elliott Cobb, Industrial Algebra LLC

## Motivation

Standard algebraic libraries treat zero as a single concept. `Monoid::empty()`
returns one value; `Option::None` means "absent"; `Result::Err` means "failure."
The *reason* for emptiness is lost.

In geometric computation — and in many other domains — there are fundamentally
different *kinds* of emptiness, and the distinction matters:

- A type check that **cannot be posed** (structural impossibility) is different
  from one that **can be posed but yields no solutions** (geometric emptiness).
- A constraint system that is **overdetermined** is different from one that is
  **underdetermined** (infinitely many solutions).
- An access control query that is **structurally denied** (incompatible
  capability classes) is different from one that is **geometrically denied**
  (compatible classes but zero intersection).

Structured emptiness is the thesis that these distinctions should be first-class
citizens in the type system, not runtime afterthoughts.

## The Lattice of Truth Values

We replace the boolean truth values `{false, true}` with a richer lattice Ω
that classifies *why* a computation yields a particular result:

| Truth Value | Meaning | Example |
|-------------|---------|---------|
| `Denied` | The question cannot be posed | Type mismatch, codim > dim |
| `Granted(0)` | The question is well-posed but the answer is zero | LR coefficient = 0 |
| `Granted(n)` | n ≥ 1 distinct solutions exist | LR coefficient = n |
| `Granted(∞)` | Infinitely many solutions | codim < dim |
| `Underdetermined` | Not enough information to decide | Computation incomplete |

This lattice is a **Heyting algebra** — a bounded lattice with implication and
negation that generalizes Boolean logic. In topos theory, Ω is the **subobject
classifier**: the object of truth values for the category.

## Mathematical Foundation

### Schubert Calculus as the Canonical Model

The canonical realization of structured emptiness is **Schubert intersection
theory on Grassmannians**. Given two Schubert classes σ_λ and σ_μ in Gr(k, n):

1. **Structural zero**: codim(λ) + codim(μ) > dim(Gr(k,n)) = k(n−k).
   The intersection cannot exist. This is a *dimension* argument — the question
   is ill-posed.

2. **Geometric zero**: codim(λ) + codim(μ) = dim(Gr) but the
   Littlewood-Richardson coefficient c^ν_{λμ} = 0 for all ν.
   The intersection is well-posed but empty. This requires *computation*.

3. **Positive**: codim(λ) + codim(μ) = dim(Gr) and c^ν_{λμ} > 0 for some ν.
   There are exactly c^ν_{λμ} intersection points.

4. **Underdetermined**: codim(λ) + codim(μ) < dim(Gr).
   The intersection is positive-dimensional — infinitely many points.

### The Heyting Algebra Structure

The truth value lattice Ω carries a Heyting algebra structure:

```
        Granted(∞)  (top)
           |
        Granted(n)
           |
        Granted(1)
           |
        Granted(0)
           |
        Underdetermined
           |
        Denied       (bottom)
```

- **Meet** (∧): the "worst case" — `Denied ∧ x = Denied`, `Granted(0) ∧ Granted(n) = Granted(0)`
- **Join** (∨): the "best case" — `Granted(n) ∨ Granted(0) = Granted(n)`
- **Implication** (→): Heyting implication, not Boolean — `Granted(0) → Denied` is not simply `true`
- **Negation** (¬): Heyting negation — `¬Granted(n)` is not `Granted(0)`

This is the internal logic of a **non-Boolean topos** — specifically, the topos
of sheaves over the structured emptiness topology.

## Type-Level Encoding in Karpal

### Current State (0.6.x)

`karpal-schubert-types` already encodes the four-valued version:

```rust
pub enum IntersectionKind {
    StructuralZero,   // codim > dim
    GeometricZero,    // codim ≤ dim, LR coeff = 0
    Positive,         // LR coeff > 0
    Underdetermined,  // codim < dim
}
```

This is a runtime classification — the intersection is computed and the kind is
determined by examining the result. The type system doesn't enforce the
distinction at compile time.

### Target State (0.7.0+)

The goal is to lift structured emptiness to the type level via:

1. **`HeytingAlgebra` trait** (Phase 16A) — extends `BoundedLattice` with
   implication and negation. The structured emptiness lattice Ω is a concrete
   instance.

2. **`EnrichedCategory<LRRing>`** (Phase 14D) — categories enriched over the
   LR coefficient ring, where hom-objects carry structured truth values.
   Composition of morphisms uses the Heyting meet, not Boolean AND.

3. **`SubobjectClassifier`** (Phase 16B) — the topos-theoretic Ω object,
   making structured emptiness the truth value object of a category of types.

4. **`SchubertProven<λ, T>`** with structured proof — currently the proof is
   binary (exists/doesn't exist). The target is a proof that carries the
   `IntersectionKind` as a phantom type parameter.

### The Enrichment Connection

The key mathematical insight is that **structured emptiness composes via
Littlewood-Richardson coefficients**, not via Boolean logic. When two type
checks are chained:

```
A → B → C
```

The compatibility of A with C is not `compatible(A,B) AND compatible(B,C)`.
It's `σ_A · σ_B · σ_C ≠ 0`, where the product is computed via LR coefficients.
The multiplicity (LR coefficient) gives the *number of distinct coercion paths*.

This is exactly an **enriched category** over the LR coefficient ring — the
hom-objects are formal sums of Schubert classes, and composition is the LR
product. Phase 14D formalizes this.

## Applications

### Type Checking as Intersection

In a type system with structured emptiness, type checking is:

```
check(T1, T2) = intersect(schubert_class(T1), schubert_class(T2))
```

The result is not `bool` but `IntersectionKind`:
- `StructuralZero`: the types are in incompatible Grassmannians — a type error
- `GeometricZero`: the types are compatible but no coercion exists
- `Positive(n)`: n distinct coercions exist (ambiguity if n > 1)
- `Underdetermined`: the types overlap in a positive-dimensional family

### Access Control as Intersection

Schubert (the access control crate) models capabilities as Schubert classes.
Access is granted when the intersection is non-zero:

```
access(capability, resource) = intersect(schubert_class(capability), schubert_class(resource))
```

The multiplicity gives the number of distinct access paths — useful for audit
trails and capability composition analysis.

### Constraint Solving as Intersection

Constraint satisfaction problems map to Schubert intersection:
- `StructuralZero`: the constraints are dimensionally incompatible
- `GeometricZero`: the constraints are compatible but unsatisfiable
- `Positive(n)`: exactly n solutions exist
- `Underdetermined`: a positive-dimensional solution space

### Interface Compatibility Scoring

API evolution analysis becomes quantitative:
- `Granted(0)`: breaking change (zero compatibility)
- `Granted(1)`: seamless migration
- `Granted(n>1)`: ambiguous migration (multiple valid adaptors)
- `Denied`: the APIs are in incompatible type spaces

## Implementation Roadmap

| Phase | Component | Deliverable |
|-------|-----------|-------------|
| 14D | `EnrichedCategory<LRRing>` for Schubert | LR-enriched category with LR product composition |
| 16A | `HeytingAlgebra` trait | Extends `BoundedLattice` with implication, negation; Ω as instance |
| 16B | `SubobjectClassifier` | Topos-theoretic Ω object; structured emptiness as truth values |
| 16C | Presheaf categories | Sheaves over the structured emptiness topology |
| 17A | E2E validation | Cross-crate scenarios exercising the full chain |

## The Deeper Claim

Structured emptiness is not just a programming technique. It's a claim about
the nature of computation:

> **The reason a computation yields no result is as important as the result
> itself. Zero is not a single value — it is a space of values, and the
> geometry of that space carries information.**

This is the thesis that separates Karpal from every other algebraic structures
library. Standard libraries treat emptiness as termination (`None`, `Err`,
`panic`). Karpal treats emptiness as **geometry** — and geometry has structure.
