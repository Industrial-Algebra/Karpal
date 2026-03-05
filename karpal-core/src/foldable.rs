#[cfg(any(feature = "std", feature = "alloc"))]
use crate::hkt::VecF;
use crate::hkt::{HKT, OptionF, ResultF};
use crate::monoid::Monoid;

/// Foldable: a structure that can be folded to a summary value.
///
/// Laws:
/// - fold_map consistency: `fold_map(fa, f) == fold_right(fa, M::empty(), |a, acc| f(a).combine(acc))`
pub trait Foldable: HKT {
    fn fold_right<A, B>(fa: Self::Of<A>, init: B, f: impl Fn(A, B) -> B) -> B;

    fn fold_map<A, M: Monoid>(fa: Self::Of<A>, f: impl Fn(A) -> M) -> M {
        Self::fold_right(fa, M::empty(), |a, acc| f(a).combine(acc))
    }
}

impl Foldable for OptionF {
    fn fold_right<A, B>(fa: Option<A>, init: B, f: impl Fn(A, B) -> B) -> B {
        match fa {
            Some(a) => f(a, init),
            None => init,
        }
    }
}

impl<E> Foldable for ResultF<E> {
    fn fold_right<A, B>(fa: Result<A, E>, init: B, f: impl Fn(A, B) -> B) -> B {
        match fa {
            Ok(a) => f(a, init),
            Err(_) => init,
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Foldable for VecF {
    fn fold_right<A, B>(fa: Vec<A>, init: B, f: impl Fn(A, B) -> B) -> B {
        fa.into_iter().rev().fold(init, |acc, a| f(a, acc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_fold_right_some() {
        assert_eq!(OptionF::fold_right(Some(3), 10, |a, b| a + b), 13);
    }

    #[test]
    fn option_fold_right_none() {
        assert_eq!(OptionF::fold_right(None::<i32>, 10, |a, b| a + b), 10);
    }

    #[test]
    fn result_fold_right_ok() {
        assert_eq!(ResultF::<&str>::fold_right(Ok(5), 10, |a, b| a + b), 15);
    }

    #[test]
    fn result_fold_right_err() {
        assert_eq!(
            ResultF::<&str>::fold_right(Err("bad"), 10, |a: i32, b| a + b),
            10
        );
    }

    #[test]
    fn vec_fold_right() {
        // fold_right [1,2,3] with init=0 and f(a,b) = a - b
        // = 1 - (2 - (3 - 0)) = 1 - (2 - 3) = 1 - (-1) = 2
        assert_eq!(VecF::fold_right(vec![1, 2, 3], 0, |a, b| a - b), 2);
    }

    #[test]
    fn vec_fold_map() {
        let result = VecF::fold_map(vec![1, 2, 3], |a: i32| a);
        assert_eq!(result, 6); // 1 + 2 + 3
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use crate::semigroup::Semigroup;
    use proptest::prelude::*;

    proptest! {
        // fold_map consistency: fold_map(fa, f) == fold_right(fa, M::empty(), |a, acc| f(a).combine(acc))
        #[test]
        fn option_fold_map_consistency(x in any::<Option<i16>>()) {
            let f = |a: i16| a as i32;
            let left = OptionF::fold_map(x, f);
            let right = OptionF::fold_right(x, i32::empty(), |a, acc| f(a).combine(acc));
            prop_assert_eq!(left, right);
        }

        #[test]
        fn vec_fold_map_consistency(x in prop::collection::vec(0i16..100, 0..10)) {
            let f = |a: i16| a as i32;
            let left = VecF::fold_map(x.clone(), f);
            let right = VecF::fold_right(x, i32::empty(), |a, acc| f(a).combine(acc));
            prop_assert_eq!(left, right);
        }
    }
}
