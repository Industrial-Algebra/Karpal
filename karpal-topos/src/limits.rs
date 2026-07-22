// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Finite limits in the presheaf topos: pullbacks and equalizers.
//!
//! Limits in a presheaf topos `[C^op, Set]` are computed **pointwise**:
//! the pullback of `P → R ← Q` at object `c` is
//! `{(p, q) | f_c(p) = g_c(q)}`, and the equalizer of `P ⇉ Q` at `c` is
//! `{p | f_c(p) = g_c(p)}`.
//!
//! Because presheaf morphisms (natural transformations) cannot be first-class
//! values in Rust (the rank-N wall, see the 16B design doc), these are exposed
//! as *fiber* functions that take the presheaf values and morphism actions at a
//! single object. The caller enumerates objects (`0..=N` for `ChainCat`) and
//! calls these per object.

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;

/// Compute the **pullback fiber** at one object: all pairs `(p, q)` with
/// `f(p) == g(q)`.
///
/// This is the value of the pullback presheaf `P ×_R Q` at a single object.
/// Enumerate the elements of `P` and `Q` at that object and supply the two
/// morphism actions `f: P → R` and `g: Q → R`.
///
/// # Example
///
/// ```
/// use karpal_topos::limits::pullback_fiber;
/// let ps = [1, 2, 3];
/// let qs = [10, 20, 30];
/// // f(p) = p mod 2, g(q) = (q / 10) mod 2
/// let pb: Vec<(i32, i32)> = pullback_fiber(&ps, &qs, |p| p % 2, |q| (q / 10) % 2);
/// // pairs where p%2 == (q/10)%2: (1,10)? 1%2=1, 1%2=1 ✓; (2,20)? 0,0 ✓; ...
/// ```
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn pullback_fiber<P: Clone, Q: Clone, R: PartialEq, F: Fn(&P) -> R, G: Fn(&Q) -> R>(
    ps: &[P],
    qs: &[Q],
    f: F,
    g: G,
) -> Vec<(P, Q)> {
    let mut out = Vec::new();
    for p in ps {
        let fp = f(p);
        for q in qs {
            if g(q) == fp {
                out.push((p.clone(), q.clone()));
            }
        }
    }
    out
}

/// Compute the **equalizer fiber** at one object: all `p` with `f(p) == g(p)`.
///
/// This is the value of the equalizer sub-presheaf of `P` (for the parallel
/// pair `f, g: P → Q`) at a single object.
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn equalizer_fiber<P: Clone, R: PartialEq, F: Fn(&P) -> R, G: Fn(&P) -> R>(
    ps: &[P],
    f: F,
    g: G,
) -> Vec<P> {
    let mut out = Vec::new();
    for p in ps {
        if f(p) == g(p) {
            out.push(p.clone());
        }
    }
    out
}

/// Compute the **characteristic morphism** `χ: P → Ω` at one object for a
/// subobject `S ↪ P`.
///
/// Given an element `p ∈ P(i)` and a predicate `in_subobject` testing
/// membership in `S`, `χ(p)` is the largest sieve (truth value) `R` such that
/// `p` restricted along any morphism in `R` remains in `S`. For `ChainCat`
/// this is the largest rank `r` such that the restriction of `p` to any object
/// covered by the rank-`r` sieve stays in `S`.
///
/// The supplied `restrict_into_sub` maps `(p, target_index)` to the element of
/// `P(target)` obtained by restricting `p`, so the caller can test whether it
/// lies in `S(target)`.
///
/// The defining theorem — **a subobject is the pullback of `truth` along χ** —
/// means: `p ∈ S(i)` iff `χ(p)` is the maximal sieve on `i` (rank `i + 1`).
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn characteristic_at<P, R>(
    i: usize,
    p: &P,
    restrict_into_sub: R,
) -> crate::classifier::TruthValue
where
    R: Fn(&P, usize) -> bool,
{
    // For ChainCat: χ(p) = largest rank r such that for all j < r, p restricted
    // to object j lies in S(j). We probe objects 0..=i in order and find the
    // largest prefix where membership holds.
    let mut rank = 0usize;
    for j in 0..=i {
        if restrict_into_sub(p, j) {
            rank = j + 1;
        } else {
            break;
        }
    }
    crate::classifier::TruthValue { rank }
}

#[cfg(all(test, any(feature = "std", feature = "alloc")))]
mod tests {
    #![allow(dead_code)]
    use super::*;
    use crate::classifier::{TruthValue, truth_at};

    #[test]
    fn pullback_fiber_basic() {
        let ps = [1, 2, 3, 4];
        let qs = [10, 20, 30, 40];
        // f(p) = p mod 2, g(q) = (q/10) mod 2.
        // p=1(odd=1) ↔ q with g=1: 10(1),30(1); p=2(even=0) ↔ 20(0),40(0); etc.
        let pb: Vec<(i32, i32)> = pullback_fiber(&ps, &qs, |p| p % 2, |q| (q / 10) % 2);
        assert!(pb.contains(&(1, 10)));
        assert!(pb.contains(&(1, 30)));
        assert!(pb.contains(&(2, 20)));
        assert!(pb.contains(&(2, 40)));
        assert!(pb.contains(&(3, 30)));
        assert!(pb.contains(&(4, 20)));
        // These must NOT be present (mismatched parity):
        assert!(!pb.contains(&(2, 10)));
        assert!(!pb.contains(&(1, 20)));
        assert_eq!(pb.len(), 8); // 2 odd-p × 2 odd-q + 2 even-p × 2 even-q
    }

    #[test]
    fn equalizer_fiber_basic() {
        let ps = [1, 2, 3, 4];
        // f(p) = p, g(p) = p + (p%2)  → equal when p even.
        let eq: Vec<i32> = equalizer_fiber(&ps, |p| *p, |p| p + (p % 2));
        assert_eq!(eq, vec![2, 4]);
    }

    #[test]
    fn characteristic_maximal_for_full_subobject() {
        // If p restricted to every j ≤ i lies in S, χ(p) = maximal sieve.
        let chi = characteristic_at(2, &42i32, |_p, _j| true);
        assert_eq!(chi, truth_at(2)); // maximal sieve on object 2
    }

    #[test]
    fn characteristic_empty_for_no_membership() {
        // If p restricted to any j is not in S, χ(p) = empty sieve (rank 0).
        let chi = characteristic_at(2, &42i32, |_p, _j| false);
        assert_eq!(chi, TruthValue::bottom());
    }

    #[test]
    fn characteristic_prefix() {
        // p restricted into S at j=0 and j=1, but not j=2 → rank 2.
        let chi = characteristic_at(2, &42i32, |_p, j| j < 2);
        assert_eq!(chi, TruthValue { rank: 2 });
    }

    #[test]
    fn subobject_is_pullback_of_truth() {
        // The defining theorem: p ∈ S(i) iff χ(p) is the maximal sieve on i.
        let chi_in = characteristic_at(2, &42i32, |_p, _j| true);
        assert_eq!(chi_in, truth_at(2));

        let chi_partial = characteristic_at(2, &42i32, |_p, j| j < 1);
        assert!(chi_partial < truth_at(2));
    }
}
