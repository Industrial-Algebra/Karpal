// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! The Yoneda lemma as a computable bijection.
//!
//! For any presheaf `P: C^op → Set` and object `c` of `C`:
//!
//! ```text
//! Nat(Hom(-, c), P)  ≅  P(c)
//! ```
//!
//! The bijection sends a natural transformation `α` to `α_c(id_c) ∈ P(c)`,
//! and sends an element `x ∈ P(c)` to the natural transformation whose
//! component at `d` maps `f: d → c` to `restrict(f, x) ∈ P(d)`.
//!
//! # Why we expose the *action*, not a first-class natural transformation
//!
//! Rust cannot represent a natural transformation as a single first-class
//! value (it is rank-N polymorphic over the object index). We therefore
//! expose the two directions of the bijection by their *computable action*:
//!
//! - [`yoneda_apply`] — given `x ∈ P(c)` and `f: d → c`, compute the
//!   component of the induced natural transformation at `d` applied to `f`.
//!   This is precisely `restrict(f, x)`.
//! - [`yoneda_extract`] — given the action of a natural transformation
//!   (as a function) and the identity morphism on `c`, recover `x ∈ P(c)`.
//!
//! The round-trip identity is then directly testable on concrete presheaves.

use crate::presheaf::Presheaf;
use crate::small_category::SmallCategory;

/// Apply the natural transformation `Hom(-, c) ⇒ P` induced by `x ∈ P(c)`
/// to a morphism `f: Dom → Cod`.
///
/// Concretely: `α_Cod(f) = restrict(f, x) ∈ P(Dom)`.
///
/// This is the forward direction of the Yoneda bijection in action form.
pub fn yoneda_apply<P, C, Dom, Cod>(f: C::Mor<Dom, Cod>, x: P::At<Cod>) -> P::At<Dom>
where
    C: SmallCategory,
    P: Presheaf<C>,
{
    P::restrict(f, x)
}

/// Extract the element `x ∈ P(c)` from a natural transformation
/// `Hom(-, c) ⇒ P` by evaluating it at `c` on the identity morphism.
///
/// Given the transformation's *action* (a function mapping a morphism
/// `f: Dom → c` to `P(Dom)`) and the identity `id_c`, this returns
/// `action(id_c) ∈ P(c)`.
///
/// This is the inverse direction of the Yoneda bijection.
pub fn yoneda_extract<P, C, Cod, F>(id_c: C::Mor<Cod, Cod>, action: F) -> P::At<Cod>
where
    C: SmallCategory,
    P: Presheaf<C>,
    F: FnOnce(C::Mor<Cod, Cod>) -> P::At<Cod>,
{
    action(id_c)
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;
    use crate::presheaf::{ConstantPresheaf, InitialSegmentPresheaf, SegmentSet};
    use crate::representable::Representable;
    use crate::small_category::{ChainCat, ChainObj};

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
    fn yoneda_apply_is_restrict_for_constant() {
        // x ∈ P(C2) for the constant presheaf at i32.
        let x: i32 = 99;
        let f = ChainCat::<3>::morphism::<C0, C2>().unwrap();
        let applied: i32 = yoneda_apply::<ConstantPresheaf<i32>, ChainCat<3>, C0, C2>(f, x);
        assert_eq!(applied, 99);
    }

    #[test]
    fn yoneda_roundtrip_extract_after_apply() {
        // Start with x ∈ P(C2). Build the NT action via yoneda_apply.
        // Extract back via yoneda_extract at id_{C2}. Should recover x.
        let x: SegmentSet = SegmentSet { len: 3 }; // P(C2) = {0,1,2}
        let id_c2 = ChainCat::<3>::identity::<C2>();

        // The action of the induced NT: given f: Dom → C2, return P(Dom).
        // We specialise to Dom = C2 for the extract (identity).
        let action = |f: <ChainCat<3> as SmallCategory>::Mor<C2, C2>| -> SegmentSet {
            yoneda_apply::<InitialSegmentPresheaf, ChainCat<3>, C2, C2>(f, x.clone())
        };

        let recovered: SegmentSet =
            yoneda_extract::<InitialSegmentPresheaf, ChainCat<3>, C2, _>(id_c2, action);
        // By the presheaf identity law, restrict(id, x) == x.
        assert_eq!(recovered.to_vec(), x.to_vec());
    }

    #[test]
    fn yoneda_apply_for_representable_recovers_precomposition() {
        // For P = Hom(-, C2), an element of P(C1) is a morphism C1 → C2.
        // yoneda_apply(f: C0 → C1, m: C1 → C2) should give m ∘ f : C0 → C2.
        let m = ChainCat::<3>::morphism::<C1, C2>().unwrap();
        let f = ChainCat::<3>::morphism::<C0, C1>().unwrap();
        let result = yoneda_apply::<Representable<C2>, ChainCat<3>, C0, C1>(f, m);
        assert_eq!((result.from(), result.to()), (0, 2));
    }

    #[test]
    fn yoneda_extract_from_naturality() {
        // Given a natural transformation represented by its action, extract
        // the generating element. For the constant presheaf, the action is
        // constant: every f maps to the same value.
        let x: i32 = 42;
        let id = ChainCat::<3>::identity::<C1>();
        let action = |_f: <ChainCat<3> as SmallCategory>::Mor<C1, C1>| -> i32 { x };
        let recovered: i32 =
            yoneda_extract::<ConstantPresheaf<i32>, ChainCat<3>, C1, _>(id, action);
        assert_eq!(recovered, 42);
    }
}
