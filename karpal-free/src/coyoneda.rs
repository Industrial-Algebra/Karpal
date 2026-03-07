#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;

use core::marker::PhantomData;

use karpal_core::functor::Functor;
use karpal_core::hkt::HKT;

/// Coyoneda — the free functor.
///
/// `Coyoneda<F, A, B>` stores a value `F::Of<B>` together with a function
/// `B → A`. The `fmap` operation composes functions without touching `F`;
/// only `lower()` applies `F::fmap` once at the end.
///
/// The type parameter `B` is the original ("base") type from `lift`.
/// After `N` calls to `fmap`, `B` stays the same — only `A` changes.
///
/// ```text
/// lift(fb: F<B>) → Coyoneda<F, B, B>     // f = identity
/// .fmap(g: B→C)  → Coyoneda<F, C, B>     // f = g
/// .fmap(h: C→D)  → Coyoneda<F, D, B>     // f = h ∘ g
/// .lower()        → F<D>                  // one F::fmap(fb, h ∘ g)
/// ```
pub struct Coyoneda<F: HKT, A, B> {
    f: Box<dyn Fn(B) -> A>,
    fb: F::Of<B>,
    _marker: PhantomData<F>,
}

impl<F: HKT, A: 'static> Coyoneda<F, A, A> {
    /// Lift a value `F<A>` into `Coyoneda<F, A, A>`.
    ///
    /// No `Functor` bound required.
    pub fn lift(fa: F::Of<A>) -> Self {
        Coyoneda {
            f: Box::new(|a| a),
            fb: fa,
            _marker: PhantomData,
        }
    }
}

impl<F: HKT, A: 'static, B: 'static> Coyoneda<F, A, B> {
    /// Map a function over this Coyoneda. No `Functor` bound required —
    /// the function is composed with the stored transform and applied
    /// when `lower()` is called.
    pub fn fmap<C: 'static>(self, g: impl Fn(A) -> C + 'static) -> Coyoneda<F, C, B> {
        let old_f = self.f;
        Coyoneda {
            f: Box::new(move |b| g(old_f(b))),
            fb: self.fb,
            _marker: PhantomData,
        }
    }

    /// Apply the stored function via `F::fmap`, producing `F<A>`.
    /// This is the only operation that requires `F: Functor`.
    pub fn lower(self) -> F::Of<A>
    where
        F: Functor,
    {
        F::fmap(self.fb, self.f)
    }
}

/// Marker type for `Coyoneda<F, _, B>`.
///
/// Note: Cannot implement `HKT` or `Functor` because `Coyoneda` has
/// three type parameters and the `B` parameter is fixed at construction.
/// Use `Coyoneda::fmap` directly.
pub struct CoyonedaF<F: HKT>(PhantomData<F>);

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::{OptionF, VecF};

    #[test]
    fn lift_lower_roundtrip_option() {
        let result = Coyoneda::<OptionF, _, _>::lift(Some(42)).lower();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn lift_lower_roundtrip_vec() {
        let result = Coyoneda::<VecF, _, _>::lift(vec![1, 2, 3]).lower();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn fmap_then_lower() {
        let result = Coyoneda::<OptionF, _, _>::lift(Some(2))
            .fmap(|x| x * 3)
            .lower();
        assert_eq!(result, Some(6));
    }

    #[test]
    fn multiple_fmaps() {
        let result = Coyoneda::<OptionF, _, _>::lift(Some(1))
            .fmap(|x| x + 1)
            .fmap(|x| x * 10)
            .fmap(|x| x + 5)
            .lower();
        assert_eq!(result, Some(25)); // (1+1)*10+5
    }

    #[test]
    fn fmap_none() {
        let result = Coyoneda::<OptionF, _, _>::lift(None::<i32>)
            .fmap(|x| x * 2)
            .lower();
        assert_eq!(result, None);
    }

    #[test]
    fn fmap_without_functor_bound() {
        // fmap works without F: Functor
        let co = Coyoneda::<OptionF, _, _>::lift(Some(10));
        let co2 = co.fmap(|x| x + 5);
        let co3 = co2.fmap(|x| format!("value: {}", x));
        // Only lower() requires Functor
        let result = co3.lower();
        assert_eq!(result, Some("value: 15".to_string()));
    }

    #[test]
    fn fmap_changes_type() {
        let result = Coyoneda::<OptionF, _, _>::lift(Some(42))
            .fmap(|x: i32| x.to_string())
            .lower();
        assert_eq!(result, Some("42".to_string()));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::hkt::OptionF;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn coyoneda_identity(x in any::<Option<i32>>()) {
            let result = Coyoneda::<OptionF, _, _>::lift(x).fmap(|a| a).lower();
            prop_assert_eq!(result, x);
        }

        #[test]
        fn coyoneda_composition(x in any::<Option<i32>>()) {
            let f = |a: i32| a.wrapping_add(1);
            let g = |a: i32| a.wrapping_mul(2);

            let left = Coyoneda::<OptionF, _, _>::lift(x).fmap(move |a| g(f(a))).lower();
            let right = Coyoneda::<OptionF, _, _>::lift(x).fmap(f).fmap(g).lower();
            prop_assert_eq!(left, right);
        }
    }
}
