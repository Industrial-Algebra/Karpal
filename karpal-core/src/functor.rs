use crate::hkt::{EnvF, HKT, IdentityF, OptionF, ResultF};
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::hkt::{NonEmptyVec, NonEmptyVecF};

/// Covariant functor: lifts a function `A -> B` into `F<A> -> F<B>`.
pub trait Functor: HKT {
    fn fmap<A, B>(fa: Self::Of<A>, f: impl Fn(A) -> B) -> Self::Of<B>;
}

impl Functor for OptionF {
    fn fmap<A, B>(fa: Option<A>, f: impl Fn(A) -> B) -> Option<B> {
        fa.map(f)
    }
}

impl<E> Functor for ResultF<E> {
    fn fmap<A, B>(fa: Result<A, E>, f: impl Fn(A) -> B) -> Result<B, E> {
        fa.map(f)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Functor for crate::hkt::VecF {
    fn fmap<A, B>(fa: Vec<A>, f: impl Fn(A) -> B) -> Vec<B> {
        fa.into_iter().map(f).collect()
    }
}

impl Functor for IdentityF {
    fn fmap<A, B>(fa: A, f: impl Fn(A) -> B) -> B {
        f(fa)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Functor for NonEmptyVecF {
    fn fmap<A, B>(fa: NonEmptyVec<A>, f: impl Fn(A) -> B) -> NonEmptyVec<B> {
        NonEmptyVec::new(f(fa.head), fa.tail.into_iter().map(&f).collect())
    }
}

impl<E> Functor for EnvF<E> {
    fn fmap<A, B>(fa: (E, A), f: impl Fn(A) -> B) -> (E, B) {
        (fa.0, f(fa.1))
    }
}

// Note: StoreF and TracedF cannot implement the generic Functor trait because
// Box<dyn Fn> requires 'static bounds that the trait signature doesn't allow.
// They get their own fmap via the Extend/Comonad implementation.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_fmap_some() {
        let result = OptionF::fmap(Some(2), |x| x * 3);
        assert_eq!(result, Some(6));
    }

    #[test]
    fn option_fmap_none() {
        let result = OptionF::fmap(None::<i32>, |x| x * 3);
        assert_eq!(result, None);
    }

    #[test]
    fn result_fmap_ok() {
        let result = ResultF::<String>::fmap(Ok(5), |x| x + 1);
        assert_eq!(result, Ok(6));
    }

    #[test]
    fn result_fmap_err() {
        let result = ResultF::<String>::fmap(Err("bad".to_string()), |x: i32| x + 1);
        assert_eq!(result, Err("bad".to_string()));
    }

    #[test]
    fn vec_fmap() {
        let result = crate::hkt::VecF::fmap(vec![1, 2, 3], |x| x * 2);
        assert_eq!(result, vec![2, 4, 6]);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    // Identity law: fmap(id, fa) == fa
    // Composition law: fmap(g . f, fa) == fmap(g, fmap(f, fa))

    proptest! {
        #[test]
        fn option_identity(x in any::<Option<i32>>()) {
            let result = OptionF::fmap(x.clone(), |a| a);
            prop_assert_eq!(result, x);
        }

        #[test]
        fn option_composition(x in any::<Option<i32>>()) {
            let f = |a: i32| a.wrapping_add(1);
            let g = |a: i32| a.wrapping_mul(2);
            let left = OptionF::fmap(x.clone(), |a| g(f(a)));
            let right = OptionF::fmap(OptionF::fmap(x, f), g);
            prop_assert_eq!(left, right);
        }

        #[test]
        fn result_identity(x in any::<Result<i32, u8>>()) {
            let result = ResultF::<u8>::fmap(x.clone(), |a| a);
            prop_assert_eq!(result, x);
        }

        #[test]
        fn result_composition(x in any::<Result<i32, u8>>()) {
            let f = |a: i32| a.wrapping_add(1);
            let g = |a: i32| a.wrapping_mul(2);
            let left = ResultF::<u8>::fmap(x.clone(), |a| g(f(a)));
            let right = ResultF::<u8>::fmap(ResultF::<u8>::fmap(x, f), g);
            prop_assert_eq!(left, right);
        }

        #[test]
        fn vec_identity(x in prop::collection::vec(any::<i32>(), 0..20)) {
            let result = crate::hkt::VecF::fmap(x.clone(), |a| a);
            prop_assert_eq!(result, x);
        }

        #[test]
        fn vec_composition(x in prop::collection::vec(any::<i32>(), 0..20)) {
            let f = |a: i32| a.wrapping_add(1);
            let g = |a: i32| a.wrapping_mul(2);
            let left = crate::hkt::VecF::fmap(x.clone(), |a| g(f(a)));
            let right = crate::hkt::VecF::fmap(crate::hkt::VecF::fmap(x, f), g);
            prop_assert_eq!(left, right);
        }
    }
}
