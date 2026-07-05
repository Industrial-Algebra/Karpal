// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! RichCat: a 2-category where 2-morphisms carry computational content.
//!
//! Unlike [`Cat`](crate::two_category::Cat) where `TwoMorphism = ()`,
//! RichCat's 2-morphisms are `TwoCell`s that carry provenance data —
//! a description of what the 2-cell witnesses.
//!
//! This answers the question from the Rabbit Hole analysis: "when do
//! 2-morphisms stop being `()`?" RichCat is the first step — 2-morphisms
//! carry labels that track their origin and composition history. Future
//! enrichments can add actual witness functions, test results, or proof
//! terms to the `TwoCell` structure.

use crate::bicategory::Bicategory;
use crate::two_category::TwoCategory;

// ---------------------------------------------------------------------------
// TwoCell: the non-trivial 2-morphism type
// ---------------------------------------------------------------------------

/// A 2-morphism in `RichCat` between two parallel 1-morphisms.
///
/// Unlike `Cat`'s `TwoMorphism = ()`, a `TwoCell` carries:
/// - A `label` identifying what the 2-cell witnesses (e.g. "naturality",
///   "associativity", "identity")
/// - A composition history (vertical composition concatenates labels)
///
/// This gives `RichCat` genuine computational content at the 2-morphism
/// level, while remaining extensible — future work can add witness
/// functions, proof terms, or test evidence to the `TwoCell` struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TwoCell {
    label: String,
}

impl TwoCell {
    /// Create a named 2-cell.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }

    /// The label identifying what this 2-cell witnesses.
    pub fn label(&self) -> &str {
        &self.label
    }
}

// ---------------------------------------------------------------------------
// RichCat: the 2-category with contentful 2-morphisms
// ---------------------------------------------------------------------------

/// Witness that **RichCat** is a strict 2-category with contentful 2-morphisms.
///
/// RichCat is the same underlying category as [`Cat`](crate::two_category::Cat)
/// — objects are Rust types, 1-morphisms are functions — but its 2-morphisms
/// are `TwoCell`s that carry provenance instead of being vacuous `()`.
///
/// This makes RichCat suitable for applications that need to *track* what
/// 2-categorical operations were performed: verification pipelines, audit
/// trails, proof reconstruction, and AI agent reasoning traces.
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct RichCat;

#[cfg(any(feature = "std", feature = "alloc"))]
impl TwoCategory for RichCat {
    type Morphism<A, B> = Box<dyn Fn(A) -> B>;
    type TwoMorphism = TwoCell;

    fn id1<A: 'static>() -> Self::Morphism<A, A> {
        Box::new(|a| a)
    }

    fn compose1<A: 'static, B: 'static, C: 'static>(
        f: Self::Morphism<A, B>,
        g: Self::Morphism<B, C>,
    ) -> Self::Morphism<A, C> {
        Box::new(move |a| g(f(a)))
    }

    fn id2() -> Self::TwoMorphism {
        TwoCell::new("id")
    }

    fn compose2_vertical(alpha: Self::TwoMorphism, beta: Self::TwoMorphism) -> Self::TwoMorphism {
        TwoCell::new(format!("{};{}", alpha.label, beta.label))
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Bicategory for RichCat {
    fn associator<A: 'static, B: 'static, C: 'static, D: 'static>() -> Self::TwoMorphism {
        TwoCell::new("associator")
    }

    fn left_unitor<A: 'static, B: 'static>() -> Self::TwoMorphism {
        TwoCell::new("left_unitor")
    }

    fn right_unitor<A: 'static, B: 'static>() -> Self::TwoMorphism {
        TwoCell::new("right_unitor")
    }
}

#[cfg(all(test, any(feature = "std", feature = "alloc")))]
mod tests {
    use super::*;

    #[test]
    fn rich_cat_id1_is_identity_function() {
        let id: Box<dyn Fn(i32) -> i32> = RichCat::id1();
        assert_eq!(id(42), 42);
    }

    #[test]
    fn rich_cat_compose1_chains_functions() {
        let f: Box<dyn Fn(i32) -> i32> = Box::new(|x| x + 1);
        let g: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        let gf = RichCat::compose1(f, g);
        assert_eq!(gf(5), 12);
    }

    #[test]
    fn rich_cat_id2_is_named() {
        let id = RichCat::id2();
        assert_eq!(id.label(), "id");
    }

    #[test]
    fn rich_cat_compose2_concatenates_labels() {
        let alpha = TwoCell::new("naturality");
        let beta = TwoCell::new("associativity");
        let gamma = RichCat::compose2_vertical(alpha, beta);
        assert_eq!(gamma.label(), "naturality;associativity");
    }

    #[test]
    fn rich_cat_associator_is_named() {
        let a = RichCat::associator::<i32, &str, bool, f64>();
        assert_eq!(a.label(), "associator");
    }

    #[test]
    fn rich_cat_unitors_are_named() {
        let l = RichCat::left_unitor::<i32, &str>();
        let r = RichCat::right_unitor::<i32, &str>();
        assert_eq!(l.label(), "left_unitor");
        assert_eq!(r.label(), "right_unitor");
    }

    #[test]
    fn two_cell_equality() {
        assert_eq!(TwoCell::new("foo"), TwoCell::new("foo"));
        assert_ne!(TwoCell::new("foo"), TwoCell::new("bar"));
    }

    #[test]
    fn rich_cat_vertical_composition_preserves_history() {
        let a = RichCat::id2();
        let b = TwoCell::new("naturality");
        let c = RichCat::compose2_vertical(a, b);
        assert_eq!(c.label(), "id;naturality");

        // Chain further
        let d = RichCat::associator::<i32, i32, i32, i32>();
        let e = RichCat::compose2_vertical(c, d);
        assert_eq!(e.label(), "id;naturality;associator");
    }
}
