#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;

use core::marker::PhantomData;

use karpal_core::hkt::HKT;

/// Dyn-safe trait for the existential encoding of Density.
///
/// Hides the state type `S` in `∃S. (W S → A, W S)`.
trait DensityDyn<W: HKT + 'static, A: 'static> {
    /// Extract the value by applying the stored function to the source.
    fn extract_dyn(&self) -> A;
}

/// Concrete cell: stores `extract_fn: &W::Of<S> → A` and `source: W::Of<S>`.
#[allow(clippy::type_complexity)]
struct DensityCell<W: HKT + 'static, A: 'static, S: 'static> {
    extract_fn: Box<dyn Fn(&W::Of<S>) -> A>,
    source: W::Of<S>,
    _marker: PhantomData<W>,
}

impl<W: HKT + 'static, A: 'static, S: 'static> DensityDyn<W, A> for DensityCell<W, A, S> {
    fn extract_dyn(&self) -> A {
        (self.extract_fn)(&self.source)
    }
}

/// Map wrapper: composes a function on top of extract.
struct DensityMap<W: HKT + 'static, Src: 'static, A: 'static> {
    inner: Box<dyn DensityDyn<W, Src>>,
    transform: Box<dyn Fn(Src) -> A>,
}

impl<W: HKT + 'static, Src: 'static, A: 'static> DensityDyn<W, A> for DensityMap<W, Src, A> {
    fn extract_dyn(&self) -> A {
        (self.transform)(self.inner.extract_dyn())
    }
}

/// Density Comonad — the CPS dual of Codensity.
///
/// `Density<W, A> ≅ ∃S. (W S → A, W S)`
///
/// This is the left Kan extension of `W` along itself (`Lan W W A`),
/// specialised into a concrete type with existential state.
///
/// # Key properties
///
/// - **`extract`**: Requires no bounds on `W` — just applies the stored function.
/// - **`fmap`**: Composes onto the extract function, no bounds on `W`.
///
/// Note: Due to Rust's GAT limitations, `DensityF` does not implement
/// `HKT` or `Comonad`. Use inherent methods instead.
pub struct Density<W: HKT + 'static, A: 'static> {
    inner: Box<dyn DensityDyn<W, A>>,
}

impl<W: HKT + 'static, A: 'static> Density<W, A> {
    /// Construct a Density from a source value and an extract function.
    pub fn lift<S: 'static>(source: W::Of<S>, f: impl Fn(&W::Of<S>) -> A + 'static) -> Self
    where
        W::Of<S>: 'static,
    {
        Density {
            inner: Box::new(DensityCell {
                extract_fn: Box::new(f),
                source,
                _marker: PhantomData,
            }),
        }
    }

    /// Extract the value. No bounds on `W` required.
    pub fn extract(&self) -> A {
        self.inner.extract_dyn()
    }

    /// Map a function over the result. No bounds on `W` required.
    pub fn fmap<B: 'static>(self, f: impl Fn(A) -> B + 'static) -> Density<W, B> {
        Density {
            inner: Box::new(DensityMap {
                inner: self.inner,
                transform: Box::new(f),
            }),
        }
    }
}

/// Marker type for `Density<W, _>`.
///
/// Note: Cannot implement `HKT` or `Comonad` due to Rust's GAT limitations.
/// Use `Density::extract`, `Density::fmap` directly.
pub struct DensityF<W: HKT + 'static>(PhantomData<W>);

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::OptionF;

    #[test]
    fn lift_and_extract() {
        let d = Density::<OptionF, i32>::lift(Some(42), |opt| opt.unwrap());
        assert_eq!(d.extract(), 42);
    }

    #[test]
    fn lift_extract_none() {
        let d = Density::<OptionF, i32>::lift(None::<i32>, |_opt| 0);
        assert_eq!(d.extract(), 0);
    }

    #[test]
    fn fmap_density() {
        let d = Density::<OptionF, i32>::lift(Some(5), |opt| opt.unwrap()).fmap(|x| x * 3);
        assert_eq!(d.extract(), 15);
    }

    #[test]
    fn fmap_identity() {
        let d = Density::<OptionF, i32>::lift(Some(7), |opt| opt.unwrap()).fmap(|x| x);
        assert_eq!(d.extract(), 7);
    }

    #[test]
    fn fmap_composition() {
        let f = |x: i32| x + 1;
        let g = |x: i32| x * 2;

        let left = Density::<OptionF, i32>::lift(Some(5), |opt| opt.unwrap())
            .fmap(move |a| g(f(a)))
            .extract();
        let right = Density::<OptionF, i32>::lift(Some(5), |opt| opt.unwrap())
            .fmap(f)
            .fmap(g)
            .extract();
        assert_eq!(left, right);
    }

    #[test]
    fn fmap_changes_type() {
        let d = Density::<OptionF, i32>::lift(Some(42), |opt| opt.unwrap())
            .fmap(|x| format!("val={x}"));
        assert_eq!(d.extract(), "val=42");
    }

    #[test]
    fn multiple_fmaps() {
        let d = Density::<OptionF, i32>::lift(Some(1), |opt| opt.unwrap())
            .fmap(|x| x + 1)
            .fmap(|x| x * 10)
            .fmap(|x| x + 5);
        assert_eq!(d.extract(), 25); // (1+1)*10+5
    }

    #[test]
    fn extract_multiple_times() {
        let d = Density::<OptionF, i32>::lift(Some(99), |opt| opt.unwrap());
        assert_eq!(d.extract(), 99);
        assert_eq!(d.extract(), 99);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::hkt::OptionF;
    use proptest::prelude::*;

    proptest! {
        // Functor identity: fmap(id).extract() == extract()
        #[test]
        fn functor_identity(x in any::<i32>()) {
            let original = Density::<OptionF, i32>::lift(Some(x), |opt| opt.unwrap()).extract();
            let mapped = Density::<OptionF, i32>::lift(Some(x), |opt| opt.unwrap())
                .fmap(|a| a)
                .extract();
            prop_assert_eq!(original, mapped);
        }

        // Functor composition: fmap(g . f) == fmap(f) . fmap(g)
        #[test]
        fn functor_composition(x in any::<i32>()) {
            let f = |a: i32| a.wrapping_add(1);
            let g = |a: i32| a.wrapping_mul(2);

            let left = Density::<OptionF, i32>::lift(Some(x), |opt| opt.unwrap())
                .fmap(move |a| g(f(a)))
                .extract();
            let right = Density::<OptionF, i32>::lift(Some(x), |opt| opt.unwrap())
                .fmap(f)
                .fmap(g)
                .extract();
            prop_assert_eq!(left, right);
        }
    }
}
