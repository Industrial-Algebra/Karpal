//! Coherence law witnesses for monoidal categories.
//!
//! Provides type-level proofs for the pentagon and triangle identities
//! that every monoidal category must satisfy.
//!
//! These are *axioms* of the `Tensor` trait — the `Justifies` impls
//! declare them true, and the `verify_*` functions produce type-level
//! `Rewrite` witnesses.

use karpal_proof::rewrite::Justifies;
#[cfg(any(feature = "std", feature = "alloc"))]
use karpal_proof::rewrite::Rewrite;

// ---------------------------------------------------------------------------
// Pentagon identity
// ---------------------------------------------------------------------------

/// Proof term: both reassociation paths from `((A⊗B)⊗C)⊗D` to `A⊗(B⊗(C⊗D))` coincide.
///
/// - **upper** path: `(α ⊗ id) ; α ; (id ⊗ α)`
/// - **lower** path: `α ; α`
pub struct PentagonIdentity;

impl<A, B, C, D> Justifies<(((A, B), C), D), (A, (B, (C, D)))> for PentagonIdentity {}

/// Construct a pentagon identity witness.
///
/// Returns a type-level `Rewrite` witnessing the pentagon coherence law.
#[cfg(any(feature = "std", feature = "alloc"))]
#[allow(clippy::type_complexity)]
pub fn verify_pentagon<A, B, C, D>() -> Rewrite<(((A, B), C), D), (A, (B, (C, D))), PentagonIdentity>
{
    Rewrite::witness()
}

// ---------------------------------------------------------------------------
// Triangle identity
// ---------------------------------------------------------------------------

/// Proof term: both cancellation paths from `(A⊗I)⊗B` to `A⊗B` coincide.
///
/// - **left** path: `ρ_A ⊗ id_B`
/// - **right** path: `α_{A,I,B} ; (id_A ⊗ λ_B)`
pub struct TriangleIdentity;

impl<A, B> Justifies<((A, ()), B), (A, B)> for TriangleIdentity {}

/// Construct a triangle identity witness.
///
/// Returns a type-level `Rewrite` witnessing the triangle coherence law.
#[cfg(any(feature = "std", feature = "alloc"))]
#[allow(clippy::type_complexity)]
pub fn verify_triangle<A, B>() -> Rewrite<((A, ()), B), (A, B), TriangleIdentity> {
    Rewrite::witness()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tensor;
    use karpal_arrow::{Category, FnA, Semigroupoid};

    #[test]
    fn pentagon_produces_witness() {
        let _: Rewrite<(((i32, u8), bool), String), (i32, (u8, (bool, String))), PentagonIdentity> =
            verify_pentagon::<i32, u8, bool, String>();
    }

    #[test]
    fn triangle_produces_witness() {
        let _: Rewrite<((i32, ()), String), (i32, String), TriangleIdentity> =
            verify_triangle::<i32, String>();
    }

    #[test]
    fn pentagon_paths_agree() {
        let input = (((1_i32, 2_u8), true), "end".to_string());

        // Upper path
        let upper_step1 = FnA::tensor(FnA::associate::<i32, u8, bool>(), FnA::id::<String>());
        let upper_step2 = FnA::associate::<i32, (u8, bool), String>();
        let upper_step3 = FnA::tensor(FnA::id::<i32>(), FnA::associate::<u8, bool, String>());
        let upper = FnA::compose(upper_step3, FnA::compose(upper_step2, upper_step1));

        // Lower path
        let lower_step1 = FnA::associate::<(i32, u8), bool, String>();
        let lower_step2 = FnA::associate::<i32, u8, (bool, String)>();
        let lower = FnA::compose(lower_step2, lower_step1);

        assert_eq!(upper(input.clone()), lower(input));
    }

    #[test]
    fn triangle_paths_agree() {
        let input = ((42_i32, ()), "hello".to_string());

        // Left path: ρ ⊗ id
        let left = FnA::tensor(FnA::right_unitor::<i32>(), FnA::id::<String>());

        // Right path: α ; (id ⊗ λ)
        let alpha = FnA::associate::<i32, (), String>();
        let right_step2 = FnA::tensor(FnA::id::<i32>(), FnA::left_unitor::<String>());
        let right = FnA::compose(right_step2, alpha);

        assert_eq!(left(input.clone()), right(input.clone()));
    }
}
