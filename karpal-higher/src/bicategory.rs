// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Bicategory trait: a weakened 2-category where composition is associative
//! and unital only up to isomorphism.
//!
//! Extends `TwoCategory` with:
//! - An associator `α_{f,g,h}: (f ∘ g) ∘ h ≅ f ∘ (g ∘ h)`
//! - Left unitor `λ_f: id ∘ f ≅ f`
//! - Right unitor `ρ_f: f ∘ id ≅ f`
//!
//! These must satisfy the pentagon and triangle coherence laws.

use crate::two_category::TwoCategory;

/// A bicategory: a 2-category with associator and unitors as isomorphisms.
///
/// The associator and unitors are provided as type-level constructors
/// (no runtime morphism values) to avoid ownership issues. They produce
/// `TwoMorphism` witnesses that the coherence isomorphisms exist.
pub trait Bicategory: TwoCategory {
    /// Associator: `(f ∘ g) ∘ h ≅ f ∘ (g ∘ h)`
    fn associator<A: 'static, B: 'static, C: 'static, D: 'static>() -> Self::TwoMorphism;

    /// Left unitor: `id_A ∘ f ≅ f` for `f: A → B`
    fn left_unitor<A: 'static, B: 'static>() -> Self::TwoMorphism;

    /// Right unitor: `f ∘ id_B ≅ f` for `f: A → B`
    fn right_unitor<A: 'static, B: 'static>() -> Self::TwoMorphism;
}

// ---------------------------------------------------------------------------
// Cat: bicategory instance
// ---------------------------------------------------------------------------

#[cfg(any(feature = "std", feature = "alloc"))]
impl Bicategory for crate::two_category::Cat {
    fn associator<A: 'static, B: 'static, C: 'static, D: 'static>() -> Self::TwoMorphism {}

    fn left_unitor<A: 'static, B: 'static>() -> Self::TwoMorphism {}

    fn right_unitor<A: 'static, B: 'static>() -> Self::TwoMorphism {}
}

#[cfg(all(test, any(feature = "std", feature = "alloc")))]
mod tests {
    use super::*;
    use crate::two_category::Cat;

    #[test]
    fn cat_associator_exists() {
        Cat::associator::<i32, &str, bool, f64>();
    }

    #[test]
    fn cat_unit_existence() {
        Cat::left_unitor::<i32, &str>();
        Cat::right_unitor::<i32, &str>();
    }
}
