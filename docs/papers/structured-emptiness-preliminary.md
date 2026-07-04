# Structured Emptiness: Zero-Intersection Semantics for Type Systems

**Justin Elliott Cobb**
Industrial Algebra LLC
July 2026

## Abstract

We present *structured emptiness*, a type-level framework that distinguishes
different kinds of computational absence — structural impossibility, geometric
unsatisfiability, positive solutions, and underdetermined solution spaces — as
first-class citizens in a programming language's type system. The framework
replaces boolean truth values with a Heyting algebra of intersection kinds,
grounded in Schubert calculus on Grassmannians. We show that type checking,
access control, constraint solving, and interface compatibility analysis all
admit a natural interpretation as Schubert intersection problems, where the
Littlewood-Richardson coefficient gives not just existence but multiplicity of
solutions. We describe an implementation in the Rust programming language via
the Karpal library, demonstrating that structured emptiness composes correctly
through enriched category theory and connects to topos theory via the subobject
classifier.

## 1. Introduction

### 1.1 The Boolean Blindness Problem

Type systems traditionally answer questions with booleans: a type check passes
or fails, a constraint is satisfied or not, a capability is granted or denied.
This binary classification discards the *reason* for a negative result.

Consider a type checker that rejects a program. The rejection could mean:
- The types are in incompatible universes (a structural error)
- The types are compatible but no coercion exists (a geometric zero)
- The types are compatible but the coercion is ambiguous (multiple paths)
- The types overlap in a family of possible coercions (underdetermined)

Standard type systems conflate all four cases into a single "type error."
Structured emptiness argues that these distinctions are mathematically
fundamental and practically useful.

### 1.2 The Geometric Insight

In enumerative geometry, the intersection of two subvarieties is never simply
"empty" or "non-empty." Schubert calculus provides a refined answer: the
intersection product of two Schubert classes σ_λ · σ_μ decomposes as a sum
of Schubert classes with non-negative integer coefficients
(Littlewood-Richardson coefficients):

σ_λ · σ_μ = Σ_ν c^ν_{λμ} σ_ν

The coefficient c^ν_{λμ} counts the number of points in the intersection of
the corresponding Schubert varieties. When the sum is zero, we can ask *why*:

- Is it because the total codimension exceeds the ambient dimension?
  (Structural zero — the intersection cannot exist)
- Or because the Littlewood-Richardson rule yields zero for all ν?
  (Geometric zero — the intersection is well-posed but empty)

This distinction is the foundation of structured emptiness.

## 2. The Structured Emptiness Lattice

### 2.1 Definition

We define the *structured emptiness lattice* Ω as the following bounded lattice:

Ω = {Denied, Granted(0), Granted(1), Granted(2), ..., Granted(∞), Underdetermined}

with the partial order:

Denied < Granted(0) < Granted(1) < ... < Granted(∞)
                                      |
                              Underdetermined

where:
- **Denied**: the computation cannot be posed (structural impossibility)
- **Granted(n)** for n ≥ 0: the computation yields exactly n solutions
- **Granted(∞)**: the computation yields infinitely many solutions
- **Underdetermined**: the computation has not been resolved to a definite count

### 2.2 Heyting Algebra Structure

Ω carries a Heyting algebra structure (a distributive lattice with a relative
pseudo-complement). The Heyting implication a → b is the largest c such that
a ∧ c ≤ b. Unlike Boolean logic, the law of excluded middle does not hold:
¬¬a ≠ a in general.

This makes Ω the natural truth value object for a non-Boolean topos —
specifically, the topos of sheaves over the structured emptiness topology.

### 2.3 Connection to Schubert Calculus

The concrete realization of Ω is via Littlewood-Richardson coefficients. Given
Schubert classes σ_λ and σ_μ in Gr(k, n):

| Ω Value | Condition | Mathematical Meaning |
|----------|-----------|---------------------|
| Denied | codim(λ) + codim(μ) > k(n−k) | Dimensional impossibility |
| Granted(0) | codim sum = dim, all c^ν_{λμ} = 0 | Well-posed but empty |
| Granted(n) | codim sum = dim, Σ_ν c^ν_{λμ} = n | n intersection points |
| Granted(∞) | codim sum < dim | Positive-dimensional intersection |
| Underdetermined | computation incomplete | Not yet resolved |

## 3. Enriched Category Theory Formulation

### 3.1 Categories Enriched over the LR Ring

The key mathematical insight is that structured emptiness *composes* via
Littlewood-Richardson coefficients, not via Boolean logic. We model this as a
category enriched over the LR coefficient ring.

Let V be the monoidal category of formal Z≥0-linear combinations of Schubert
classes, with tensor product given by the LR product. A V-enriched category C
has:

- Objects: types (or Rust types, in our implementation)
- Hom-objects: C(A, B) ∈ V, a formal sum of Schubert classes
- Composition: C(A, B) ⊗ C(B, C) → C(A, C) via the LR product
- Identity: the unit of V → C(A, A)

The truth value of "is A compatible with B?" is not bool but the Ω-value
obtained by evaluating C(A, B) as an intersection.

### 3.2 Multiplicity-Aware Compatibility

The LR coefficient c^ν_{λμ} gives the *multiplicity* of solutions — the number
of distinct coercion paths from type λ to type μ. This is strictly more
informative than boolean compatibility:

| LR Coefficient | Boolean | Structured Emptiness |
|----------------|---------|---------------------|
| 0 | false | Granted(0) — incompatible but well-posed |
| 1 | true | Granted(1) — unique coercion |
| n > 1 | true | Granted(n) — ambiguous: n coercions exist |

Ambiguity (n > 1) is actionable information: the programmer must disambiguate,
and the number n tells them how many options they have.

## 4. Implementation in Rust

### 4.1 The Karpal Library

We implement structured emptiness in the Karpal library, a category theory
library for Rust. The implementation spans four crates:

- `karpal-schubert-types`: runtime `SchubertType`, `Intersection`, `IntersectionKind`
- `karpal-higher`: `EnrichedCategory<V>` trait for enriched category theory
- `karpal-proof`: type-level `Justifies<Lhs, Rhs>` witnesses for proof-carrying code
- `karpal-verify`: external verification (SMT-LIB2, Lean 4, Kani) of LR computations

### 4.2 The IntersectionKind Enum

The four-valued version is already implemented:

```rust
pub enum IntersectionKind {
    StructuralZero,
    GeometricZero,
    Positive,
    Underdetermined,
}
```

The target is to lift this to a phantom-typed proof:

```rust
pub struct SchubertProven<M: SchubertTyped, T> {
    value: T,
    _marker: PhantomData<M>,
    cached_type: SchubertType,
}
```

where the marker type M encodes the Schubert class at the type level, and
intersection checks produce `Rewrite<M1, M2, ByIntersection>` proof terms.

### 4.3 Verification Pipeline

The structured emptiness claims are externally verifiable:

1. Generate `ObligationBundle` for LR consistency, partition validity, and
   intersection emptiness
2. Export to SMT-LIB2 (finite, exhaustive for small Grassmannians) or Lean 4
   (structural arguments requiring induction)
3. Import `Certificate` witnesses via the `ProofBridge` trust boundary
4. Produce `SchubertProven` values backed by external proof

## 5. Applications

### 5.1 Type Checking as Schubert Intersection

In a type system with structured emptiness, type compatibility is:

compat(A, B) = intersect(schubert_class(A), schubert_class(B))

yielding an `IntersectionKind`, not a boolean. Error messages become structured:
"StructuralZero: types inhabit incompatible Grassmannians Gr(2,4) and Gr(3,6)"
versus "GeometricZero: LR coefficient is zero for all ν in Gr(2,4)."

### 5.2 Capability-Based Access Control

Capabilities and resources are Schubert classes. Access is granted iff the
intersection is non-zero. The multiplicity counts access paths — useful for
audit trails and capability composition analysis.

### 5.3 Interface Compatibility Scoring

API evolution analysis becomes quantitative. A breaking change yields
Granted(0); a seamless migration yields Granted(1); an ambiguous migration
yields Granted(n) for n > 1.

### 5.4 Constraint Solving

Constraint satisfaction maps to Schubert intersection:
- StructuralZero: constraints are dimensionally incompatible
- GeometricZero: constraints are compatible but unsatisfiable
- Granted(n): exactly n solutions
- Granted(∞): positive-dimensional solution space

## 6. Related Work

### 6.1 Refinement Types

Refinement types (LiquidHaskell, F*) constrain types with logical predicates.
Structured emptiness generalizes this by replacing boolean predicates with
Heyting-valued predicates carrying geometric provenance.

### 6.2 Intersection Type Systems

Classical intersection type systems (Coppo, Dezani) use type intersection
for lambda calculus. Our system uses *Schubert* intersection — a geometrically
grounded intersection with multiplicities from algebraic geometry.

### 6.3 Gradual Typing

Gradual typing (Siek, Taha) introduces "unknown" as a third truth value
(beyond definitely-typed and definitely-untyped). Structured emptiness provides
a richer lattice: Denied (definitely incompatible), Granted(0) (compatible
but no coercion), Granted(n) (n coercions), Underdetermined (not yet resolved).

### 6.4 Algebraic Effect Handlers

Algebraic effect systems (Koka, Eff) track effects at the type level.
Structured emptiness is orthogonal — it tracks the *geometry of type
compatibility*, not the effects of computation. The two compose: an effect
handler can itself be analyzed for Schubert intersection compatibility.

## 7. Future Directions

### 7.1 Topos Theory

The structured emptiness lattice Ω is the subobject classifier of a topos.
Phase 16 of the Karpal roadmap formalizes this: presheaf categories, sheaves,
and Lawvere-Tierney topologies provide the categorical foundation for
Ω-valued logic.

### 7.2 ∞-Categories

The Heyting algebra Ω is a 1-categorical object. In an ∞-categorical setting,
the "space of reasons for emptiness" becomes a higher structure — potentially
an ∞-groupoid of proof witnesses. This connects to homotopy type theory.

### 7.3 Machine Learning

Structured emptiness provides a natural loss function for type inference
algorithms: the "distance" from Granted(0) to Granted(1) is meaningful,
and the multiplicity n measures ambiguity. This could inform neural-guided
type inference systems.

## 8. Conclusion

Structured emptiness is the thesis that computational absence has geometry.
The zero of a computation is not a single point but a space, and the structure
of that space carries information — information that standard type systems
discard. By grounding this structure in Schubert calculus and formalizing it
through enriched category theory, we obtain a type system where:

- Every "no" carries a reason
- Every "yes" carries a count
- Composition preserves structure through LR coefficients
- The truth values form a Heyting algebra, not a Boolean algebra

This is not merely a programming technique. It is a claim about the nature
of computation: that the geometry of absence is as fundamental as the
geometry of presence.

## References

1. Fulton, W. *Young Tableaux*. LMS Student Texts 35, Cambridge University Press, 1997.
2. Mac Lane, S. and Moerdijk, I. *Sheaves in Geometry and Logic*. Springer, 1992.
3. Karpal library. https://github.com/Industrial-Algebra/Karpal
4. Schubert calculus on Grassmannians. See `amari-enumerative` crate documentation.
5. Littlewood-Richardson rule. Fulton [1], Chapter 5.
6. Heyting algebras and topos theory. Mac Lane and Moerdijk [2], Chapters I–IV.

---

*This is a preliminary draft. Comments welcome.*
