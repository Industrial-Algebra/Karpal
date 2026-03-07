#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;

#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::rc::Rc;

use core::marker::PhantomData;

use karpal_core::applicative::Applicative;
use karpal_core::hkt::HKT;
use karpal_core::natural::NaturalTransformation;

/// Day Convolution — combines two functors `F` and `G` into a single
/// computation.
///
/// `Day<F, G, A, B, C> ≅ (F B, G C, B → C → A)`
///
/// Stores an `F` value, a `G` value, and a combining function.
/// The intermediate types `B` and `C` are visible in the type signature
/// (Rust lacks first-class existential types, same as `Lan` and `Coyoneda`).
///
/// # Key properties
///
/// - **Functor in A**: `fmap` composes onto the combining function,
///   no bounds on F or G needed.
/// - **Interpretation**: `run_day` uses two natural transformations
///   (one for F, one for G) to interpret into a target Applicative.
pub struct Day<F: HKT, G: HKT, A, B, C> {
    f_val: F::Of<B>,
    g_val: G::Of<C>,
    combine: Box<dyn Fn(B, C) -> A>,
    _marker: PhantomData<(F, G)>,
}

impl<F: HKT + 'static, G: HKT + 'static, A: 'static, B: Clone + 'static, C: Clone + 'static>
    Day<F, G, A, B, C>
{
    /// Construct a Day from two functor values and a combining function.
    pub fn new(f_val: F::Of<B>, g_val: G::Of<C>, combine: impl Fn(B, C) -> A + 'static) -> Self {
        Day {
            f_val,
            g_val,
            combine: Box::new(combine),
            _marker: PhantomData,
        }
    }

    /// Map a function over the result. No bounds on F or G required.
    pub fn fmap<D: 'static>(self, f: impl Fn(A) -> D + 'static) -> Day<F, G, D, B, C> {
        let old_combine = self.combine;
        Day {
            f_val: self.f_val,
            g_val: self.g_val,
            combine: Box::new(move |b, c| f(old_combine(b, c))),
            _marker: PhantomData,
        }
    }

    /// Interpret this Day value into a target applicative `M` using
    /// two natural transformations: `NF: F ~> M` and `NG: G ~> M`.
    pub fn run_day<M, NF, NG>(self) -> M::Of<A>
    where
        M: Applicative,
        NF: NaturalTransformation<F, M>,
        NG: NaturalTransformation<G, M>,
    {
        let m_b: M::Of<B> = NF::transform(self.f_val);
        let m_c: M::Of<C> = NG::transform(self.g_val);
        let combine = Rc::new(self.combine);
        let m_curried: M::Of<Box<dyn Fn(C) -> A>> =
            M::fmap(m_b, move |b: B| -> Box<dyn Fn(C) -> A> {
                let combine = combine.clone();
                Box::new(move |c: C| combine(b.clone(), c))
            });
        M::ap(m_curried, m_c)
    }
}

/// Marker type for `Day<F, G, _, B, C>`.
///
/// Note: Cannot implement `HKT` or `Functor` due to Rust's GAT limitations
/// (extra type parameters F, G, B, C cannot be threaded through `type Of<T>`).
/// Use `Day::fmap` directly.
pub struct DayF<F: HKT + 'static, G: HKT + 'static>(PhantomData<(F, G)>);

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::OptionF;

    struct OptionId;
    impl NaturalTransformation<OptionF, OptionF> for OptionId {
        fn transform<A>(fa: Option<A>) -> Option<A> {
            fa
        }
    }

    #[test]
    fn new_and_run_day() {
        let day = Day::<OptionF, OptionF, i32, i32, i32>::new(Some(3), Some(4), |a, b| a * b);
        let result = day.run_day::<OptionF, OptionId, OptionId>();
        assert_eq!(result, Some(12));
    }

    #[test]
    fn run_day_none_left() {
        let day = Day::<OptionF, OptionF, i32, i32, i32>::new(None::<i32>, Some(4), |a, b| a + b);
        let result = day.run_day::<OptionF, OptionId, OptionId>();
        assert_eq!(result, None);
    }

    #[test]
    fn run_day_none_right() {
        let day = Day::<OptionF, OptionF, i32, i32, i32>::new(Some(3), None::<i32>, |a, b| a + b);
        let result = day.run_day::<OptionF, OptionId, OptionId>();
        assert_eq!(result, None);
    }

    #[test]
    fn fmap_day() {
        let day = Day::<OptionF, OptionF, i32, i32, i32>::new(Some(2), Some(5), |a, b| a + b);
        let mapped = day.fmap(|x| x * 3);
        let result = mapped.run_day::<OptionF, OptionId, OptionId>();
        assert_eq!(result, Some(21)); // (2+5)*3
    }

    #[test]
    fn fmap_identity() {
        let day = Day::<OptionF, OptionF, i32, i32, i32>::new(Some(7), Some(3), |a, b| a - b);
        let mapped = day.fmap(|x| x);
        let result = mapped.run_day::<OptionF, OptionId, OptionId>();
        assert_eq!(result, Some(4));
    }

    #[test]
    fn fmap_composition() {
        let f = |x: i32| x + 1;
        let g = |x: i32| x * 2;

        let left = Day::<OptionF, OptionF, i32, i32, i32>::new(Some(3), Some(4), |a, b| a + b)
            .fmap(move |a| g(f(a)));
        let right = Day::<OptionF, OptionF, i32, i32, i32>::new(Some(3), Some(4), |a, b| a + b)
            .fmap(f)
            .fmap(g);

        assert_eq!(
            left.run_day::<OptionF, OptionId, OptionId>(),
            right.run_day::<OptionF, OptionId, OptionId>()
        );
    }

    #[test]
    fn type_changing_combine() {
        let day = Day::<OptionF, OptionF, String, i32, &str>::new(
            Some(42),
            Some("hello"),
            |n: i32, s: &str| format!("{s}={n}"),
        );
        let result = day.run_day::<OptionF, OptionId, OptionId>();
        assert_eq!(result, Some("hello=42".to_string()));
    }

    #[test]
    fn multiple_fmaps() {
        let day = Day::<OptionF, OptionF, i32, i32, i32>::new(Some(1), Some(2), |a, b| a + b)
            .fmap(|x| x * 10)
            .fmap(|x| x + 5);
        let result = day.run_day::<OptionF, OptionId, OptionId>();
        assert_eq!(result, Some(35)); // (1+2)*10+5
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::hkt::OptionF;
    use proptest::prelude::*;

    struct OptionId;
    impl NaturalTransformation<OptionF, OptionF> for OptionId {
        fn transform<A>(fa: Option<A>) -> Option<A> {
            fa
        }
    }

    proptest! {
        // Functor identity: fmap(id).run_day() == run_day()
        #[test]
        fn functor_identity(a in any::<i32>(), b in any::<i32>()) {
            let original = Day::<OptionF, OptionF, i32, i32, i32>::new(
                Some(a), Some(b), |x, y| x.wrapping_add(y)
            ).run_day::<OptionF, OptionId, OptionId>();
            let mapped = Day::<OptionF, OptionF, i32, i32, i32>::new(
                Some(a), Some(b), |x, y| x.wrapping_add(y)
            ).fmap(|x| x).run_day::<OptionF, OptionId, OptionId>();
            prop_assert_eq!(original, mapped);
        }

        // Functor composition: fmap(g . f) == fmap(f) . fmap(g)
        #[test]
        fn functor_composition(a in any::<i32>(), b in any::<i32>()) {
            let f = |x: i32| x.wrapping_add(1);
            let g = |x: i32| x.wrapping_mul(2);

            let left = Day::<OptionF, OptionF, i32, i32, i32>::new(
                Some(a), Some(b), |x, y| x.wrapping_add(y)
            ).fmap(move |x| g(f(x))).run_day::<OptionF, OptionId, OptionId>();
            let right = Day::<OptionF, OptionF, i32, i32, i32>::new(
                Some(a), Some(b), |x, y| x.wrapping_add(y)
            ).fmap(f).fmap(g).run_day::<OptionF, OptionId, OptionId>();
            prop_assert_eq!(left, right);
        }
    }
}
