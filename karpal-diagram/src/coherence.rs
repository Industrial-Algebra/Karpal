//! Coherence law witnesses for monoidal categories.
//!
//! Provides type-level proofs for the pentagon, triangle, and hexagon
//! identities that every (braided) monoidal category must satisfy.
//!
//! These are *axioms* of the `Tensor` and `Braiding` traits — the
//! `Justifies` impls declare them true, and the `verify_*` functions
//! produce type-level `Rewrite` witnesses.

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

// ---------------------------------------------------------------------------
// Hexagon identity
// ---------------------------------------------------------------------------

/// Proof term: two paths from `(A⊗B)⊗C` to `(B⊗C)⊗A` coincide.
///
/// - **upper** path: `braid ; α⁻¹ ; braid ; α⁻¹`
/// - **lower** path: `α ; braid`
pub struct HexagonIdentity;

impl<A, B, C> Justifies<((A, B), C), ((B, C), A)> for HexagonIdentity {}

/// Construct a hexagon identity witness.
///
/// Returns a type-level `Rewrite` witnessing the hexagon coherence law.
#[cfg(any(feature = "std", feature = "alloc"))]
#[allow(clippy::type_complexity)]
pub fn verify_hexagon<A, B, C>() -> Rewrite<((A, B), C), ((B, C), A), HexagonIdentity> {
    Rewrite::witness()
}

// ---------------------------------------------------------------------------
// Diagrammatic rewriting bridge
// ---------------------------------------------------------------------------

/// Justification: diagram equivalence established by runtime normalization.
///
/// This bridges the runtime `Diagram::equivalent_to` check with the
/// `karpal-proof` type-level `Justifies`/`Rewrite` framework. Any two
/// types `A` and `B` can serve as phantom markers — the real evidence
/// comes from normalizing the two diagrams and checking they match.
pub struct ByNormalization;

impl<A, B> Justifies<A, B> for ByNormalization {}

/// Prove two diagrams are equivalent via normalization.
///
/// Normalizes both diagrams and, if they match, returns a type-level
/// `Rewrite` witness. The type parameters `A` and `B` are phantom markers
/// that let callers distinguish different proof instances.
///
/// Returns `None` when the diagrams differ after normalization.
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn equivalent_proved<A, B>(
    a: &crate::Diagram,
    b: &crate::Diagram,
) -> Option<Rewrite<A, B, ByNormalization>> {
    if a.equivalent_to(b) {
        Some(Rewrite::witness())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Compact-closed / yanking witnesses
// ---------------------------------------------------------------------------

/// Justification: cup/cap yanking equation verified by normalization.
///
/// The yanking equation `(cup ⊗ id) ; (id ⊗ cap) = id` is a defining
/// axiom of compact-closed categories. This witness asserts it holds
/// for the given diagram arity.
pub struct ByYanking;

impl<A, B> Justifies<A, B> for ByYanking {}

/// Prove the yanking equation for the given arity.
///
/// Builds both left and right yanking diagrams, verifies they normalize
/// to identity, and returns a type-level `Rewrite` witness.
///
/// # Panics
///
/// Panics if the yanking diagram fails to normalize to identity —
/// this should never happen for a correct monoidal diagram implementation.
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn prove_yanking<A, B>(arity: usize) -> Rewrite<A, B, ByYanking> {
    let left_yank = crate::Diagram::cup(arity)
        .parallel(crate::Diagram::identity(arity))
        .then(crate::Diagram::identity(arity).parallel(crate::Diagram::cap(arity)));

    let right_yank = crate::Diagram::identity(arity)
        .parallel(crate::Diagram::cup(arity))
        .then(crate::Diagram::cap(arity).parallel(crate::Diagram::identity(arity)));

    assert!(
        left_yank.equivalent_to(&crate::Diagram::identity(arity)),
        "left yanking equation failed for arity {arity}"
    );
    assert!(
        right_yank.equivalent_to(&crate::Diagram::identity(arity)),
        "right yanking equation failed for arity {arity}"
    );

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

    #[test]
    fn hexagon_produces_witness() {
        let _: Rewrite<((i32, u8), bool), ((u8, bool), i32), HexagonIdentity> =
            verify_hexagon::<i32, u8, bool>();
    }

    #[test]
    fn hexagon_paths_agree() {
        use crate::Braiding;

        let input = ((1_i32, 2_u8), true);

        // Upper path: braid ; α⁻¹ ; braid ; α⁻¹
        let braid_ab_c = FnA::braid::<(i32, u8), bool>();
        let assoc_inv_c_a_b = FnA::associate_inv::<bool, i32, u8>();
        let braid_ca_b = FnA::braid::<(bool, i32), u8>();
        let assoc_inv_b_c_a = FnA::associate_inv::<u8, bool, i32>();
        let upper = FnA::compose(
            assoc_inv_b_c_a,
            FnA::compose(braid_ca_b, FnA::compose(assoc_inv_c_a_b, braid_ab_c)),
        );

        // Lower path: α ; braid
        let assoc_a_b_c = FnA::associate::<i32, u8, bool>();
        let braid_a_bc = FnA::braid::<i32, (u8, bool)>();
        let lower = FnA::compose(braid_a_bc, assoc_a_b_c);

        assert_eq!(upper(input.clone()), lower(input));
    }
}
