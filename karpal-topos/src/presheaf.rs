// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Presheaves: contravariant functors `C^op → Set`.

use core::marker::PhantomData;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;

use crate::small_category::SmallCategory;

#[cfg(any(feature = "std", feature = "alloc"))]
use crate::small_category::{ChainCat, ChainMor};

/// A presheaf on `C`: a contravariant functor `C^op → Set`.
///
/// For each object `c` it assigns a set [`Self::At<Obj>`]; for each morphism
/// `f: Dom → Cod` in `C` it assigns a restriction map
/// `restrict(f): P(Cod) → P(Dom)`.
///
/// # Laws
///
/// - Identity: `restrict(id, x) == x`
/// - Composition: `restrict(g ∘ f, x) == restrict(f, restrict(g, x))`
///
/// Note the contravariance: restriction along `f: Dom → Cod` maps values at
/// `Cod` to values at `Dom`, and composition order reverses.
pub trait Presheaf<C: SmallCategory> {
    /// The set `P(Obj)`: the value of the presheaf at object `Obj`.
    type At<Obj>;

    /// Restriction along `f: Dom → Cod`. Maps `P(Cod) → P(Dom)`.
    fn restrict<Dom, Cod>(f: C::Mor<Dom, Cod>, x: Self::At<Cod>) -> Self::At<Dom>;
}

/// The constant presheaf: `P(c) = T` for every object, restriction is identity.
///
/// This is the simplest non-trivial presheaf and is useful for testing the
/// functor laws trivially.
pub struct ConstantPresheaf<T>(PhantomData<fn() -> T>);

impl<C: SmallCategory, T> Presheaf<C> for ConstantPresheaf<T> {
    type At<Obj> = T;

    fn restrict<Dom, Cod>(_f: C::Mor<Dom, Cod>, x: T) -> T {
        x
    }
}

/// A presheaf over [`ChainCat`] that assigns the initial segment
/// `P(i) = {0, 1, …, i}` to object `i`. Restriction `P(j) → P(i)` (for
/// `i ≤ j`) truncates the segment to the first `i + 1` elements.
///
/// This exercises non-trivial restriction and is the workhorse for the
/// presheaf-law and Yoneda tests.
pub struct InitialSegmentPresheaf;

/// The value `P(i)`: a vector `0..=i` represented compactly by its length.
#[cfg(any(feature = "std", feature = "alloc"))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SegmentSet {
    pub len: usize,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl SegmentSet {
    /// Reconstruct the segment as a `Vec<u32>` = `{0, 1, …, len-1}`.
    pub fn to_vec(&self) -> Vec<u32> {
        (0..self.len as u32).collect()
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<const N: usize> Presheaf<ChainCat<N>> for InitialSegmentPresheaf {
    type At<Obj> = SegmentSet;

    fn restrict<Dom, Cod>(f: ChainMor<Dom, Cod>, x: SegmentSet) -> SegmentSet {
        // f: Dom → Cod, so Dom::IDX ≤ Cod::IDX. Restriction truncates P(Cod)
        // down to P(Dom): keep the first (Dom::IDX + 1) elements.
        let target_len = f.from() + 1;
        SegmentSet {
            len: x.len.min(target_len),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;
    use crate::small_category::ChainObj;

    // Object markers for a chain of length 3: 0 ≤ 1 ≤ 2 ≤ 3.
    struct C0;
    struct C1;
    struct C2;
    struct C3;

    impl ChainObj for C0 {
        const IDX: usize = 0;
    }
    impl ChainObj for C1 {
        const IDX: usize = 1;
    }
    impl ChainObj for C2 {
        const IDX: usize = 2;
    }
    impl ChainObj for C3 {
        const IDX: usize = 3;
    }

    #[test]
    fn constant_presheaf_identity() {
        let id = ChainCat::<3>::identity::<C2>();
        let x: i32 = 42;
        let result: i32 = <ConstantPresheaf<i32> as Presheaf<ChainCat<3>>>::restrict(id, x);
        assert_eq!(result, 42);
    }

    #[test]
    fn constant_presheaf_composition() {
        let f = ChainCat::<3>::morphism::<C0, C1>().unwrap();
        let g = ChainCat::<3>::morphism::<C1, C3>().unwrap();
        let x: i32 = 7;

        let left: i32 = <ConstantPresheaf<i32> as Presheaf<ChainCat<3>>>::restrict(
            ChainCat::<3>::compose(g, f),
            x,
        );
        let inner: i32 = <ConstantPresheaf<i32> as Presheaf<ChainCat<3>>>::restrict(g, x);
        let right: i32 = <ConstantPresheaf<i32> as Presheaf<ChainCat<3>>>::restrict(f, inner);

        assert_eq!(left, right);
        assert_eq!(left, 7);
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn initial_segment_value() {
        // P(C2) = {0, 1, 2}
        let at_c2: <InitialSegmentPresheaf as Presheaf<ChainCat<3>>>::At<C2> =
            SegmentSet { len: 3 };
        assert_eq!(at_c2.to_vec(), vec![0, 1, 2]);
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn initial_segment_restrict_truncates() {
        // f: C1 → C3, so P(C3) → P(C1) truncates to len 2.
        let f = ChainCat::<3>::morphism::<C1, C3>().unwrap();
        let at_c3 = SegmentSet { len: 4 }; // P(C3) = {0,1,2,3}
        let restricted: SegmentSet =
            <InitialSegmentPresheaf as Presheaf<ChainCat<3>>>::restrict(f, at_c3);
        assert_eq!(restricted.to_vec(), vec![0, 1]);
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn initial_segment_identity_law() {
        let id = ChainCat::<3>::identity::<C2>();
        let at_c2 = SegmentSet { len: 3 };
        let result: SegmentSet =
            <InitialSegmentPresheaf as Presheaf<ChainCat<3>>>::restrict(id, at_c2);
        assert_eq!(result.to_vec(), vec![0, 1, 2]);
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn initial_segment_composition_law() {
        // f: C0 → C1, g: C1 → C2. restrict(g∘f, x) == restrict(f, restrict(g, x)).
        let f = ChainCat::<3>::morphism::<C0, C1>().unwrap();
        let g = ChainCat::<3>::morphism::<C1, C2>().unwrap();
        let at_c2 = SegmentSet { len: 3 }; // P(C2) = {0,1,2}

        let left: SegmentSet = <InitialSegmentPresheaf as Presheaf<ChainCat<3>>>::restrict(
            ChainCat::<3>::compose(g, f),
            at_c2.clone(),
        );
        let inner: SegmentSet =
            <InitialSegmentPresheaf as Presheaf<ChainCat<3>>>::restrict(g, at_c2);
        let right: SegmentSet =
            <InitialSegmentPresheaf as Presheaf<ChainCat<3>>>::restrict(f, inner);

        assert_eq!(left.to_vec(), right.to_vec());
        assert_eq!(left.to_vec(), vec![0]);
    }
}
