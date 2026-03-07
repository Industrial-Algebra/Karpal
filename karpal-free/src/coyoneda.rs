#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;

use core::marker::PhantomData;

use karpal_core::functor::Functor;
use karpal_core::hkt::HKT;

/// Private dyn-safe trait for the existential encoding of Coyoneda.
trait CoyonedaLower<F: HKT + 'static, A: 'static> {
    fn lower(self: Box<Self>) -> F::Of<A>
    where
        F: Functor;
}

/// Leaf node: wraps an `F::Of<A>` directly.
struct CoyonedaLift<F: HKT + 'static, A: 'static> {
    value: F::Of<A>,
}

impl<F: HKT + 'static, A: 'static> CoyonedaLower<F, A> for CoyonedaLift<F, A>
where
    F::Of<A>: 'static,
{
    fn lower(self: Box<Self>) -> F::Of<A>
    where
        F: Functor,
    {
        self.value
    }
}

/// Map layer: wraps an inner `CoyonedaLower<F, Src>` with a function `Src -> A`.
struct CoyonedaMap<F: HKT + 'static, Src: 'static, A: 'static> {
    inner: Box<dyn CoyonedaLower<F, Src>>,
    transform: Box<dyn Fn(Src) -> A>,
}

impl<F: HKT + 'static, Src: 'static, A: 'static> CoyonedaLower<F, A> for CoyonedaMap<F, Src, A> {
    fn lower(self: Box<Self>) -> F::Of<A>
    where
        F: Functor,
    {
        F::fmap(self.inner.lower(), self.transform)
    }
}

/// Coyoneda — the free functor. Makes any type constructor into a Functor
/// by deferring `fmap` as function composition, applying it only when lowered.
///
/// `Coyoneda<F, A>` is isomorphic to `F<A>` when `F: Functor`, but the
/// `fmap` operation requires no `Functor` bound on `F` — it simply layers
/// the transformation, deferring application until `lower()`.
///
/// Note: Due to Rust's GAT limitations, `CoyonedaF` does not implement
/// the `HKT` or `Functor` traits. Use the inherent `fmap` method instead.
pub struct Coyoneda<F: HKT + 'static, A: 'static> {
    inner: Box<dyn CoyonedaLower<F, A>>,
}

impl<F: HKT + 'static, A: 'static> Coyoneda<F, A>
where
    F::Of<A>: 'static,
{
    /// Lift a value `F<A>` into `Coyoneda<F, A>`.
    pub fn lift(fa: F::Of<A>) -> Self {
        Coyoneda {
            inner: Box::new(CoyonedaLift::<F, A> { value: fa }),
        }
    }
}

impl<F: HKT + 'static, A: 'static> Coyoneda<F, A> {
    /// Apply the deferred transformations, producing `F<A>`.
    /// This is the only operation that requires `F: Functor`.
    pub fn lower(self) -> F::Of<A>
    where
        F: Functor,
    {
        self.inner.lower()
    }

    /// Map a function over this Coyoneda. No `Functor` bound required —
    /// the function is stored and applied when `lower()` is called.
    pub fn fmap<B: 'static>(self, f: impl Fn(A) -> B + 'static) -> Coyoneda<F, B> {
        Coyoneda {
            inner: Box::new(CoyonedaMap {
                inner: self.inner,
                transform: Box::new(f),
            }),
        }
    }
}

/// Marker type for `Coyoneda<F, _>`.
///
/// Note: Cannot implement `HKT` or `Functor` due to Rust's GAT limitations
/// (`type Of<T>` cannot add `T: 'static` in impl). Use `Coyoneda::fmap` directly.
pub struct CoyonedaF<F: HKT + 'static>(PhantomData<F>);

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::{OptionF, VecF};

    #[test]
    fn lift_lower_roundtrip_option() {
        let result = Coyoneda::<OptionF, i32>::lift(Some(42)).lower();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn lift_lower_roundtrip_vec() {
        let result = Coyoneda::<VecF, i32>::lift(vec![1, 2, 3]).lower();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn fmap_then_lower() {
        let result = Coyoneda::<OptionF, i32>::lift(Some(2))
            .fmap(|x| x * 3)
            .lower();
        assert_eq!(result, Some(6));
    }

    #[test]
    fn multiple_fmaps() {
        let result = Coyoneda::<OptionF, i32>::lift(Some(1))
            .fmap(|x| x + 1)
            .fmap(|x| x * 10)
            .fmap(|x| x + 5)
            .lower();
        assert_eq!(result, Some(25)); // (1+1)*10+5
    }

    #[test]
    fn fmap_none() {
        let result = Coyoneda::<OptionF, i32>::lift(None).fmap(|x| x * 2).lower();
        assert_eq!(result, None);
    }

    #[test]
    fn fmap_without_functor_bound() {
        // Demonstrates that fmap works without F: Functor
        // We use OptionF here but the fmap call itself doesn't require Functor
        let co = Coyoneda::<OptionF, i32>::lift(Some(10));
        let co2 = co.fmap(|x| x + 5);
        let co3 = co2.fmap(|x| format!("value: {}", x));
        // Only lower() requires Functor
        let result = co3.lower();
        assert_eq!(result, Some("value: 15".to_string()));
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
            let result = Coyoneda::<OptionF, i32>::lift(x).fmap(|a| a).lower();
            prop_assert_eq!(result, x);
        }

        #[test]
        fn coyoneda_composition(x in any::<Option<i32>>()) {
            let f = |a: i32| a.wrapping_add(1);
            let g = |a: i32| a.wrapping_mul(2);

            let left = Coyoneda::<OptionF, i32>::lift(x).fmap(move |a| g(f(a))).lower();
            let right = Coyoneda::<OptionF, i32>::lift(x).fmap(f).fmap(g).lower();
            prop_assert_eq!(left, right);
        }
    }
}
