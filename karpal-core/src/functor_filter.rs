use crate::functor::Functor;
use crate::hkt::OptionF;
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::hkt::VecF;

/// FunctorFilter: a Functor that can filter elements during mapping.
///
/// Laws:
/// - Identity: `filter_map(fa, Some) == fa`
/// - Composition: `filter_map(filter_map(fa, f), g) == filter_map(fa, |a| f(a).and_then(g))`
pub trait FunctorFilter: Functor {
    fn filter_map<A, B>(fa: Self::Of<A>, f: impl Fn(A) -> Option<B>) -> Self::Of<B>;

    fn filter<A: Clone>(fa: Self::Of<A>, pred: impl Fn(&A) -> bool) -> Self::Of<A> {
        Self::filter_map(fa, |a| if pred(&a) { Some(a) } else { None })
    }
}

impl FunctorFilter for OptionF {
    fn filter_map<A, B>(fa: Option<A>, f: impl Fn(A) -> Option<B>) -> Option<B> {
        fa.and_then(f)
    }
}

// No FunctorFilter for ResultF — would need E: Default, too restrictive.

#[cfg(any(feature = "std", feature = "alloc"))]
impl FunctorFilter for VecF {
    fn filter_map<A, B>(fa: Vec<A>, f: impl Fn(A) -> Option<B>) -> Vec<B> {
        fa.into_iter().filter_map(f).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_filter_map_some() {
        let result = OptionF::filter_map(Some(3), |x| if x > 0 { Some(x * 2) } else { None });
        assert_eq!(result, Some(6));
    }

    #[test]
    fn option_filter_map_filtered() {
        let result = OptionF::filter_map(Some(-1), |x| if x > 0 { Some(x * 2) } else { None });
        assert_eq!(result, None);
    }

    #[test]
    fn option_filter() {
        assert_eq!(OptionF::filter(Some(3), |x| *x > 0), Some(3));
        assert_eq!(OptionF::filter(Some(-1), |x| *x > 0), None);
    }

    #[test]
    fn vec_filter_map() {
        let result = VecF::filter_map(
            vec![1, -2, 3, -4],
            |x| if x > 0 { Some(x * 2) } else { None },
        );
        assert_eq!(result, vec![2, 6]);
    }

    #[test]
    fn vec_filter() {
        assert_eq!(VecF::filter(vec![1, -2, 3, -4], |x| *x > 0), vec![1, 3]);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Identity: filter_map(fa, Some) == fa
        #[test]
        fn option_identity(x in any::<Option<i32>>()) {
            let result = OptionF::filter_map(x, Some);
            prop_assert_eq!(result, x);
        }

        // Composition: filter_map(filter_map(fa, f), g) == filter_map(fa, |a| f(a).and_then(g))
        #[test]
        fn option_composition(x in any::<Option<i16>>()) {
            let f = |a: i16| if a > 0 { Some(a) } else { None };
            let g = |a: i16| if a < 100 { Some(a.wrapping_mul(2)) } else { None };

            let left = OptionF::filter_map(OptionF::filter_map(x, f), g);
            let right = OptionF::filter_map(x, |a| f(a).and_then(g));
            prop_assert_eq!(left, right);
        }

        #[test]
        fn vec_identity(x in prop::collection::vec(any::<i32>(), 0..10)) {
            let result = VecF::filter_map(x.clone(), Some);
            prop_assert_eq!(result, x);
        }

        #[test]
        fn vec_composition(x in prop::collection::vec(any::<i8>(), 0..10)) {
            let f = |a: i8| if a > 0 { Some(a) } else { None };
            let g = |a: i8| if a < 50 { Some(a.wrapping_mul(2)) } else { None };

            let left = VecF::filter_map(VecF::filter_map(x.clone(), f), g);
            let right = VecF::filter_map(x, |a| f(a).and_then(g));
            prop_assert_eq!(left, right);
        }
    }
}
