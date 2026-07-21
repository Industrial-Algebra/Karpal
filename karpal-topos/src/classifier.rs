// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! The subobject classifier Ω and the terminal object.
//!
//! In a presheaf topos `[C^op, Set]`, the subobject classifier is the presheaf
//! assigning to each object `c` the set of **sieves on `c`**. The truth map
//! `1 → Ω` selects the maximal sieve. This module realizes Ω concretely over
//! [`ChainCat`](crate::small_category::ChainCat), where sieves are downward-
//! closed subsets representable by a rank — a chain Heyting algebra that is a
//! concrete instance of the structured-emptiness lattice.

use crate::presheaf::Presheaf;
use crate::small_category::{ChainMor, SmallCategory};

/// A truth value in Ω: a sieve represented by its rank (the size of the
/// downward-closed set).
///
/// For `ChainCat<N>`, `Ω(i)` contains ranks `0..=i+1`:
/// - rank `0` = the empty sieve (bottom — "nothing is covered")
/// - rank `k` = the sieve `{0, …, k-1}` (sources into `i`)
/// - rank `i+1` = the maximal sieve (top — "everything is covered")
///
/// Ordered by inclusion, these form a chain Heyting algebra.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct TruthValue {
    pub rank: usize,
}

impl TruthValue {
    /// The bottom truth value (empty sieve).
    pub fn bottom() -> Self {
        TruthValue { rank: 0 }
    }

    /// The top truth value at object `i`: the maximal sieve on `i`
    /// (rank `i + 1`).
    pub fn top_at(i: usize) -> Self {
        TruthValue { rank: i + 1 }
    }

    /// Lattice meet (sieve intersection): the smaller down-set.
    pub fn meet(self, other: TruthValue) -> TruthValue {
        TruthValue {
            rank: self.rank.min(other.rank),
        }
    }

    /// Lattice join (sieve union): the larger down-set.
    pub fn join(self, other: TruthValue) -> TruthValue {
        TruthValue {
            rank: self.rank.max(other.rank),
        }
    }

    /// Heyting implication on the chain: `self → other`.
    ///
    /// On a chain Heyting algebra, `a → b = top` if `a ≤ b`, else `b`.
    /// Here `top` is the maximal truth value at the relevant object; the
    /// caller supplies the object index `i` so `top = top_at(i)`.
    pub fn implies_at(self, other: TruthValue, i: usize) -> TruthValue {
        if self.rank <= other.rank {
            TruthValue::top_at(i)
        } else {
            other
        }
    }

    /// Heyting negation: `¬a = a → bottom`. On a chain, this is `bottom`
    /// unless `a` is already `bottom` (in which case it is `top_at(i)`).
    pub fn neg_at(self, i: usize) -> TruthValue {
        if self.rank == 0 {
            TruthValue::top_at(i)
        } else {
            TruthValue::bottom()
        }
    }
}

/// The subobject classifier Ω of the presheaf topos `[ChainCat<N>^op, Set]`.
///
/// `Ω(i) = { sieves on i }`, concretely the chain of ranks `0..=i+1`.
/// Restriction along `f: Dom → Cod` pulls a sieve back: rank `r` becomes
/// `min(r, Dom::IDX + 1)`.
pub struct Omega;

impl<const N: usize> Presheaf<crate::small_category::ChainCat<N>> for Omega {
    type At<Obj> = TruthValue;

    fn restrict<Dom, Cod>(f: ChainMor<Dom, Cod>, tv: TruthValue) -> TruthValue {
        // Pull back the sieve along f: Dom → Cod. A rank-r sieve on Cod
        // pulls back to the rank-min(r, Dom+1) sieve on Dom.
        TruthValue {
            rank: tv.rank.min(f.from() + 1),
        }
    }
}

/// The terminal presheaf: sends every object to the singleton `()`.
///
/// This is the terminal object of the presheaf topos; the truth map
/// `1 → Ω` is a morphism from `Terminal` to `Omega`.
pub struct Terminal;

impl<C: SmallCategory> Presheaf<C> for Terminal {
    type At<Obj> = ();

    fn restrict<Dom, Cod>(_f: C::Mor<Dom, Cod>, _x: ()) {}
}

/// The truth map `true: 1 → Ω` evaluated at object `i`: the maximal sieve
/// on `i` (rank `i + 1`).
///
/// A subobject `S ↪ A` corresponds to the unique `χ: A → Ω` whose pullback
/// along `true` recovers `S`.
pub fn truth_at(i: usize) -> TruthValue {
    TruthValue::top_at(i)
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;
    use crate::presheaf::Presheaf;
    use crate::small_category::{ChainCat, ChainObj};

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
    fn truth_value_top_and_bottom() {
        assert_eq!(TruthValue::bottom(), TruthValue { rank: 0 });
        assert_eq!(TruthValue::top_at(2), TruthValue { rank: 3 });
        assert_eq!(truth_at(2), TruthValue::top_at(2));
    }

    #[test]
    fn lattice_meet_join() {
        let a = TruthValue { rank: 2 };
        let b = TruthValue { rank: 4 };
        assert_eq!(a.meet(b), TruthValue { rank: 2 });
        assert_eq!(a.join(b), TruthValue { rank: 4 });
    }

    #[test]
    fn heyting_implies() {
        // 2 → 4 at object with top rank 5: 2 ≤ 4, so implies = top.
        let r = TruthValue { rank: 2 }.implies_at(TruthValue { rank: 4 }, 4);
        assert_eq!(r, TruthValue::top_at(4));
        // 4 → 2: 4 > 2, so implies = 2.
        let r = TruthValue { rank: 4 }.implies_at(TruthValue { rank: 2 }, 4);
        assert_eq!(r, TruthValue { rank: 2 });
    }

    #[test]
    fn heyting_neg() {
        // ¬(rank 0) = top; ¬(rank > 0) = bottom.
        assert_eq!(TruthValue::bottom().neg_at(3), TruthValue::top_at(3));
        assert_eq!(TruthValue { rank: 2 }.neg_at(3), TruthValue::bottom());
    }

    #[test]
    fn omega_restrict_pulls_back_sieve() {
        // f: C0 → C2. A rank-3 sieve on C2 pulls back to min(3, 0+1) = 1.
        let f = ChainCat::<2>::morphism::<C0, C2>().unwrap();
        let tv = TruthValue { rank: 3 }; // maximal sieve on C2
        let restricted: TruthValue = <Omega as Presheaf<ChainCat<2>>>::restrict(f, tv);
        assert_eq!(restricted, TruthValue { rank: 1 });
    }

    #[test]
    fn omega_restrict_truth_is_natural() {
        // Pulling back the maximal sieve along f: C1 → C2 gives the maximal
        // sieve on C1 (rank 2 = C1::IDX + 1). This is the naturality of truth.
        let f = ChainCat::<2>::morphism::<C1, C2>().unwrap();
        let truth_c2 = truth_at(2);
        let pulled: TruthValue = <Omega as Presheaf<ChainCat<2>>>::restrict(f, truth_c2);
        assert_eq!(pulled, truth_at(1));
    }

    #[test]
    fn omega_identity_law() {
        let id = ChainCat::<2>::identity::<C1>();
        let tv = TruthValue { rank: 2 };
        let result: TruthValue = <Omega as Presheaf<ChainCat<2>>>::restrict(id, tv);
        assert_eq!(result, tv);
    }

    #[test]
    fn terminal_is_singleton() {
        let id = ChainCat::<2>::identity::<C2>();
        let _: () = <Terminal as Presheaf<ChainCat<2>>>::restrict(id, ());
    }
}
