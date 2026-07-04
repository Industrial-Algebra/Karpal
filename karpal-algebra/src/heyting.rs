// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use crate::bounded_lattice::BoundedLattice;

/// A Heyting algebra: a bounded distributive lattice with a relative
/// pseudo-complement (Heyting implication).
///
/// Heyting algebras generalize Boolean algebras. The key difference is that
/// the law of excluded middle does not hold: `¬¬a ≠ a` in general. Heyting
/// algebras are the algebraic semantics of intuitionistic logic.
///
/// Laws (in addition to `BoundedLattice` laws):
/// - `a ∧ (a → b) = a ∧ b` (modus ponens / adjunction)
/// - `a → a = top()` (identity)
/// - `a → bottom() = ¬a` (negation as implication of bottom)
/// - `¬a = a → bottom()` (negation definition)
/// - `¬¬a ≤ a` (double negation is weaker than identity)
///
/// The structured emptiness lattice Ω is the canonical non-Boolean Heyting
/// algebra instance.
pub trait HeytingAlgebra: BoundedLattice {
    /// Heyting implication: the largest `c` such that `a ∧ c ≤ b`.
    ///
    /// In Boolean logic this would be `¬a ∨ b`, but in Heyting logic it is
    /// generally different — it is the right adjoint to meet.
    fn implies(self, other: Self) -> Self;

    /// Heyting negation: `¬a = a → bottom()`.
    ///
    /// Unlike Boolean negation, `¬¬a ≠ a` in general.
    fn neg(self) -> Self {
        self.implies(Self::bottom())
    }
}

/// The Boolean Heyting algebra on `bool`.
///
/// For `bool`, Heyting implication coincides with Boolean implication:
/// `a → b = ¬a ∨ b`.
impl HeytingAlgebra for bool {
    fn implies(self, other: Self) -> Self {
        !self || other
    }
}

macro_rules! impl_heyting_for_ord {
    ($($t:ty),*) => {
        $(
            impl HeytingAlgebra for $t {
                fn implies(self, other: Self) -> Self {
                    // For totally ordered sets: a → b = top if a ≤ b, else b
                    if self <= other {
                        Self::top()
                    } else {
                        other
                    }
                }
            }
        )*
    };
}

impl_heyting_for_ord!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool_implies_is_material_implication() {
        assert_eq!(true.implies(true), true);
        assert_eq!(true.implies(false), false);
        assert_eq!(false.implies(true), true);
        assert_eq!(false.implies(false), true);
    }

    #[test]
    fn bool_negation_is_boolean_not() {
        assert_eq!(true.neg(), false);
        assert_eq!(false.neg(), true);
    }

    #[test]
    fn bool_double_negation_holds_for_boolean() {
        // For Bool specifically, ¬¬a = a (Boolean algebras are Heyting)
        assert_eq!(true.neg().neg(), true);
        assert_eq!(false.neg().neg(), false);
    }

    #[test]
    fn bool_implies_identity() {
        assert_eq!(true.implies(true), true);
        assert_eq!(false.implies(false), true);
    }

    #[test]
    fn i32_implies_gives_top_when_leq() {
        assert_eq!(5i32.implies(10), i32::top());
        assert_eq!(5i32.implies(5), i32::top());
    }

    #[test]
    fn i32_implies_gives_other_when_gt() {
        assert_eq!(10i32.implies(5), 5);
    }

    #[test]
    fn i32_negation_is_implication_of_bottom() {
        assert_eq!(5i32.neg(), i32::bottom());
        assert_eq!(i32::MIN.neg(), i32::top());
    }
}
