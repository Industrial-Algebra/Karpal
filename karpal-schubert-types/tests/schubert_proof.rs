// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use karpal_schubert_types::{
    IntersectionKind, SchubertProven, SchubertType, SchubertTyped, compose_checks,
};

// ---------------------------------------------------------------------------
// SchubertTyped marker types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct Sigma1;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Sigma2;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Sigma22;

impl SchubertTyped for Sigma1 {
    fn schubert_type() -> SchubertType {
        SchubertType::new(vec![1], (2, 4)).expect("valid σ₁")
    }
}

impl SchubertTyped for Sigma2 {
    fn schubert_type() -> SchubertType {
        SchubertType::new(vec![2], (2, 4)).expect("valid σ₂")
    }
}

impl SchubertTyped for Sigma22 {
    fn schubert_type() -> SchubertType {
        SchubertType::new(vec![2, 2], (2, 4)).expect("valid σ₂₂")
    }
}

// ---------------------------------------------------------------------------
// SchubertProven tests
// ---------------------------------------------------------------------------

#[test]
fn schubert_proven_holds_value() {
    let proven = SchubertProven::<Sigma1, i32>::new(42);
    assert_eq!(*proven.value(), 42);
}

#[test]
fn schubert_proven_check_against_compatible_type_is_nonzero() {
    let proven = SchubertProven::<Sigma1, &str>::new("data");
    // σ₁ · σ₁ in Gr(2,4) is positive-dimensional (not empty)
    let intersection = proven
        .check_against::<Sigma1>()
        .expect("σ₁ compatible with itself");
    assert_eq!(intersection.kind(), IntersectionKind::Positive);
}

#[test]
fn schubert_proven_check_against_incompatible_type_is_none() {
    let proven = SchubertProven::<Sigma22, &str>::new("data");
    // σ₂₂ · σ₂₂ in Gr(2,4): codim 4+4 = 8 > dim 4 → structural zero
    let intersection = proven.check_against::<Sigma22>();
    assert!(
        intersection.is_none(),
        "σ₂₂ should be incompatible with itself"
    );
}

#[test]
fn schubert_proven_into_inner_unwraps() {
    let proven = SchubertProven::<Sigma1, Vec<i32>>::new(vec![1, 2, 3]);
    assert_eq!(proven.into_inner(), vec![1, 2, 3]);
}

// ---------------------------------------------------------------------------
// compose_checks tests
// ---------------------------------------------------------------------------

#[test]
fn compose_checks_compatible_chain_succeeds() {
    // σ₁ · σ₁ is positive, then composing with σ₁ again
    let result = compose_checks::<Sigma1, Sigma1, Sigma1>();
    assert!(result.is_some(), "σ₁ → σ₁ → σ₁ should be composable");
}

#[test]
fn compose_checks_incompatible_chain_fails() {
    // σ₂₂ · σ₂₂ is zero, so chaining fails immediately
    let result = compose_checks::<Sigma22, Sigma22, Sigma1>();
    assert!(result.is_none(), "σ₂₂ → σ₂₂ should fail");
}
