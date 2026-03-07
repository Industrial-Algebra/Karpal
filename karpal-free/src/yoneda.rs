#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;

use core::marker::PhantomData;

use karpal_core::functor::Functor;
use karpal_core::hkt::HKT;

/// Private dyn-safe trait for Yoneda's inner representation.
trait YonedaLower<F: HKT + Functor + 'static, A: 'static> {
    fn lower(self: Box<Self>) -> F::Of<A>;
}

/// Leaf node: wraps an `F<A>` directly (created by `lift`).
struct YonedaLift<F: HKT + Functor + 'static, A: 'static> {
    value: F::Of<A>,
}

impl<F: HKT + Functor + 'static, A: 'static> YonedaLower<F, A> for YonedaLift<F, A>
where
    F::Of<A>: 'static,
{
    fn lower(self: Box<Self>) -> F::Of<A> {
        self.value
    }
}

/// Map layer: wraps an inner `YonedaLower<F, Src>` with a function `Src -> A`.
struct YonedaMap<F: HKT + Functor + 'static, Src: 'static, A: 'static> {
    inner: Box<dyn YonedaLower<F, Src>>,
    transform: Box<dyn Fn(Src) -> A>,
}

impl<F: HKT + Functor + 'static, Src: 'static, A: 'static> YonedaLower<F, A>
    for YonedaMap<F, Src, A>
{
    fn lower(self: Box<Self>) -> F::Of<A> {
        F::fmap(self.inner.lower(), self.transform)
    }
}

/// Yoneda — the Yoneda lemma as a data type. Wraps a value `F<A>` in
/// CPS form, enabling O(1) map composition.
///
/// Each `fmap` composes into the stored continuation without touching `F`,
/// so chaining N maps defers all computation until `lower()` is called.
///
/// Unlike Coyoneda, `lift` requires `F: Functor`. The benefit of Yoneda
/// is map fusion: chained maps are composed before being applied.
///
/// Note: Due to Rust's GAT limitations, `YonedaF` does not implement
/// the `HKT` or `Functor` traits. Use the inherent `fmap` method instead.
pub struct Yoneda<F: HKT + Functor + 'static, A: 'static> {
    inner: Box<dyn YonedaLower<F, A>>,
}

impl<F: HKT + Functor + 'static, A: Clone + 'static> Yoneda<F, A>
where
    F::Of<A>: Clone + 'static,
{
    /// Lift a value `F<A>` into `Yoneda<F, A>`. Requires `F: Functor`.
    pub fn lift(fa: F::Of<A>) -> Self {
        Yoneda {
            inner: Box::new(YonedaLift::<F, A> { value: fa }),
        }
    }
}

impl<F: HKT + Functor + 'static, A: 'static> Yoneda<F, A> {
    /// Lower back to `F<A>` by applying the accumulated transformations.
    pub fn lower(self) -> F::Of<A> {
        self.inner.lower()
    }

    /// Map a function over this Yoneda. The function is composed into the
    /// stored continuation, deferring application until `lower()`.
    pub fn fmap<B: 'static>(self, f: impl Fn(A) -> B + 'static) -> Yoneda<F, B> {
        Yoneda {
            inner: Box::new(YonedaMap {
                inner: self.inner,
                transform: Box::new(f),
            }),
        }
    }
}

/// Marker type for `Yoneda<F, _>`.
///
/// Note: Cannot implement `HKT` or `Functor` due to Rust's GAT limitations
/// (`type Of<T>` cannot add `T: 'static` in impl). Use `Yoneda::fmap` directly.
pub struct YonedaF<F: HKT + Functor + 'static>(PhantomData<F>);

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::{OptionF, VecF};

    #[test]
    fn lift_lower_roundtrip_option() {
        let result = Yoneda::<OptionF, i32>::lift(Some(42)).lower();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn lift_lower_roundtrip_vec() {
        let result = Yoneda::<VecF, i32>::lift(vec![1, 2, 3]).lower();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn fmap_then_lower() {
        let result = Yoneda::<OptionF, i32>::lift(Some(2))
            .fmap(|x| x * 3)
            .lower();
        assert_eq!(result, Some(6));
    }

    #[test]
    fn multiple_fmaps() {
        let result = Yoneda::<OptionF, i32>::lift(Some(1))
            .fmap(|x| x + 1)
            .fmap(|x| x * 10)
            .fmap(|x| x + 5)
            .lower();
        assert_eq!(result, Some(25)); // (1+1)*10+5
    }

    #[test]
    fn fmap_none() {
        let result = Yoneda::<OptionF, i32>::lift(None::<i32>)
            .fmap(|x: i32| x * 2)
            .lower();
        assert_eq!(result, None);
    }

    #[test]
    fn fmap_type_change() {
        let result = Yoneda::<OptionF, i32>::lift(Some(42))
            .fmap(|x| format!("val={}", x))
            .lower();
        assert_eq!(result, Some("val=42".to_string()));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::hkt::OptionF;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn yoneda_identity(x in any::<Option<i32>>()) {
            let result = Yoneda::<OptionF, i32>::lift(x).fmap(|a: i32| a).lower();
            prop_assert_eq!(result, x);
        }

        #[test]
        fn yoneda_composition(x in any::<Option<i32>>()) {
            let f = |a: i32| a.wrapping_add(1);
            let g = |a: i32| a.wrapping_mul(2);

            let left = Yoneda::<OptionF, i32>::lift(x).fmap(move |a: i32| g(f(a))).lower();
            let right = Yoneda::<OptionF, i32>::lift(x).fmap(f).fmap(g).lower();
            prop_assert_eq!(left, right);
        }
    }
}
