# Schubert Types

Schubert intersection type system — `karpal-schubert-types` (Phase 14 A–C).


## Overview

Types are Schubert classes σ<sub>λ</sub> in a Grassmannian Gr(k, n), and type compatibility is computed via Littlewood-Richardson intersection coefficients. Two types are compatible when their Schubert classes intersect nontrivially (σ<sub>A</sub> · σ<sub>B</sub> ≠ 0). The LR coefficient gives the *multiplicity* — the number of distinct coercion paths.


## SchubertType

A Schubert class indexed by a partition (Young diagram) in a Grassmannian:

``` rust
use karpal_schubert_types::SchubertType;

// σ₁ in Gr(2,4) — lines meeting a fixed 2-plane
let sigma_1 = SchubertType::new(vec![1], (2, 4)).expect("valid");

// σ₂₂ in Gr(2,4) — point class
let sigma_22 = SchubertType::new(vec![2, 2], (2, 4)).expect("valid");

// Partition entry exceeds box bound → error
assert!(SchubertType::new(vec![3], (2, 4)).is_err());

assert_eq!(sigma_1.codimension(), 1); // sum of partition entries
assert_eq!(sigma_22.codimension(), 4);
```


## Intersection

`check_intersection(a, b)` computes the intersection product via `amari-enumerative` and classifies the result:

| Kind              | Meaning                                          |
|-------------------|--------------------------------------------------|
| `StructuralZero`  | Total codimension exceeds Grassmannian dimension |
| `GeometricZero`   | Correctly dimensioned but no intersection points |
| `Positive`        | Nonempty intersection with known multiplicity    |
| `Underdetermined` | Computation could not resolve the result         |

``` rust
use karpal_schubert_types::{check_intersection, IntersectionKind, SchubertType};

let s1 = SchubertType::new(vec![1], (2, 4)).unwrap();
let s22 = SchubertType::new(vec![2, 2], (2, 4)).unwrap();

// σ₁ · σ₁ is positive-dimensional
let result = check_intersection(&s1, &s1);
assert_eq!(result.kind(), IntersectionKind::Positive);

// σ₂₂ · σ₂₂ is a structural zero (codim 8 > dim 4)
let zero = check_intersection(&s22, &s22);
assert_eq!(zero.kind(), IntersectionKind::StructuralZero);
assert_eq!(zero.multiplicity(), 0);
```


## SchubertTyped & SchubertProven

`SchubertTyped` associates a Schubert class with a Rust type. `SchubertProven<M, T>` is the Schubert analogue of `karpal_proof::Proven<P, T>`:

``` rust
use karpal_schubert_types::{SchubertProven, SchubertType, SchubertTyped};

// Declare a marker type
struct Sigma1;

impl SchubertTyped for Sigma1 {
    fn schubert_type() -> SchubertType {
        SchubertType::new(vec![1], (2, 4)).expect("σ₁")
    }
}

// Wrap a value with type-level proof
let proven = SchubertProven::::new("my_data");
assert_eq!(*proven.value(), "my_data");

// Check compatibility with another type
assert!(proven.check_against::().is_some());

// Unwrap
assert_eq!(proven.into_inner(), "my_data");
```


## Chained Composition

`compose_checks::<A, B, C>()` verifies a chain of type compatibilities via the LR rule:

``` rust
use karpal_schubert_types::compose_checks;

// Verify A → B → C composition chain
let chain = compose_checks::();
assert!(chain.is_some());
```


## External Verification

Schubert calculus properties export as `karpal-verify` obligation bundles:

``` rust
use karpal_schubert_types::verification::verify_schubert;

let report = verify_schubert();
assert_eq!(report.obligations.len(), 3);

for obl in &report.obligations {
    assert!(obl.certificate.is_some());
}
```


