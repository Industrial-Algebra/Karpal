// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Higher functors and monads at the 2-categorical level.
//!
//! - `FFunctor`: a functor between 2-categories that preserves 2-morphisms
//! - `FMonad`: a monad in the endofunctor 2-category `Endo(C)`

use crate::two_category::TwoCategory;

/// A functor between two 2-categories that preserves the 2-categorical structure.
///
/// `FFunctor` maps objects, 1-morphisms, and 2-morphisms from `C1` to `C2`
/// while preserving identity and composition.
pub trait FFunctor<C1: TwoCategory, C2: TwoCategory> {
    /// Map a 1-morphism from C1 to C2.
    fn map_morphism<A: 'static, B: 'static>(f: C1::Morphism<A, B>) -> C2::Morphism<A, B>;

    /// Map a 2-morphism from C1 to C2.
    fn map_two_morphism(alpha: C1::TwoMorphism) -> C2::TwoMorphism;
}

// ---------------------------------------------------------------------------
// Identity FFunctor
// ---------------------------------------------------------------------------

/// The identity functor on a 2-category C.
pub struct IdentityFFunctor<C: TwoCategory>(core::marker::PhantomData<C>);

impl<C: TwoCategory> FFunctor<C, C> for IdentityFFunctor<C> {
    fn map_morphism<A: 'static, B: 'static>(f: C::Morphism<A, B>) -> C::Morphism<A, B> {
        f
    }

    fn map_two_morphism(alpha: C::TwoMorphism) -> C::TwoMorphism {
        alpha
    }
}

// ---------------------------------------------------------------------------
// FMonad
// ---------------------------------------------------------------------------

/// A monad in the 2-category of endofunctors `Endo(C)`.
///
/// An `FMonad` on a 2-category `C` is an endofunctor `T` together with
/// 2-morphisms `η: Id ⇒ T` (unit) and `μ: T ∘ T ⇒ T` (multiplication)
/// satisfying the monad laws.
pub trait FMonad<C: TwoCategory>: FFunctor<C, C> {
    /// Unit: `η: Id ⇒ T`
    fn unit<A: 'static>() -> C::TwoMorphism;

    /// Multiplication: `μ: T ∘ T ⇒ T`
    fn multiply<A: 'static>() -> C::TwoMorphism;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_ffunctor_preserves_morphism() {
        struct Marker;
        impl TwoCategory for Marker {
            type Morphism<A, B> = ();
            type TwoMorphism = ();
            fn id1<A: 'static>() -> Self::Morphism<A, A> {}
            fn compose1<A: 'static, B: 'static, C: 'static>(
                _: Self::Morphism<A, B>,
                _: Self::Morphism<B, C>,
            ) -> Self::Morphism<A, C> {
            }
            fn id2() -> Self::TwoMorphism {}
            fn compose2_vertical(_: Self::TwoMorphism, _: Self::TwoMorphism) -> Self::TwoMorphism {}
        }
        let _unit: () = IdentityFFunctor::<Marker>::map_morphism::<(), ()>(());
    }

    #[test]
    fn id_ffunctor_preserves_two_morphism() {
        struct Marker;
        impl TwoCategory for Marker {
            type Morphism<A, B> = ();
            type TwoMorphism = ();
            fn id1<A: 'static>() -> Self::Morphism<A, A> {}
            fn compose1<A: 'static, B: 'static, C: 'static>(
                _: Self::Morphism<A, B>,
                _: Self::Morphism<B, C>,
            ) -> Self::Morphism<A, C> {
            }
            fn id2() -> Self::TwoMorphism {}
            fn compose2_vertical(_: Self::TwoMorphism, _: Self::TwoMorphism) -> Self::TwoMorphism {}
        }
        let _unit: () = IdentityFFunctor::<Marker>::map_two_morphism(());
    }
}
