use crate::hkt::{EnvF, HKT, IdentityF, OptionF, ResultF};
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::hkt::{NonEmptyVec, NonEmptyVecF, VecF};

/// Invariant functor: maps with both a covariant and contravariant function.
///
/// Every covariant Functor is trivially Invariant (ignoring `g`).
/// Every Contravariant is also Invariant (ignoring `f`).
///
/// Laws:
/// - Identity: `invmap(fa, id, id) == fa`
/// - Composition: `invmap(fa, g1 . f1, f2 . g2) == invmap(invmap(fa, f1, f2), g1, g2)`
pub trait Invariant: HKT {
    fn invmap<A, B>(fa: Self::Of<A>, f: impl Fn(A) -> B, g: impl Fn(B) -> A) -> Self::Of<B>;
}

impl Invariant for OptionF {
    fn invmap<A, B>(fa: Option<A>, f: impl Fn(A) -> B, _g: impl Fn(B) -> A) -> Option<B> {
        fa.map(f)
    }
}

impl<E> Invariant for ResultF<E> {
    fn invmap<A, B>(fa: Result<A, E>, f: impl Fn(A) -> B, _g: impl Fn(B) -> A) -> Result<B, E> {
        fa.map(f)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Invariant for VecF {
    fn invmap<A, B>(fa: Vec<A>, f: impl Fn(A) -> B, _g: impl Fn(B) -> A) -> Vec<B> {
        fa.into_iter().map(f).collect()
    }
}

impl Invariant for IdentityF {
    fn invmap<A, B>(fa: A, f: impl Fn(A) -> B, _g: impl Fn(B) -> A) -> B {
        f(fa)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Invariant for NonEmptyVecF {
    fn invmap<A, B>(fa: NonEmptyVec<A>, f: impl Fn(A) -> B, _g: impl Fn(B) -> A) -> NonEmptyVec<B> {
        NonEmptyVec::new(f(fa.head), fa.tail.into_iter().map(&f).collect())
    }
}

impl<E> Invariant for EnvF<E> {
    fn invmap<A, B>(fa: (E, A), f: impl Fn(A) -> B, _g: impl Fn(B) -> A) -> (E, B) {
        (fa.0, f(fa.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_invmap() {
        let result = OptionF::invmap(Some(3), |x| x * 2, |x| x / 2);
        assert_eq!(result, Some(6));
    }

    #[test]
    fn option_invmap_none() {
        let result = OptionF::invmap(None::<i32>, |x| x * 2, |x| x / 2);
        assert_eq!(result, None);
    }

    #[test]
    fn result_invmap() {
        let result = ResultF::<&str>::invmap(Ok(5), |x| x + 1, |x| x - 1);
        assert_eq!(result, Ok(6));
    }

    #[test]
    fn vec_invmap() {
        let result = VecF::invmap(vec![1, 2, 3], |x| x * 2, |x| x / 2);
        assert_eq!(result, vec![2, 4, 6]);
    }

    #[test]
    fn identity_invmap() {
        let result = IdentityF::invmap(42, |x| x + 1, |x| x - 1);
        assert_eq!(result, 43);
    }

    #[test]
    fn nonemptyvec_invmap() {
        let nev = NonEmptyVec::new(1, vec![2, 3]);
        let result = NonEmptyVecF::invmap(nev, |x| x * 10, |x| x / 10);
        assert_eq!(result, NonEmptyVec::new(10, vec![20, 30]));
    }

    #[test]
    fn env_invmap() {
        let result = EnvF::<&str>::invmap(("hello", 42), |x| x + 1, |x| x - 1);
        assert_eq!(result, ("hello", 43));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Identity: invmap(fa, id, id) == fa
        #[test]
        fn option_identity(x in any::<Option<i32>>()) {
            let result = OptionF::invmap(x, |a| a, |a| a);
            prop_assert_eq!(result, x);
        }

        // Composition: invmap(fa, g1 . f1, f2 . g2) == invmap(invmap(fa, f1, f2), g1, g2)
        #[test]
        fn option_composition(x in any::<Option<i16>>()) {
            let f1 = |a: i16| a.wrapping_add(1);
            let f2 = |a: i16| a.wrapping_sub(1);
            let g1 = |a: i16| a.wrapping_mul(2);
            let g2 = |a: i16| a / 2; // approximate inverse

            let left = OptionF::invmap(x, |a| g1(f1(a)), |a| f2(g2(a)));
            let right = OptionF::invmap(OptionF::invmap(x, f1, f2), g1, g2);
            prop_assert_eq!(left, right);
        }

        #[test]
        fn vec_identity(x in prop::collection::vec(any::<i32>(), 0..10)) {
            let result = VecF::invmap(x.clone(), |a| a, |a| a);
            prop_assert_eq!(result, x);
        }

        #[test]
        fn vec_composition(x in prop::collection::vec(any::<i16>(), 0..10)) {
            let f1 = |a: i16| a.wrapping_add(1);
            let f2 = |a: i16| a.wrapping_sub(1);
            let g1 = |a: i16| a.wrapping_mul(2);
            let g2 = |a: i16| a / 2;

            let left = VecF::invmap(x.clone(), |a| g1(f1(a)), |a| f2(g2(a)));
            let right = VecF::invmap(VecF::invmap(x, f1, f2), g1, g2);
            prop_assert_eq!(left, right);
        }
    }
}
