// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! LR-enriched category: Schubert intersection as a category enriched
//! over the Littlewood-Richardson coefficient ring.
//!
//! This is Phase 14 sub-phase D — the formal connection between structured
//! emptiness and enriched category theory. The hom-objects carry
//! `IntersectionKind` values (the structured emptiness lattice Ω), and
//! composition uses the LR product rather than Boolean AND.

use crate::intersection::IntersectionKind;
use karpal_higher::EnrichedCategory;

/// Marker: enrichment over the LR coefficient ring.
///
/// In this enrichment, hom-objects are `IntersectionKind` values — the
/// structured emptiness lattice Ω. Composition of hom-objects uses the
/// meet of the lattice (worst-case propagation), which for intersections
/// corresponds to the LR product's zero-propagation behavior.
pub struct LRRingEnrichment;

/// A category enriched over the LR coefficient ring.
///
/// Objects are `SchubertType`s, and the hom-object between two types is
/// their `IntersectionKind` — the structured truth value of compatibility.
/// This makes `SchubertProven` a morphism in an LR-enriched category.
pub struct SchubertEnrichedCategory;

impl EnrichedCategory<LRRingEnrichment> for SchubertEnrichedCategory {
    /// The hom-object is the structured emptiness value of the intersection.
    type Hom<A, B> = IntersectionKind;

    /// Composition: intersect A with B, then B with C, and take the meet
    /// (worst-case propagation). If either link is zero, the composition
    /// is zero — but the *kind* of zero is preserved.
    ///
    /// This is the LR-product semantics: if σ_A · σ_B = 0 (structural),
    /// then σ_A · σ_B · σ_C = 0 (structural) regardless of σ_C.
    fn compose<A: 'static, B: 'static, C: 'static>(
        f: Self::Hom<A, B>,
        g: Self::Hom<B, C>,
    ) -> Self::Hom<A, C> {
        meet_intersection_kinds(f, g)
    }

    /// Identity: the point class σ_∅ intersects everything positively.
    /// In the structured emptiness lattice, the identity hom-object is
    /// `Positive` (the unit of the LR ring).
    fn id<A: 'static>() -> Self::Hom<A, A> {
        IntersectionKind::Positive
    }
}

/// Compute the meet (greatest lower bound) of two IntersectionKind values
/// in the structured emptiness lattice.
///
/// The lattice order from bottom to top:
/// ```text
/// StructuralZero < GeometricZero < Positive < Underdetermined
/// ```
///
/// The meet takes the "worst case" — if either side is a zero, the result
/// is that zero. If both are positive, the result is positive. If either
/// is underdetermined, the result is the other (underdetermined is "top"
/// in the sense that it imposes no constraint).
pub fn meet_intersection_kinds(a: IntersectionKind, b: IntersectionKind) -> IntersectionKind {
    use IntersectionKind::*;
    match (a, b) {
        // Structural zero dominates everything (bottom of lattice)
        (StructuralZero, _) | (_, StructuralZero) => StructuralZero,
        // Geometric zero dominates everything except structural
        (GeometricZero, _) | (_, GeometricZero) => GeometricZero,
        // Both positive → positive
        (Positive, Positive) => Positive,
        // Positive with underdetermined → positive (the more specific result wins)
        (Positive, Underdetermined) | (Underdetermined, Positive) => Positive,
        // Both underdetermined → underdetermined
        (Underdetermined, Underdetermined) => Underdetermined,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intersection::check_intersection;
    use crate::schubert_type::SchubertType;

    #[test]
    fn lr_identity_is_positive() {
        let id = SchubertEnrichedCategory::id::<()>();
        assert_eq!(id, IntersectionKind::Positive);
    }

    #[test]
    fn lr_compose_positive_positive_is_positive() {
        let result = SchubertEnrichedCategory::compose::<(), (), ()>(
            IntersectionKind::Positive,
            IntersectionKind::Positive,
        );
        assert_eq!(result, IntersectionKind::Positive);
    }

    #[test]
    fn lr_compose_structural_zero_dominates() {
        let result = SchubertEnrichedCategory::compose::<(), (), ()>(
            IntersectionKind::StructuralZero,
            IntersectionKind::Positive,
        );
        assert_eq!(result, IntersectionKind::StructuralZero);

        let result2 = SchubertEnrichedCategory::compose::<(), (), ()>(
            IntersectionKind::Positive,
            IntersectionKind::StructuralZero,
        );
        assert_eq!(result2, IntersectionKind::StructuralZero);
    }

    #[test]
    fn lr_compose_geometric_zero_propagates() {
        let result = SchubertEnrichedCategory::compose::<(), (), ()>(
            IntersectionKind::GeometricZero,
            IntersectionKind::Positive,
        );
        assert_eq!(result, IntersectionKind::GeometricZero);
    }

    #[test]
    fn lr_compose_positive_underdetermined_gives_positive() {
        let result = SchubertEnrichedCategory::compose::<(), (), ()>(
            IntersectionKind::Positive,
            IntersectionKind::Underdetermined,
        );
        assert_eq!(result, IntersectionKind::Positive);
    }

    #[test]
    fn lr_compose_underdetermined_underdetermined_is_underdetermined() {
        let result = SchubertEnrichedCategory::compose::<(), (), ()>(
            IntersectionKind::Underdetermined,
            IntersectionKind::Underdetermined,
        );
        assert_eq!(result, IntersectionKind::Underdetermined);
    }

    #[test]
    fn meet_lattice_order_is_correct() {
        // StructuralZero is bottom
        assert_eq!(
            meet_intersection_kinds(IntersectionKind::StructuralZero, IntersectionKind::Positive),
            IntersectionKind::StructuralZero
        );
        // GeometricZero > StructuralZero
        assert_eq!(
            meet_intersection_kinds(
                IntersectionKind::GeometricZero,
                IntersectionKind::StructuralZero
            ),
            IntersectionKind::StructuralZero
        );
        // Positive > GeometricZero
        assert_eq!(
            meet_intersection_kinds(IntersectionKind::Positive, IntersectionKind::GeometricZero),
            IntersectionKind::GeometricZero
        );
    }

    #[test]
    fn real_schubert_intersection_composes_correctly() {
        // σ₁ in Gr(2,4): Positive intersection with itself
        let s1_s1 = check_intersection(
            &SchubertType::new(vec![1], (2, 4)).unwrap(),
            &SchubertType::new(vec![1], (2, 4)).unwrap(),
        );
        assert_eq!(s1_s1.kind(), IntersectionKind::Positive);

        // The composed hom-object preserves positivity
        let composed = SchubertEnrichedCategory::compose::<(), (), ()>(s1_s1.kind(), s1_s1.kind());
        assert_eq!(composed, IntersectionKind::Positive);

        // σ₂₂ · σ₂₂ is structural zero — composition propagates it
        let s22_s22 = check_intersection(
            &SchubertType::new(vec![2, 2], (2, 4)).unwrap(),
            &SchubertType::new(vec![2, 2], (2, 4)).unwrap(),
        );
        assert_eq!(s22_s22.kind(), IntersectionKind::StructuralZero);

        let composed_zero = SchubertEnrichedCategory::compose::<(), (), ()>(
            s22_s22.kind(),
            IntersectionKind::Positive,
        );
        assert_eq!(composed_zero, IntersectionKind::StructuralZero);
    }
}
