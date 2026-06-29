// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use karpal_schubert_types::{Intersection, IntersectionKind, SchubertType, check_intersection};

// ---------------------------------------------------------------------------
// SchubertType construction
// ---------------------------------------------------------------------------

#[test]
fn schubert_type_valid_in_gr_2_4() {
    // σ₁ in Gr(2,4): partition [1], box bound is 4-2=2
    let st = SchubertType::new(vec![1], (2, 4)).expect("valid partition");
    assert_eq!(st.partition(), &[1]);
    assert_eq!(st.grassmannian_dim(), (2, 4));
    assert_eq!(st.codimension(), 1);
}

#[test]
fn schubert_type_rejects_partition_exceeding_box() {
    // Partition entry 3 exceeds n-k = 4-2 = 2
    let result = SchubertType::new(vec![3], (2, 4));
    assert!(result.is_err());
}

#[test]
fn schubert_type_empty_partition_is_point_class() {
    // Empty partition = point class, codim = dim(Gr)
    let st = SchubertType::new(vec![], (2, 4)).expect("empty partition is valid");
    assert_eq!(st.codimension(), 0);
    assert!(st.is_point_class());
}

// ---------------------------------------------------------------------------
// Intersection
// ---------------------------------------------------------------------------

#[test]
fn intersection_structural_zero_when_codim_exceeds_dim() {
    // Gr(2,4) has dim = 2*(4-2) = 4
    // σ₂₂ (partition [2,2]) has codim 4
    // Two of them have codim 8 > 4, so intersection is empty
    let sigma_22 = SchubertType::new(vec![2, 2], (2, 4)).expect("valid");
    let result = check_intersection(&sigma_22, &sigma_22);
    assert_eq!(result.kind(), IntersectionKind::StructuralZero);
    assert_eq!(result.multiplicity(), 0);
}

#[test]
fn intersection_positive_in_gr_2_4() {
    // σ₁ · σ₁ in Gr(2,4): codim 1 + 1 = 2 ≤ 4, should be positive dimensional
    let sigma_1 = SchubertType::new(vec![1], (2, 4)).expect("valid");
    let result = check_intersection(&sigma_1, &sigma_1);
    assert_eq!(result.kind(), IntersectionKind::Positive);
    assert!(result.multiplicity() > 0);
}

#[test]
fn intersection_four_lines_in_gr_2_4_is_finite() {
    // σ₁⁴ in Gr(2,4): codim 1*4 = 4 = dim(Gr) → finite intersection
    // Classic result: exactly 2 lines meet 4 general lines in P³
    use amari_enumerative::{IntersectionResult, SchubertCalculus, SchubertClass};

    let sigma_1 = SchubertClass::new(vec![1], (2, 4)).expect("valid");
    let mut calc = SchubertCalculus::new((2, 4));
    let result = calc.multi_intersect(&[
        sigma_1.clone(),
        sigma_1.clone(),
        sigma_1.clone(),
        sigma_1.clone(),
    ]);

    assert_eq!(result, IntersectionResult::Finite(2));
}

// ---------------------------------------------------------------------------
// IntersectionKind classification
// ---------------------------------------------------------------------------

#[test]
fn intersection_kind_structural_zero_is_zero() {
    assert!(IntersectionKind::StructuralZero.is_zero());
}

#[test]
fn intersection_kind_geometric_zero_is_zero() {
    assert!(IntersectionKind::GeometricZero.is_zero());
}

#[test]
fn intersection_kind_positive_is_not_zero() {
    assert!(!IntersectionKind::Positive.is_zero());
}
