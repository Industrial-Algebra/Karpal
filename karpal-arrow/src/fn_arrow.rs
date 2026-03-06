use karpal_core::hkt::HKT2;

use crate::arrow::Arrow;
use crate::arrow_apply::ArrowApply;
use crate::arrow_choice::ArrowChoice;
use crate::arrow_loop::ArrowLoop;
use crate::category::Category;
use crate::semigroupoid::Semigroupoid;

/// Marker type whose `P<A, B>` is `Box<dyn Fn(A) -> B>`.
///
/// This is the canonical Arrow instance: the function arrow.
/// Equivalent to `FnP` in karpal-profunctor but independent (no cross-crate dep).
pub struct FnA;

impl HKT2 for FnA {
    type P<A, B> = Box<dyn Fn(A) -> B>;
}

impl Semigroupoid for FnA {
    fn compose<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        f: Box<dyn Fn(B) -> C>,
        g: Box<dyn Fn(A) -> B>,
    ) -> Box<dyn Fn(A) -> C> {
        Box::new(move |a| f(g(a)))
    }
}

impl Category for FnA {
    fn id<A: Clone + 'static>() -> Box<dyn Fn(A) -> A> {
        Box::new(|a| a)
    }
}

impl Arrow for FnA {
    fn arr<A: Clone + 'static, B: Clone + 'static>(
        f: impl Fn(A) -> B + 'static,
    ) -> Box<dyn Fn(A) -> B> {
        Box::new(f)
    }

    fn first<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        pab: Box<dyn Fn(A) -> B>,
    ) -> Box<dyn Fn((A, C)) -> (B, C)> {
        Box::new(move |(a, c)| (pab(a), c))
    }

    fn second<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        pab: Box<dyn Fn(A) -> B>,
    ) -> Box<dyn Fn((C, A)) -> (C, B)> {
        Box::new(move |(c, a)| (c, pab(a)))
    }
}

impl ArrowChoice for FnA {
    fn left<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        pab: Box<dyn Fn(A) -> B>,
    ) -> Box<dyn Fn(Result<A, C>) -> Result<B, C>> {
        Box::new(move |r| match r {
            Ok(a) => Ok(pab(a)),
            Err(c) => Err(c),
        })
    }

    fn right<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
        pab: Box<dyn Fn(A) -> B>,
    ) -> Box<dyn Fn(Result<C, A>) -> Result<C, B>> {
        Box::new(move |r| match r {
            Ok(c) => Ok(c),
            Err(a) => Err(pab(a)),
        })
    }
}

impl ArrowApply for FnA {
    fn app<A: Clone + 'static, B: Clone + 'static>() -> Box<dyn Fn((Box<dyn Fn(A) -> B>, A)) -> B> {
        Box::new(|(f, a)| f(a))
    }
}

impl ArrowLoop for FnA {
    fn loop_arrow<A: Clone + 'static, B: Clone + 'static, D: Default + Clone + 'static>(
        f: Box<dyn Fn((A, D)) -> (B, D)>,
    ) -> Box<dyn Fn(A) -> B> {
        Box::new(move |a| {
            let (b, _d) = f((a, D::default()));
            b
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fna_compose() {
        let f: Box<dyn Fn(i32) -> i32> = Box::new(|x| x + 1);
        let g: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        let fg = FnA::compose(f, g);
        assert_eq!(fg(3), 7); // (3 * 2) + 1
    }

    #[test]
    fn fna_id() {
        let id = FnA::id::<i32>();
        assert_eq!(id(42), 42);
    }

    #[test]
    fn fna_arr() {
        let f = FnA::arr(|x: i32| x.to_string());
        assert_eq!(f(42), "42");
    }

    #[test]
    fn fna_first() {
        let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        let f = FnA::first::<i32, i32, &str>(double);
        assert_eq!(f((5, "hi")), (10, "hi"));
    }

    #[test]
    fn fna_second() {
        let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        let f = FnA::second::<i32, i32, &str>(double);
        assert_eq!(f(("hi", 5)), ("hi", 10));
    }

    #[test]
    fn fna_split() {
        let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        let negate: Box<dyn Fn(i32) -> i32> = Box::new(|x| -x);
        let f = FnA::split(double, negate);
        assert_eq!(f((3, 4)), (6, -4));
    }

    #[test]
    fn fna_fanout() {
        let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        let negate: Box<dyn Fn(i32) -> i32> = Box::new(|x| -x);
        let f = FnA::fanout(double, negate);
        assert_eq!(f(5), (10, -5));
    }

    #[test]
    fn fna_left() {
        let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        let f = FnA::left::<i32, i32, &str>(double);
        assert_eq!(f(Ok(5)), Ok(10));
        assert_eq!(f(Err("nope")), Err("nope"));
    }

    #[test]
    fn fna_right() {
        let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        let f = FnA::right::<i32, i32, &str>(double);
        assert_eq!(f(Err(5)), Err(10));
        assert_eq!(f(Ok("yep")), Ok("yep"));
    }

    #[test]
    fn fna_splat() {
        let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        let len: Box<dyn Fn(String) -> usize> = Box::new(|s| s.len());
        let f = FnA::splat(double, len);
        assert_eq!(f(Ok(5)), Ok(10));
        assert_eq!(f(Err("hello".to_string())), Err(5));
    }

    #[test]
    fn fna_fanin() {
        let double: Box<dyn Fn(i32) -> String> = Box::new(|x| format!("int:{}", x));
        let show: Box<dyn Fn(bool) -> String> = Box::new(|b| format!("bool:{}", b));
        let f = FnA::fanin(double, show);
        assert_eq!(f(Ok(42)), "int:42");
        assert_eq!(f(Err(true)), "bool:true");
    }

    #[test]
    fn fna_app() {
        let app = FnA::app::<i32, i32>();
        let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        assert_eq!(app((double, 5)), 10);
    }

    #[test]
    fn fna_loop_arrow() {
        // loop_arrow feeds D::default() as the feedback value
        let f = FnA::loop_arrow::<i32, i32, i32>(Box::new(|(a, d)| (a + d, d)));
        assert_eq!(f(5), 5); // 5 + 0 (i32::default() == 0)
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Semigroupoid associativity: compose(f, compose(g, h)) == compose(compose(f, g), h)
        #[test]
        fn associativity(x in any::<i16>()) {
            let f = || -> Box<dyn Fn(i16) -> i16> { Box::new(|a| a.wrapping_add(1)) };
            let g = || -> Box<dyn Fn(i16) -> i16> { Box::new(|a| a.wrapping_mul(2)) };
            let h = || -> Box<dyn Fn(i16) -> i16> { Box::new(|a| a.wrapping_sub(3)) };

            let left = FnA::compose(f(), FnA::compose(g(), h()));
            let right = FnA::compose(FnA::compose(f(), g()), h());
            prop_assert_eq!(left(x), right(x));
        }

        // Category left identity: compose(id(), f) == f
        #[test]
        fn left_identity(x in any::<i16>()) {
            let f = || -> Box<dyn Fn(i16) -> i16> { Box::new(|a| a.wrapping_mul(2)) };
            let left = FnA::compose(FnA::id(), f());
            let right = f();
            prop_assert_eq!(left(x), right(x));
        }

        // Category right identity: compose(f, id()) == f
        #[test]
        fn right_identity(x in any::<i16>()) {
            let f = || -> Box<dyn Fn(i16) -> i16> { Box::new(|a| a.wrapping_mul(2)) };
            let left = FnA::compose(f(), FnA::id());
            let right = f();
            prop_assert_eq!(left(x), right(x));
        }

        // Arrow: arr(id) == id()
        #[test]
        fn arr_id(x in any::<i16>()) {
            let left = FnA::arr(|a: i16| a);
            let right = FnA::id::<i16>();
            prop_assert_eq!(left(x), right(x));
        }

        // Arrow: arr(g . f) == compose(arr(g), arr(f))
        #[test]
        fn arr_composition(x in any::<i16>()) {
            let f = |a: i16| a.wrapping_add(1);
            let g = |a: i16| a.wrapping_mul(2);

            let left = FnA::arr(move |a: i16| g(f(a)));
            let right = FnA::compose(FnA::arr(g), FnA::arr(f));
            prop_assert_eq!(left(x), right(x));
        }

        // Arrow: first(arr(f)) == arr(|(a, c)| (f(a), c))
        #[test]
        fn first_arr(x in any::<i16>(), c in any::<i16>()) {
            let f = |a: i16| a.wrapping_add(1);
            let left = FnA::first::<i16, i16, i16>(FnA::arr(f));
            let right = FnA::arr(move |(a, c): (i16, i16)| (f(a), c));
            prop_assert_eq!(left((x, c)), right((x, c)));
        }

        // Arrow: first(compose(f, g)) == compose(first(f), first(g))
        #[test]
        fn first_compose(x in any::<i16>(), c in any::<i16>()) {
            let f = || -> Box<dyn Fn(i16) -> i16> { Box::new(|a| a.wrapping_add(1)) };
            let g = || -> Box<dyn Fn(i16) -> i16> { Box::new(|a| a.wrapping_mul(2)) };

            let left = FnA::first::<i16, i16, i16>(FnA::compose(f(), g()));
            let right = FnA::compose(
                FnA::first::<i16, i16, i16>(f()),
                FnA::first::<i16, i16, i16>(g()),
            );
            prop_assert_eq!(left((x, c)), right((x, c)));
        }

        // ArrowChoice: left(arr(f)) == arr(|r| r.map(f))
        #[test]
        fn left_arr(x in any::<Result<i16, i16>>()) {
            let f = |a: i16| a.wrapping_add(1);
            let left = FnA::left::<i16, i16, i16>(FnA::arr(f));
            let right = FnA::arr(move |r: Result<i16, i16>| r.map(f));
            prop_assert_eq!(left(x), right(x));
        }
    }
}
