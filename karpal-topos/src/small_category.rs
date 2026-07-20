// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Small categories encoded at the type level.
//!
//! Objects are phantom marker types; morphisms `Mor<A, B>` are values
//! carrying runtime data. This module provides the abstract
//! [`SmallCategory`] trait plus two concrete instances: [`ChainCat`] (a
//! finite total order) and [`DiscreteCat`] (only identities).
//!
//! # Why morphisms-as-data, not morphisms-as-function
//!
//! [`karpal_arrow::Category`](../../karpal_arrow/index.html) is biased toward
//! *computable* morphisms (`compose`/`id` return function-like values).
//! Presheaves are defined over arbitrary small categories, where morphisms
//! are often finite data (the simplex category Δ, poset categories). This
//! module's `SmallCategory` is deliberately separate: morphisms are indexing
//! data, and identity is constructed per-concrete-category.
//!
//! # Identity is not a trait method
//!
//! Rust cannot extract object identity from phantom type parameters, so
//! [`SmallCategory`] provides only `compose`. Each concrete category
//! supplies `identity` as an inherent method bound to an object-index trait
//! (e.g. [`ChainObj`]). This is an honest limitation, not an omission.

use core::marker::PhantomData;

/// A small category: objects are phantom types, morphisms `Mor<A, B>` are values.
///
/// Laws (verified per concrete category):
/// - Associativity: `compose(h, compose(g, f)) == compose(compose(h, g), f)`
/// - (Identity is provided by concrete categories via inherent methods.)
pub trait SmallCategory {
    /// The type of morphisms from `A` to `B`.
    type Mor<A, B>;

    /// Compose `g: B → C` after `f: A → B`, yielding `g ∘ f: A → C`.
    fn compose<A, B, C>(g: Self::Mor<B, C>, f: Self::Mor<A, B>) -> Self::Mor<A, C>;
}

/// Object-index trait for [`ChainCat`]: each object marker exposes its
/// position as a compile-time `IDX`.
pub trait ChainObj {
    const IDX: usize;
}

/// The poset category of a finite chain `0 ≤ 1 ≤ … ≤ N`.
///
/// A morphism `i → j` exists iff `i ≤ j` (unique witness). This is the
/// simplest non-trivial small category and is used throughout the crate's
/// tests.
pub struct ChainCat<const N: usize>;

/// A morphism `A → B` in [`ChainCat`], carrying the runtime source/target
/// indices derived from the object markers.
///
/// Implements `Copy` unconditionally because it holds only `usize` indices
/// and `PhantomData` (the object markers need not be `Copy`).
pub struct ChainMor<A, B> {
    pub(crate) from: usize,
    pub(crate) to: usize,
    _a: PhantomData<fn() -> A>,
    _b: PhantomData<fn() -> B>,
}

impl<A, B> Clone for ChainMor<A, B> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<A, B> Copy for ChainMor<A, B> {}

impl<A, B> ChainMor<A, B> {
    /// The source object index.
    pub fn from(&self) -> usize {
        self.from
    }

    /// The target object index.
    pub fn to(&self) -> usize {
        self.to
    }
}

impl<const N: usize> ChainCat<N> {
    /// The identity morphism on object `A`. Requires `A: ChainObj`.
    pub fn identity<A: ChainObj>() -> ChainMor<A, A> {
        ChainMor {
            from: A::IDX,
            to: A::IDX,
            _a: PhantomData,
            _b: PhantomData,
        }
    }

    /// Construct a morphism `A → B` witnessing `A::IDX ≤ B::IDX`.
    ///
    /// Returns `None` if `A::IDX > B::IDX` (no such morphism in the chain)
    /// or if either index is out of range `[0, N]`.
    pub fn morphism<A: ChainObj, B: ChainObj>() -> Option<ChainMor<A, B>> {
        let (i, j) = (A::IDX, B::IDX);
        if i <= j && i <= N && j <= N {
            Some(ChainMor {
                from: i,
                to: j,
                _a: PhantomData,
                _b: PhantomData,
            })
        } else {
            None
        }
    }
}

impl<const N: usize> SmallCategory for ChainCat<N> {
    type Mor<A, B> = ChainMor<A, B>;

    fn compose<A, B, C>(g: ChainMor<B, C>, f: ChainMor<A, B>) -> ChainMor<A, C> {
        debug_assert!(
            f.to == g.from,
            "ChainCat::compose: codomain of f ({}) must equal domain of g ({})",
            f.to,
            g.from
        );
        ChainMor {
            from: f.from,
            to: g.to,
            _a: PhantomData,
            _b: PhantomData,
        }
    }
}

/// A discrete category: objects only, with identity morphisms and no
/// non-identity morphisms.
pub struct DiscreteCat;

/// A morphism in [`DiscreteCat`]: only identities exist.
///
/// Implements `Copy` unconditionally (holds only `PhantomData`).
pub struct DiscreteMor<A, B> {
    _a: PhantomData<fn() -> A>,
    _b: PhantomData<fn() -> B>,
}

impl<A, B> Clone for DiscreteMor<A, B> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<A, B> Copy for DiscreteMor<A, B> {}

impl DiscreteCat {
    /// The identity morphism on `A`. The only morphism in a discrete category.
    pub fn identity<A>() -> DiscreteMor<A, A> {
        DiscreteMor {
            _a: PhantomData,
            _b: PhantomData,
        }
    }
}

impl SmallCategory for DiscreteCat {
    type Mor<A, B> = DiscreteMor<A, B>;

    fn compose<A, B, C>(_g: DiscreteMor<B, C>, _f: DiscreteMor<A, B>) -> DiscreteMor<A, C> {
        // Only identities exist; composing identities yields the identity.
        DiscreteMor {
            _a: PhantomData,
            _b: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;

    // Three object markers for a chain of length 2: 0 ≤ 1 ≤ 2.
    struct C0;
    struct C1;
    struct C2;

    impl ChainObj for C0 {
        const IDX: usize = 0;
    }
    impl ChainObj for C1 {
        const IDX: usize = 1;
    }
    impl ChainObj for C2 {
        const IDX: usize = 2;
    }

    #[test]
    fn chain_identity_roundtrips() {
        let id: ChainMor<C1, C1> = ChainCat::<2>::identity::<C1>();
        assert_eq!(id.from(), 1);
        assert_eq!(id.to(), 1);
    }

    #[test]
    fn chain_morphism_exists_when_ordered() {
        let f: ChainMor<C0, C2> = ChainCat::<2>::morphism::<C0, C2>().unwrap();
        assert_eq!(f.from(), 0);
        assert_eq!(f.to(), 2);
    }

    #[test]
    fn chain_morphism_absent_when_unordered() {
        // C2 → C0 does not exist (2 > 0).
        assert!(ChainCat::<2>::morphism::<C2, C0>().is_none());
    }

    #[test]
    fn chain_compose_associativity() {
        // f: C0→C1, g: C1→C2, h: C2→C2
        let f = ChainCat::<2>::morphism::<C0, C1>().unwrap();
        let g = ChainCat::<2>::morphism::<C1, C2>().unwrap();
        let h = ChainCat::<2>::identity::<C2>();

        let left = ChainCat::<2>::compose(h, ChainCat::<2>::compose(g, f));
        let right = ChainCat::<2>::compose(ChainCat::<2>::compose(h, g), f);

        assert_eq!(left.from(), right.from());
        assert_eq!(left.to(), right.to());
        assert_eq!((left.from(), left.to()), (0, 2));
    }

    #[test]
    fn chain_compose_left_identity() {
        let f = ChainCat::<2>::morphism::<C0, C2>().unwrap();
        let id = ChainCat::<2>::identity::<C2>();
        let composed: ChainMor<C0, C2> = ChainCat::<2>::compose(id, f);
        assert_eq!((composed.from(), composed.to()), (0, 2));
    }

    #[test]
    fn chain_compose_right_identity() {
        let f = ChainCat::<2>::morphism::<C0, C2>().unwrap();
        let id = ChainCat::<2>::identity::<C0>();
        let composed: ChainMor<C0, C2> = ChainCat::<2>::compose(f, id);
        assert_eq!((composed.from(), composed.to()), (0, 2));
    }

    #[test]
    fn discrete_identity_and_compose() {
        let id: DiscreteMor<C0, C0> = DiscreteCat::identity::<C0>();
        let _composed: DiscreteMor<C0, C0> =
            DiscreteCat::compose(id, DiscreteCat::identity::<C0>());
    }
}
