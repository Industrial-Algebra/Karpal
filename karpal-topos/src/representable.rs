// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! The representable presheaf `Hom_C(-, c)`.

use core::marker::PhantomData;

use crate::presheaf::Presheaf;
use crate::small_category::SmallCategory;

/// The representable presheaf `Hom_C(-, c)`.
///
/// For each object `d`, `At<d> = Hom_C(d, c)` (the morphisms `d → c`).
/// Restriction along `f: Dom → Cod` is precomposition:
/// `Hom(Cod, c) → Hom(Dom, c)` sends `m: Cod → c` to `m ∘ f: Dom → c`.
///
/// This is the anchor of the Yoneda lemma: natural transformations
/// `Hom(-, c) ⇒ P` are in bijection with elements of `P(c)`.
pub struct Representable<RepObj>(PhantomData<fn() -> RepObj>);

impl<C: SmallCategory, RepObj> Presheaf<C> for Representable<RepObj> {
    type At<Obj> = C::Mor<Obj, RepObj>;

    fn restrict<Dom, Cod>(f: C::Mor<Dom, Cod>, m: C::Mor<Cod, RepObj>) -> C::Mor<Dom, RepObj> {
        // Precompose: m ∘ f : Dom → RepObj.
        C::compose(m, f)
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;
    use crate::small_category::{ChainCat, ChainMor, ChainObj};

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
    fn representable_at_is_hom() {
        // Hom(C0, C2): At<C0> for Representable<C2> = ChainMor<C0, C2>.
        let f: <Representable<C2> as Presheaf<ChainCat<2>>>::At<C0> =
            ChainCat::<2>::morphism::<C0, C2>().unwrap();
        assert_eq!((f.from(), f.to()), (0, 2));
    }

    #[test]
    fn representable_restrict_precomposes() {
        // m: C1 → C2 (an element of Hom(C1, C2) = At<C1> for Representable<C2>).
        let m = ChainCat::<2>::morphism::<C1, C2>().unwrap();
        // f: C0 → C1 (the restriction morphism).
        let f = ChainCat::<2>::morphism::<C0, C1>().unwrap();
        // restrict(f, m) should be m ∘ f : C0 → C2.
        let composed: ChainMor<C0, C2> =
            <Representable<C2> as Presheaf<ChainCat<2>>>::restrict(f, m);
        assert_eq!((composed.from(), composed.to()), (0, 2));
    }

    #[test]
    fn representable_identity_law() {
        // restrict(id_C1, m) == m for m ∈ Hom(C1, C2).
        let id = ChainCat::<2>::identity::<C1>();
        let m = ChainCat::<2>::morphism::<C1, C2>().unwrap();
        let result: ChainMor<C1, C2> =
            <Representable<C2> as Presheaf<ChainCat<2>>>::restrict(id, m);
        assert_eq!((result.from(), result.to()), (1, 2));
    }
}
