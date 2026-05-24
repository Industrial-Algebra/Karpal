# karpal-schubert-types

Schubert intersection type system for the Industrial Algebra ecosystem.

`karpal-schubert-types` begins Phase 14 of the Karpal roadmap with:

- `SchubertType` — Schubert classes in Grassmannians as type-level markers
- `Intersection` — intersection computation via Littlewood-Richardson coefficients
- `IntersectionKind` — StructuralZero / GeometricZero / Positive / Underdetermined
- `SchubertTyped` trait — associate a Schubert class with a Rust type
- `SchubertProven<M, T>` — proof-carrying type assertions (Schubert analogue of `karpal_proof::Proven`)
- `compose_checks()` — chained type-check composition via the LR rule
- `schubert_bundle()` + `verify_schubert()` — external verification integration with `karpal-verify`

## Example

```rust
use karpal_schubert_types::{
    check_intersection, compose_checks, IntersectionKind, SchubertProven, SchubertType,
    SchubertTyped,
};

// σ₁ in Gr(2,4) — the class of lines meeting a fixed 2-plane
let sigma_1 = SchubertType::new(vec![1], (2, 4)).expect("valid");
let sigma_2 = SchubertType::new(vec![2], (2, 4)).expect("valid");

// Intersection of two σ₁ classes in Gr(2,4) is positive-dimensional
let result = check_intersection(&sigma_1, &sigma_1);
assert_eq!(result.kind(), IntersectionKind::Positive);

// σ₂₂ · σ₂₂ is a structural zero (codim 8 > dim 4)
let sigma_22 = SchubertType::new(vec![2, 2], (2, 4)).expect("valid");
let zero = check_intersection(&sigma_22, &sigma_22);
assert_eq!(zero.kind(), IntersectionKind::StructuralZero);

// Proof-carrying values via SchubertTyped
struct Sigma1;
impl SchubertTyped for Sigma1 {
    fn schubert_type() -> SchubertType {
        SchubertType::new(vec![1], (2, 4)).expect("σ₁")
    }
}

let proven = SchubertProven::<Sigma1, &str>::new("data");
assert!(proven.check_against::<Sigma1>().is_some());

// Chained composition
let chain = compose_checks::<Sigma1, Sigma1, Sigma1>();
assert!(chain.is_some());

// Verification certificates
let certs = karpal_schubert_types::verification::verify_schubert();
assert_eq!(certs.obligations.len(), 3);
```

## License

AGPL-3.0-or-later
