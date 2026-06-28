// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use amari_enumerative::{IntersectionResult, SchubertCalculus};

use crate::schubert_type::SchubertType;

/// Result of intersecting two Schubert types.
#[derive(Debug, Clone)]
pub struct Intersection {
    kind: IntersectionKind,
    multiplicity: u64,
    /// When the result is positive-dimensional, this carries the
    /// decomposition into Schubert classes if computable.
    decomposition: Vec<SchubertType>,
}

impl Intersection {
    fn structural_zero() -> Self {
        Self {
            kind: IntersectionKind::StructuralZero,
            multiplicity: 0,
            decomposition: Vec::new(),
        }
    }

    fn geometric_zero() -> Self {
        Self {
            kind: IntersectionKind::GeometricZero,
            multiplicity: 0,
            decomposition: Vec::new(),
        }
    }

    fn positive(multiplicity: u64, decomposition: Vec<SchubertType>) -> Self {
        Self {
            kind: IntersectionKind::Positive,
            multiplicity,
            decomposition,
        }
    }

    /// The classification of this intersection.
    pub fn kind(&self) -> IntersectionKind {
        self.kind
    }

    /// The intersection multiplicity (0 for zeros and underdetermined).
    pub fn multiplicity(&self) -> u64 {
        self.multiplicity
    }

    /// Decomposition into Schubert classes, if available.
    pub fn decomposition(&self) -> &[SchubertType] {
        &self.decomposition
    }

    /// When the intersection is positive, attempt to extract the
    /// resulting Schubert type (the first one in the decomposition).
    pub fn into_schubert(self) -> Option<SchubertType> {
        if self.kind == IntersectionKind::Positive && !self.decomposition.is_empty() {
            Some(self.decomposition.into_iter().next().unwrap())
        } else {
            None
        }
    }
}

/// Classification of an intersection result.
///
/// - `StructuralZero`: total codimension exceeds Grassmannian dimension
/// - `GeometricZero`: correctly dimensioned but no intersection points
/// - `Positive`: nonempty intersection with known multiplicity
/// - `Underdetermined`: the computation could not resolve the result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntersectionKind {
    StructuralZero,
    GeometricZero,
    Positive,
    Underdetermined,
}

impl IntersectionKind {
    /// True for either kind of empty intersection.
    pub fn is_zero(&self) -> bool {
        matches!(self, Self::StructuralZero | Self::GeometricZero)
    }
}

/// Compute the intersection of two Schubert types.
///
/// Uses `amari-enumerative`'s Schubert calculus engine to compute the
/// intersection product and classify the result.
pub fn check_intersection(a: &SchubertType, b: &SchubertType) -> Intersection {
    let mut calc = SchubertCalculus::new(a.grassmannian_dim());

    // Use multi_intersect for the two classes
    let classes = [a.as_inner().clone(), b.as_inner().clone()];
    let result = calc.multi_intersect(&classes);

    match result {
        IntersectionResult::Empty => {
            // Distinguish structural vs geometric zero
            let total_codim = a.codimension() + b.codimension();
            let dim = calc.grassmannian_dimension();
            if total_codim > dim {
                Intersection::structural_zero()
            } else {
                Intersection::geometric_zero()
            }
        }
        IntersectionResult::Finite(n) => Intersection::positive(n, Vec::new()),
        IntersectionResult::PositiveDimensional {
            dimension: _,
            degree,
        } => {
            // Positive-dimensional intersection is always non-empty
            let multiplicity = degree.unwrap_or(1);
            Intersection::positive(multiplicity, Vec::new())
        }
    }
}
