# Structured Emptiness

## The Problem

Standard libraries treat emptiness as a single concept: `None`, `Err`, `0`, `empty()`. The *reason* for emptiness is lost.

In geometric computation — and many other domains — there are fundamentally different *kinds* of emptiness:

| Kind | Meaning | Example |
|------|---------|---------|
| Structural zero | The question cannot be posed | codim > dim |
| Geometric zero | Well-posed but no solutions | LR coefficient = 0 |
| Positive | n solutions exist | LR coefficient = n |
| Underdetermined | Infinitely many solutions | codim < dim |

## The Lattice Ω

Karpal replaces boolean truth values with a richer lattice:

```
Denied < Granted(0) < Granted(1) < ... < Granted(∞)
```

This is a **Heyting algebra** — a bounded lattice with implication where the law of excluded middle does not hold (`¬¬a ≠ a` in general).

## Implementation

The concrete realization is via Schubert calculus on Grassmannians. Given two Schubert classes σ_λ and σ_μ in Gr(k, n):

- Their intersection product is computed via Littlewood-Richardson coefficients
- The result is classified as `StructuralZero`, `GeometricZero`, `Positive`, or `Underdetermined`
- Composition of intersections uses the lattice meet (worst-case propagation)

```rust
use karpal_schubert_types::{check_intersection, IntersectionKind, SchubertType};

let s1 = SchubertType::new(vec![1], (2, 4)).unwrap();
let s22 = SchubertType::new(vec![2, 2], (2, 4)).unwrap();

assert_eq!(check_intersection(&s1, &s1).kind(), IntersectionKind::Positive);
assert_eq!(check_intersection(&s22, &s22).kind(), IntersectionKind::StructuralZero);
```

## The Deeper Claim

> The reason a computation yields no result is as important as the result itself. Zero is not a single value — it is a space of values, and the geometry of that space carries information.

See the [design document](https://github.com/Industrial-Algebra/Karpal/blob/develop/docs/design/structured-emptiness.md) for the full mathematical treatment.
