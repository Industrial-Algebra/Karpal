// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Grothendieck topologies and Lawvere-Tierney closure operators.
//!
//! A **Grothendieck topology** `J` assigns to each object a collection of
//! covering sieves satisfying maximality, stability, and transitivity. A
//! **Lawvere-Tierney topology** is the equivalent notion as a closure operator
//! `j: Ω → Ω` on the subobject classifier.
//!
//! Both are realized concretely over [`ChainCat`](crate::small_category::ChainCat),
//! where a sieve on object `i` is identified by its rank `r ∈ {0,…,i+1}`
//! (see [`TruthValue`](crate::classifier::TruthValue)).

// ---------------------------------------------------------------------------
// Grothendieck topologies
// ---------------------------------------------------------------------------

/// A Grothendieck topology: which sieves count as *covering*.
///
/// Laws (verified by [`axiom_maximality`], [`axiom_stability`], and the
/// transitivity tests in the concrete instances):
/// 1. **Maximality** — the maximal sieve (rank `i+1`) is covering.
/// 2. **Stability** — if rank `r` covers `i`, then `min(r, j+1)` covers `j`.
/// 3. **Transitivity** — checked per concrete instance.
pub trait GrothendieckTopology {
    /// Is the sieve of rank `r` on object `i` a covering sieve?
    fn is_covering(i: usize, rank: usize) -> bool;
}

/// The trivial (indiscrete) topology: only the maximal sieve covers.
pub struct TrivialTopology;

impl GrothendieckTopology for TrivialTopology {
    fn is_covering(i: usize, rank: usize) -> bool {
        rank == i + 1
    }
}

/// The dense topology: any non-empty sieve covers.
pub struct DenseTopology;

impl GrothendieckTopology for DenseTopology {
    fn is_covering(_i: usize, rank: usize) -> bool {
        rank >= 1
    }
}

/// Axiom 1 (maximality): the maximal sieve on `i` is always covering.
pub fn axiom_maximality<J: GrothendieckTopology>(i: usize) -> bool {
    J::is_covering(i, i + 1)
}

/// Axiom 2 (stability / base change): if `rank` covers `i`, then pulling it
/// back along `j → i` (yielding rank `min(rank, j+1)`) covers `j`.
pub fn axiom_stability<J: GrothendieckTopology>(i: usize, rank: usize, j: usize) -> bool {
    if J::is_covering(i, rank) {
        J::is_covering(j, rank.min(j + 1))
    } else {
        true
    }
}

/// Axiom 3 (transitivity): if `s_rank` covers `i`, and a sieve `r_rank` is such
/// that its pullback to every object `k < s_rank` covers `k`, then `r_rank`
/// covers `i`.
pub fn axiom_transitivity<J: GrothendieckTopology>(i: usize, s_rank: usize, r_rank: usize) -> bool {
    if !J::is_covering(i, s_rank) {
        return true;
    }
    // For every k covered by the sieve s_rank (i.e. k < s_rank), the pullback
    // of r_rank to k must cover k. Pullback of rank r to k is min(r, k+1).
    let pullbacks_cover = (0..s_rank).all(|k| J::is_covering(k, r_rank.min(k + 1)));
    if pullbacks_cover {
        J::is_covering(i, r_rank)
    } else {
        true
    }
}

// ---------------------------------------------------------------------------
// Lawvere-Tierney topologies
// ---------------------------------------------------------------------------

/// A Lawvere-Tierney topology: a closure operator `j: Ω → Ω` on truth values.
///
/// Laws:
/// - `j(top) = top` (top is closed)
/// - `j(j(r)) = j(r)` (idempotence)
/// - `j(min(r, s)) = min(j(r), j(s))` (meet-preserving)
///
/// There is a bijection between Grothendieck topologies and Lawvere-Tierney
/// topologies; the concrete instances below correspond to the
/// [`GrothendieckTopology`] instances of the same name.
pub trait LawvereTierneyTopology {
    /// Apply the closure operator at object `i` to a truth value of rank `r`,
    /// returning the closed rank.
    fn j(i: usize, rank: usize) -> usize;
}

/// Axiom: `j` preserves top (`j(i+1) == i+1`).
pub fn lt_axiom_top<J: LawvereTierneyTopology>(i: usize) -> bool {
    J::j(i, i + 1) == i + 1
}

/// Axiom: `j` is idempotent (`j(j(r)) == j(r)`).
pub fn lt_axiom_idempotent<J: LawvereTierneyTopology>(i: usize, r: usize) -> bool {
    J::j(i, J::j(i, r)) == J::j(i, r)
}

/// Axiom: `j` preserves meet (`j(min(r,s)) == min(j(r), j(s))`).
pub fn lt_axiom_meet<J: LawvereTierneyTopology>(i: usize, r: usize, s: usize) -> bool {
    J::j(i, r.min(s)) == J::j(i, r).min(J::j(i, s))
}

impl LawvereTierneyTopology for TrivialTopology {
    fn j(i: usize, _rank: usize) -> usize {
        // Everything closes to the top.
        i + 1
    }
}

impl LawvereTierneyTopology for DenseTopology {
    fn j(_i: usize, rank: usize) -> usize {
        // The empty sieve (rank 0) closes to rank 1; everything else is closed.
        rank.max(1)
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;

    // ---- Grothendieck topology axioms ----

    #[test]
    fn trivial_maximality() {
        for i in 0..=5 {
            assert!(axiom_maximality::<TrivialTopology>(i));
        }
    }

    #[test]
    fn trivial_stability() {
        for i in 0..=5 {
            for r in 0..=i + 1 {
                for j in 0..=i {
                    assert!(axiom_stability::<TrivialTopology>(i, r, j));
                }
            }
        }
    }

    #[test]
    fn trivial_transitivity() {
        for i in 0..=5 {
            for s in 0..=i + 1 {
                for r in 0..=i + 1 {
                    assert!(axiom_transitivity::<TrivialTopology>(i, s, r));
                }
            }
        }
    }

    #[test]
    fn dense_maximality() {
        for i in 0..=5 {
            assert!(axiom_maximality::<DenseTopology>(i));
        }
    }

    #[test]
    fn dense_stability() {
        for i in 0..=5 {
            for r in 0..=i + 1 {
                for j in 0..=i {
                    assert!(axiom_stability::<DenseTopology>(i, r, j));
                }
            }
        }
    }

    #[test]
    fn dense_transitivity() {
        for i in 0..=5 {
            for s in 0..=i + 1 {
                for r in 0..=i + 1 {
                    assert!(axiom_transitivity::<DenseTopology>(i, s, r));
                }
            }
        }
    }

    #[test]
    fn trivial_only_maximal_covers() {
        for i in 0..=5 {
            for r in 0..=i {
                assert!(!TrivialTopology::is_covering(i, r), "rank {r} on {i}");
            }
            assert!(TrivialTopology::is_covering(i, i + 1));
        }
    }

    #[test]
    fn dense_nonempty_covers() {
        for i in 0..=5 {
            assert!(!DenseTopology::is_covering(i, 0));
            for r in 1..=i + 1 {
                assert!(DenseTopology::is_covering(i, r));
            }
        }
    }

    // ---- Lawvere-Tierney axioms ----

    #[test]
    fn lt_trivial_axioms() {
        for i in 0..=5 {
            assert!(lt_axiom_top::<TrivialTopology>(i));
            for r in 0..=i + 1 {
                assert!(lt_axiom_idempotent::<TrivialTopology>(i, r));
                for s in 0..=i + 1 {
                    assert!(lt_axiom_meet::<TrivialTopology>(i, r, s));
                }
            }
        }
    }

    #[test]
    fn lt_dense_axioms() {
        for i in 0..=5 {
            assert!(lt_axiom_top::<DenseTopology>(i));
            for r in 0..=i + 1 {
                assert!(lt_axiom_idempotent::<DenseTopology>(i, r));
                for s in 0..=i + 1 {
                    assert!(lt_axiom_meet::<DenseTopology>(i, r, s));
                }
            }
        }
    }

    #[test]
    fn lt_dense_closes_empty_to_rank1() {
        assert_eq!(DenseTopology::j(5, 0), 1);
        assert_eq!(DenseTopology::j(5, 3), 3); // non-empty unchanged
    }

    #[test]
    fn lt_trivial_closes_everything_to_top() {
        assert_eq!(TrivialTopology::j(5, 0), 6);
        assert_eq!(TrivialTopology::j(5, 3), 6);
    }
}
