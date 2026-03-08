use alloc::boxed::Box;
use alloc::vec::Vec;
use core::marker::PhantomData;

use karpal_core::Monoid;

use crate::choice::Choice;
use crate::profunctor::{HKT2, Profunctor};
use crate::strong::Strong;
use crate::traversing::Traversing;

/// Marker type whose `P<A, B> = Box<dyn Fn(A) -> R>`.
///
/// `B` is phantom — the second argument to `dimap` is ignored.
/// This profunctor "forgets" the output and extracts a summary value.
pub struct ForgetF<R>(PhantomData<R>);

impl<R: 'static> HKT2 for ForgetF<R> {
    type P<A, B> = Box<dyn Fn(A) -> R>;
}

impl<R: 'static> Profunctor for ForgetF<R> {
    fn dimap<A: 'static, B: 'static, C, D>(
        f: impl Fn(C) -> A + 'static,
        _g: impl Fn(B) -> D + 'static,
        pab: Box<dyn Fn(A) -> R>,
    ) -> Box<dyn Fn(C) -> R> {
        Box::new(move |c| pab(f(c)))
    }
}

impl<R: 'static> Strong for ForgetF<R> {
    fn first<A, B, C>(pab: Box<dyn Fn(A) -> R>) -> Box<dyn Fn((A, C)) -> R>
    where
        A: 'static,
        B: 'static,
        C: 'static,
    {
        Box::new(move |(a, _)| pab(a))
    }

    fn second<A, B, C>(pab: Box<dyn Fn(A) -> R>) -> Box<dyn Fn((C, A)) -> R>
    where
        A: 'static,
        B: 'static,
        C: 'static,
    {
        Box::new(move |(_, a)| pab(a))
    }
}

impl<R: Monoid + 'static> Choice for ForgetF<R> {
    fn left<A, B, C>(pab: Box<dyn Fn(A) -> R>) -> Box<dyn Fn(Result<A, C>) -> R>
    where
        A: 'static,
        B: 'static,
        C: 'static,
    {
        Box::new(move |r| match r {
            Ok(a) => pab(a),
            Err(_) => R::empty(),
        })
    }

    fn right<A, B, C>(pab: Box<dyn Fn(A) -> R>) -> Box<dyn Fn(Result<C, A>) -> R>
    where
        A: 'static,
        B: 'static,
        C: 'static,
    {
        Box::new(move |r| match r {
            Ok(_) => R::empty(),
            Err(a) => pab(a),
        })
    }
}

impl<R: Monoid + 'static> Traversing for ForgetF<R> {
    fn wander<S, T, A, B>(
        get_all: impl Fn(&S) -> Vec<A> + 'static,
        _modify_all: impl Fn(S, &dyn Fn(A) -> B) -> T + 'static,
        pab: Self::P<A, B>,
    ) -> Self::P<S, T>
    where
        S: 'static,
        T: 'static,
        A: 'static,
        B: 'static,
    {
        Box::new(move |s: S| {
            get_all(&s)
                .into_iter()
                .map(&*pab)
                .fold(R::empty(), |acc, r| acc.combine(r))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn forget_dimap_ignores_g() {
        let extract: Box<dyn Fn(i32) -> String> = Box::new(|x| x.to_string());
        // g should be completely ignored
        let result = ForgetF::dimap(|x: i32| x + 1, |_: String| 999i32, extract);
        assert_eq!(result(4), "5"); // f(4) = 5, then extract(5) = "5"
    }

    #[test]
    fn forget_strong_first() {
        let extract: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 10);
        let f = <ForgetF<i32> as Strong>::first::<i32, i32, &str>(extract);
        assert_eq!(f((3, "hi")), 30);
    }

    #[test]
    fn forget_strong_second() {
        let extract: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 10);
        let f = <ForgetF<i32> as Strong>::second::<i32, i32, &str>(extract);
        assert_eq!(f(("hi", 3)), 30);
    }

    #[test]
    fn forget_choice_left_match() {
        let extract: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 10);
        let f = <ForgetF<i32> as Choice>::left::<i32, i32, &str>(extract);
        assert_eq!(f(Ok(3)), 30);
    }

    #[test]
    fn forget_choice_left_miss() {
        let extract: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 10);
        let f = <ForgetF<i32> as Choice>::left::<i32, i32, &str>(extract);
        assert_eq!(f(Err("nope")), 0); // Monoid::empty for i32
    }

    #[test]
    fn forget_choice_right_match() {
        let extract: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 10);
        let f = <ForgetF<i32> as Choice>::right::<i32, i32, &str>(extract);
        assert_eq!(f(Err(3)), 30);
    }

    #[test]
    fn forget_phantom_b_verification() {
        // ForgetF<R>::P<A, B> = Box<dyn Fn(A) -> R>, B is completely phantom
        let extract: Box<dyn Fn(i32) -> String> = Box::new(|x| format!("got {x}"));
        // B can be anything — it's never used
        let result = <ForgetF<String> as Profunctor>::dimap(
            |x: i32| x,
            |_: String| vec![1, 2, 3], // g produces Vec<i32>, totally ignored
            extract,
        );
        assert_eq!(result(42), "got 42");
    }

    proptest! {
        #[test]
        fn forget_profunctor_identity(x in any::<i32>()) {
            let id_fn: Box<dyn Fn(i32) -> i32> = Box::new(|a| a);
            let dimapped = <ForgetF<i32> as Profunctor>::dimap(|a: i32| a, |b: i32| b, id_fn);
            prop_assert_eq!(dimapped(x), x);
        }
    }
}
