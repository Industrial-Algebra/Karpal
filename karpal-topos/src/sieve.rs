// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Sieves: precomposition-closed families of morphisms.
//!
//! A **sieve** on an object `c` of a category `C` is a subfunctor of the
//! representable presheaf `Hom(-, c)`: a family `S` of morphisms into `c`
//! such that whenever `f: d → c` is in `S` and `g: e → d` is any morphism,
//! the composite `f ∘ g: e → c` is also in `S`.
//!
//! Sieves are the "covering" concept underlying Grothendieck topologies
//! (Phase 16D). This module provides a finite, enumerable form
//! ([`FiniteSieve`]) suitable for testing on small categories.

use core::marker::PhantomData;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;

#[cfg(any(feature = "std", feature = "alloc"))]
use crate::small_category::{ChainCat, ChainMor, ChainObj, SmallCategory};

/// A sieve on an object `Cod`: a precomposition-closed family of morphisms
/// into `Cod`.
///
/// This trait is the abstract interface; [`FiniteSieve`] is the concrete,
/// enumerable implementation for small categories.
pub trait Sieve<C: SmallCategory, Cod> {
    /// Is the morphism `f: Dom → Cod` a member of the sieve?
    fn contains<Dom>(&self, f: &C::Mor<Dom, Cod>) -> bool;

    /// Verify precomposition closure: for every `f ∈ S` and every composable
    /// `g`, the composite `f ∘ g` is also in `S`.
    fn is_closed(&self) -> bool;
}

/// A finite sieve over [`ChainCat`]: an explicit set of source indices whose
/// morphisms into `Cod` are members. A morphism `Dom → Cod` is in the sieve
/// iff `Dom::IDX ∈ sources`.
///
/// Closure under precomposition means: if `i ∈ sources` and `j ≤ i`, then
/// `j ∈ sources` (because `Dom=j → i → Cod` composes to `j → Cod`).
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct FiniteSieve<Cod> {
    /// Sorted, deduplicated source object indices whose morphisms into `Cod`
    /// belong to the sieve.
    sources: Vec<usize>,
    _cod: PhantomData<fn() -> Cod>,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<Cod> FiniteSieve<Cod> {
    /// Construct a sieve from a set of source indices. The sieve is
    /// **not** automatically closed; call [`Self::close`] to enforce
    /// precomposition closure in place.
    pub fn new(sources: impl IntoIterator<Item = usize>) -> Self {
        let mut v: Vec<usize> = sources.into_iter().collect();
        v.sort_unstable();
        v.dedup();
        FiniteSieve {
            sources: v,
            _cod: PhantomData,
        }
    }

    /// Enforce precomposition closure: for each `i ∈ sources`, add all
    /// `j ≤ i`. For a chain category this means: if `i` is present, every
    /// `j ∈ [0, i]` must be present.
    pub fn close(mut self) -> Self {
        if let Some(&max) = self.sources.iter().max() {
            self.sources = (0..=max).collect();
        }
        self
    }

    /// Is source index `i` in the sieve?
    pub fn contains_index(&self, i: usize) -> bool {
        self.sources.binary_search(&i).is_ok()
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<Cod: ChainObj> FiniteSieve<Cod> {
    /// The maximal sieve on `Cod`: contains all morphisms into `Cod`,
    /// i.e. source indices `[0, Cod::IDX]`.
    pub fn maximal() -> Self {
        FiniteSieve::new(0..=Cod::IDX).close()
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<Cod, const N: usize> Sieve<ChainCat<N>, Cod> for FiniteSieve<Cod> {
    fn contains<Dom>(&self, f: &ChainMor<Dom, Cod>) -> bool {
        self.contains_index(f.from())
    }

    fn is_closed(&self) -> bool {
        // For a chain: closure requires that if i is present, all j <= i are.
        match self.sources.iter().max() {
            None => true,
            Some(&max) => {
                self.sources.len() == max + 1
                    && self.sources.iter().enumerate().all(|(k, &v)| v == k)
            }
        }
    }
}

#[cfg(all(test, any(feature = "std", feature = "alloc")))]
mod tests {
    #![allow(dead_code)]
    use super::*;

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
    fn maximal_sieve_contains_all_sources() {
        let s: FiniteSieve<C3> = FiniteSieve::maximal();
        for i in 0..=3 {
            assert!(s.contains_index(i), "index {i} should be in maximal sieve");
        }
    }

    #[test]
    fn unclosed_sieve_detected() {
        // {2} alone is not closed: precomposition with 0→2 and 1→2 requires
        // 0 and 1 to be present too.
        let s: FiniteSieve<C3> = FiniteSieve::new([2]);
        assert!(!Sieve::<ChainCat<3>, C3>::is_closed(&s));
    }

    #[test]
    fn close_enforces_downward_closure() {
        let s: FiniteSieve<C3> = FiniteSieve::new([2]).close();
        assert!(Sieve::<ChainCat<3>, C3>::is_closed(&s));
        assert!(s.contains_index(0));
        assert!(s.contains_index(1));
        assert!(s.contains_index(2));
    }

    #[test]
    fn sieve_membership_via_morphism() {
        let s: FiniteSieve<C3> = FiniteSieve::new([0, 1]).close();
        let f = ChainCat::<3>::morphism::<C1, C3>().unwrap();
        assert!(Sieve::<ChainCat<3>, C3>::contains(&s, &f));
        let g = ChainCat::<3>::morphism::<C2, C3>().unwrap();
        assert!(!Sieve::<ChainCat<3>, C3>::contains(&s, &g));
    }

    #[test]
    fn empty_sieve_is_closed() {
        let s: FiniteSieve<C3> = FiniteSieve::new(core::iter::empty::<usize>());
        assert!(Sieve::<ChainCat<3>, C3>::is_closed(&s));
    }
}
