use core::marker::PhantomData;

use karpal_core::applicative::Applicative;
use karpal_core::chain::Chain;
use karpal_core::functor::Functor;
use karpal_core::hkt::{HKT, HKT2};
use karpal_core::plus::Plus;

use crate::arrow::Arrow;
use crate::arrow_apply::ArrowApply;
use crate::arrow_choice::ArrowChoice;
use crate::arrow_plus::ArrowPlus;
use crate::arrow_zero::ArrowZero;
use crate::category::Category;
use crate::semigroupoid::Semigroupoid;

/// Kleisli arrow: `A -> M<B>` for a Monad `M`.
///
/// `KleisliF<M>` is a two-parameter type constructor where
/// `P<A, B> = Box<dyn Fn(A) -> M::Of<B>>`.
///
/// Implements the full Arrow hierarchy when `M: Chain + Applicative`.
pub struct KleisliF<M: HKT>(PhantomData<M>);

impl<M: HKT> HKT2 for KleisliF<M> {
    type P<A, B> = Box<dyn Fn(A) -> M::Of<B>>;
}

impl<M: Chain + Applicative> Semigroupoid for KleisliF<M>
where
    M: 'static,
{
    fn compose<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        f: Box<dyn Fn(B) -> M::Of<C>>,
        g: Box<dyn Fn(A) -> M::Of<B>>,
    ) -> Box<dyn Fn(A) -> M::Of<C>> {
        Box::new(move |a| M::chain(g(a), &f))
    }
}

impl<M: Chain + Applicative> Category for KleisliF<M>
where
    M: 'static,
{
    fn id<A: Clone + 'static>() -> Box<dyn Fn(A) -> M::Of<A>> {
        Box::new(|a| M::pure(a))
    }
}

impl<M: Chain + Applicative + Functor> Arrow for KleisliF<M>
where
    M: 'static,
{
    fn arr<A: Clone + 'static, B: Clone + 'static>(
        f: impl Fn(A) -> B + 'static,
    ) -> Box<dyn Fn(A) -> M::Of<B>> {
        Box::new(move |a| M::pure(f(a)))
    }

    fn first<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        pab: Box<dyn Fn(A) -> M::Of<B>>,
    ) -> Box<dyn Fn((A, C)) -> M::Of<(B, C)>> {
        Box::new(move |(a, c): (A, C)| M::fmap(pab(a), move |b| (b, c.clone())))
    }

    fn second<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        pab: Box<dyn Fn(A) -> M::Of<B>>,
    ) -> Box<dyn Fn((C, A)) -> M::Of<(C, B)>> {
        Box::new(move |(c, a): (C, A)| M::fmap(pab(a), move |b| (c.clone(), b)))
    }
}

impl<M: Chain + Applicative + Functor> ArrowChoice for KleisliF<M>
where
    M: 'static,
{
    fn left<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        pab: Box<dyn Fn(A) -> M::Of<B>>,
    ) -> Box<dyn Fn(Result<A, C>) -> M::Of<Result<B, C>>> {
        Box::new(move |r| match r {
            Ok(a) => M::fmap(pab(a), Ok),
            Err(c) => M::pure(Err(c)),
        })
    }

    fn right<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        pab: Box<dyn Fn(A) -> M::Of<B>>,
    ) -> Box<dyn Fn(Result<C, A>) -> M::Of<Result<C, B>>> {
        Box::new(move |r| match r {
            Ok(c) => M::pure(Ok(c)),
            Err(a) => M::fmap(pab(a), Err),
        })
    }
}

impl<M: Chain + Applicative + Functor> ArrowApply for KleisliF<M>
where
    M: 'static,
{
    fn app<A: Clone + 'static, B: Clone + 'static>()
    -> Box<dyn Fn((Box<dyn Fn(A) -> M::Of<B>>, A)) -> M::Of<B>> {
        Box::new(|(f, a)| f(a))
    }
}

impl<M: Chain + Applicative + Functor + Plus> ArrowZero for KleisliF<M>
where
    M: 'static,
{
    fn zero_arrow<A: Clone + 'static, B: Clone + 'static>() -> Box<dyn Fn(A) -> M::Of<B>> {
        Box::new(|_| M::zero())
    }
}

impl<M: Chain + Applicative + Functor + Plus> ArrowPlus for KleisliF<M>
where
    M: 'static,
{
    fn plus<A: Clone + 'static, B: Clone + 'static>(
        f: Box<dyn Fn(A) -> M::Of<B>>,
        g: Box<dyn Fn(A) -> M::Of<B>>,
    ) -> Box<dyn Fn(A) -> M::Of<B>> {
        Box::new(move |a: A| {
            let fa = f(a.clone());
            let ga = g(a);
            M::alt(fa, ga)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::OptionF;

    type KOpt = KleisliF<OptionF>;

    #[test]
    fn kleisli_compose() {
        let f: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|x| Some(x + 1));
        let g: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|x| Some(x * 2));
        let fg = KOpt::compose(f, g);
        assert_eq!(fg(3), Some(7)); // (3 * 2) + 1
    }

    #[test]
    fn kleisli_compose_short_circuits() {
        let f: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|x| Some(x + 1));
        let g: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|_| None);
        let fg = KOpt::compose(f, g);
        assert_eq!(fg(3), None);
    }

    #[test]
    fn kleisli_id() {
        let id = KOpt::id::<i32>();
        assert_eq!(id(42), Some(42));
    }

    #[test]
    fn kleisli_arr() {
        let f = KOpt::arr(|x: i32| x.to_string());
        assert_eq!(f(42), Some("42".to_string()));
    }

    #[test]
    fn kleisli_first() {
        let f: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|x| Some(x * 2));
        let first = KOpt::first::<i32, i32, &str>(f);
        assert_eq!(first((5, "hi")), Some((10, "hi")));
    }

    #[test]
    fn kleisli_first_none() {
        let f: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|_| None);
        let first = KOpt::first::<i32, i32, &str>(f);
        assert_eq!(first((5, "hi")), None);
    }

    #[test]
    fn kleisli_second() {
        let f: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|x| Some(x * 2));
        let second = KOpt::second::<i32, i32, &str>(f);
        assert_eq!(second(("hi", 5)), Some(("hi", 10)));
    }

    #[test]
    fn kleisli_left() {
        let f: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|x| Some(x * 2));
        let left = KOpt::left::<i32, i32, &str>(f);
        assert_eq!(left(Ok(5)), Some(Ok(10)));
        assert_eq!(left(Err("nope")), Some(Err("nope")));
    }

    #[test]
    fn kleisli_left_none() {
        let f: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|_| None);
        let left = KOpt::left::<i32, i32, &str>(f);
        assert_eq!(left(Ok(5)), None);
    }

    #[test]
    fn kleisli_app() {
        let app = KOpt::app::<i32, i32>();
        let double: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|x| Some(x * 2));
        assert_eq!(app((double, 5)), Some(10));
    }

    #[test]
    fn kleisli_zero_arrow() {
        let z = KOpt::zero_arrow::<i32, i32>();
        assert_eq!(z(42), None);
    }

    #[test]
    fn kleisli_plus() {
        let f: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|_| None);
        let g: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|x| Some(x * 2));
        let fg = KOpt::plus(f, g);
        assert_eq!(fg(5), Some(10));
    }

    #[test]
    fn kleisli_plus_first_wins() {
        let f: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|x| Some(x + 1));
        let g: Box<dyn Fn(i32) -> Option<i32>> = Box::new(|x| Some(x * 2));
        let fg = KOpt::plus(f, g);
        assert_eq!(fg(5), Some(6)); // first succeeds
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::hkt::OptionF;
    use proptest::prelude::*;

    type KOpt = KleisliF<OptionF>;

    proptest! {
        // Semigroupoid associativity
        #[test]
        fn associativity(x in any::<i16>()) {
            let f = || -> Box<dyn Fn(i16) -> Option<i16>> { Box::new(|a| Some(a.wrapping_add(1))) };
            let g = || -> Box<dyn Fn(i16) -> Option<i16>> { Box::new(|a| Some(a.wrapping_mul(2))) };
            let h = || -> Box<dyn Fn(i16) -> Option<i16>> { Box::new(|a| Some(a.wrapping_sub(3))) };

            let left = KOpt::compose(f(), KOpt::compose(g(), h()));
            let right = KOpt::compose(KOpt::compose(f(), g()), h());
            prop_assert_eq!(left(x), right(x));
        }

        // Category left identity
        #[test]
        fn left_identity(x in any::<i16>()) {
            let f = || -> Box<dyn Fn(i16) -> Option<i16>> { Box::new(|a| Some(a.wrapping_mul(2))) };
            let left = KOpt::compose(KOpt::id(), f());
            let right = f();
            prop_assert_eq!(left(x), right(x));
        }

        // Category right identity
        #[test]
        fn right_identity(x in any::<i16>()) {
            let f = || -> Box<dyn Fn(i16) -> Option<i16>> { Box::new(|a| Some(a.wrapping_mul(2))) };
            let left = KOpt::compose(f(), KOpt::id());
            let right = f();
            prop_assert_eq!(left(x), right(x));
        }

        // Arrow: arr(id) == id()
        #[test]
        fn arr_id(x in any::<i16>()) {
            let left = KOpt::arr(|a: i16| a);
            let right = KOpt::id::<i16>();
            prop_assert_eq!(left(x), right(x));
        }

        // ArrowZero left absorption: compose(zero_arrow(), f) == zero_arrow()
        #[test]
        fn zero_left_absorption(x in any::<i16>()) {
            let f = || -> Box<dyn Fn(i16) -> Option<i16>> { Box::new(|a| Some(a.wrapping_mul(2))) };
            let left = KOpt::compose(KOpt::zero_arrow::<i16, i16>(), f());
            let right = KOpt::zero_arrow::<i16, i16>();
            prop_assert_eq!(left(x), right(x));
        }

        // ArrowPlus left identity: plus(zero_arrow(), f) == f
        #[test]
        fn plus_left_identity(x in any::<i16>()) {
            let f = || -> Box<dyn Fn(i16) -> Option<i16>> { Box::new(|a| Some(a.wrapping_mul(2))) };
            let left = KOpt::plus(KOpt::zero_arrow(), f());
            let right = f();
            prop_assert_eq!(left(x), right(x));
        }

        // ArrowPlus right identity: plus(f, zero_arrow()) == f
        #[test]
        fn plus_right_identity(x in any::<i16>()) {
            let f = || -> Box<dyn Fn(i16) -> Option<i16>> { Box::new(|a| Some(a.wrapping_mul(2))) };
            let left = KOpt::plus(f(), KOpt::zero_arrow());
            let right = f();
            prop_assert_eq!(left(x), right(x));
        }
    }
}
