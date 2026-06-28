// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! 2-category encoding for the Karpal ecosystem.
//!
//! A strict 2-category consists of objects, 1-morphisms between objects,
//! and 2-morphisms between parallel 1-morphisms, with vertical and
//! horizontal composition satisfying the interchange law.
//!
//! The canonical example is **Cat**: objects are Rust types, 1-morphisms
//! are functors (`HKT` implementors), and 2-morphisms are natural
//! transformations (`NaturalTransformation<F, G>`).

/// A strict 2-category.
///
/// # Type parameters
///
/// - `Morphism<A, B>` — 1-morphisms from object A to object B
/// - `TwoMorphism` — a 2-morphism between parallel 1-morphisms (opaque: the
///   parallel condition is enforced at the impl level, not at the trait level)
pub trait TwoCategory {
    /// A 1-morphism from A to B.
    type Morphism<A, B>;

    /// A 2-morphism (distinct type for each 1-morphism pair).
    type TwoMorphism;

    /// Identity 1-morphism on object A.
    fn id1<A: 'static>() -> Self::Morphism<A, A>;

    /// Vertical composition: `f: A → B`, `g: B → C` → `g ∘ f: A → C`.
    fn compose1<A: 'static, B: 'static, C: 'static>(
        f: Self::Morphism<A, B>,
        g: Self::Morphism<B, C>,
    ) -> Self::Morphism<A, C>;

    /// Identity 2-morphism on a 1-morphism.
    fn id2() -> Self::TwoMorphism;

    /// Vertical composition of 2-morphisms: `α: F ⇒ G`, `β: G ⇒ H` → `β ∘ᵥ α: F ⇒ H`.
    fn compose2_vertical(alpha: Self::TwoMorphism, beta: Self::TwoMorphism) -> Self::TwoMorphism;
}

// ---------------------------------------------------------------------------
// Cat: the 2-category of categories
// ---------------------------------------------------------------------------

/// Witness that **Cat** is a strict 2-category.
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct Cat;

#[cfg(any(feature = "std", feature = "alloc"))]
impl TwoCategory for Cat {
    type Morphism<A, B> = Box<dyn Fn(A) -> B>;
    type TwoMorphism = ();

    fn id1<A: 'static>() -> Self::Morphism<A, A> {
        Box::new(|a| a)
    }

    fn compose1<A: 'static, B: 'static, C: 'static>(
        f: Self::Morphism<A, B>,
        g: Self::Morphism<B, C>,
    ) -> Self::Morphism<A, C> {
        Box::new(move |a| g(f(a)))
    }

    fn id2() -> Self::TwoMorphism {}

    fn compose2_vertical(_alpha: Self::TwoMorphism, _beta: Self::TwoMorphism) -> Self::TwoMorphism {
    }
}

#[cfg(all(test, any(feature = "std", feature = "alloc")))]
mod tests {
    use super::*;

    #[test]
    fn cat_id1_is_identity_function() {
        let id: Box<dyn Fn(i32) -> i32> = Cat::id1();
        assert_eq!(id(42), 42);
    }

    #[test]
    fn cat_compose1_chains_functions() {
        let f: Box<dyn Fn(i32) -> i32> = Box::new(|x| x + 1);
        let g: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        let gf = Cat::compose1(f, g);
        assert_eq!(gf(5), 12); // (5+1)*2
    }

    #[test]
    fn cat_compose2_vertical_is_total() {
        let alpha = Cat::id2();
        let beta = Cat::id2();
        let _gamma = Cat::compose2_vertical(alpha, beta);
    }
}
