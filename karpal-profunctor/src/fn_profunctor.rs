use alloc::boxed::Box;

use crate::choice::Choice;
use crate::profunctor::{HKT2, Profunctor};
use crate::strong::Strong;

/// Marker type whose `P<A, B>` is `Box<dyn Fn(A) -> B>`.
///
/// This is the canonical `Profunctor` instance: the function arrow.
pub struct FnP;

impl HKT2 for FnP {
    type P<A, B> = Box<dyn Fn(A) -> B>;
}

impl Profunctor for FnP {
    fn dimap<A: 'static, B: 'static, C, D>(
        f: impl Fn(C) -> A + 'static,
        g: impl Fn(B) -> D + 'static,
        pab: Box<dyn Fn(A) -> B>,
    ) -> Box<dyn Fn(C) -> D> {
        Box::new(move |c| g(pab(f(c))))
    }
}

impl Strong for FnP {
    fn first<A, B, C>(pab: Box<dyn Fn(A) -> B>) -> Box<dyn Fn((A, C)) -> (B, C)>
    where
        A: 'static,
        B: 'static,
        C: 'static,
    {
        Box::new(move |(a, c)| (pab(a), c))
    }

    fn second<A, B, C>(pab: Box<dyn Fn(A) -> B>) -> Box<dyn Fn((C, A)) -> (C, B)>
    where
        A: 'static,
        B: 'static,
        C: 'static,
    {
        Box::new(move |(c, a)| (c, pab(a)))
    }
}

impl Choice for FnP {
    fn left<A, B, C>(pab: Box<dyn Fn(A) -> B>) -> Box<dyn Fn(Result<A, C>) -> Result<B, C>>
    where
        A: 'static,
        B: 'static,
        C: 'static,
    {
        Box::new(move |r| match r {
            Ok(a) => Ok(pab(a)),
            Err(c) => Err(c),
        })
    }

    fn right<A, B, C>(pab: Box<dyn Fn(A) -> B>) -> Box<dyn Fn(Result<C, A>) -> Result<C, B>>
    where
        A: 'static,
        B: 'static,
        C: 'static,
    {
        Box::new(move |r| match r {
            Ok(c) => Ok(c),
            Err(a) => Err(pab(a)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnp_dimap() {
        let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        let f = FnP::dimap(|s: &str| s.len() as i32, |n: i32| n.to_string(), double);
        assert_eq!(f("hello"), "10");
    }

    #[test]
    fn fnp_first() {
        let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        let f = FnP::first::<i32, i32, &str>(double);
        assert_eq!(f((5, "hi")), (10, "hi"));
    }

    #[test]
    fn fnp_second() {
        let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        let f = FnP::second::<i32, i32, &str>(double);
        assert_eq!(f(("hi", 5)), ("hi", 10));
    }

    #[test]
    fn fnp_left() {
        let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        let f = FnP::left::<i32, i32, &str>(double);
        assert_eq!(f(Ok(5)), Ok(10));
        assert_eq!(f(Err("nope")), Err("nope"));
    }

    #[test]
    fn fnp_right() {
        let double: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        let f = FnP::right::<i32, i32, &str>(double);
        assert_eq!(f(Err(5)), Err(10));
        assert_eq!(f(Ok("yep")), Ok("yep"));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn profunctor_identity(x in any::<i32>()) {
            let id_fn: Box<dyn Fn(i32) -> i32> = Box::new(|a| a);
            let dimapped = FnP::dimap(|a: i32| a, |b: i32| b, id_fn);
            prop_assert_eq!(dimapped(x), x);
        }

        #[test]
        fn profunctor_composition(x in any::<i32>()) {
            let base: Box<dyn Fn(i32) -> i32> = Box::new(|a| a);

            let f = |a: i32| a.wrapping_add(1);
            let g = |a: i32| a.wrapping_mul(2);
            let h = |a: i32| a.wrapping_add(3);
            let i_fn = |a: i32| a.wrapping_mul(4);

            // dimap(f . g, h . i, p)
            let left = FnP::dimap(
                move |a: i32| f(g(a)),
                move |b: i32| h(i_fn(b)),
                base,
            );

            let base2: Box<dyn Fn(i32) -> i32> = Box::new(|a| a);
            // dimap(g, h, dimap(f, i, p))
            let inner = FnP::dimap(f, i_fn, base2);
            let right = FnP::dimap(g, h, inner);

            prop_assert_eq!(left(x), right(x));
        }
    }
}
