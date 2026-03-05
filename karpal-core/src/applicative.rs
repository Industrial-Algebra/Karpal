use crate::apply::Apply;
use crate::hkt::OptionF;
use crate::hkt::ResultF;
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::hkt::VecF;

/// Applicative: an Apply that can lift a pure value into the functor.
///
/// Laws:
/// - Identity: `ap(pure(id), v) == v`
/// - Homomorphism: `ap(pure(f), pure(x)) == pure(f(x))`
/// - Interchange: `ap(u, pure(y)) == ap(pure(|f| f(y)), u)`
pub trait Applicative: Apply {
    fn pure<A>(a: A) -> Self::Of<A>;
}

impl Applicative for OptionF {
    fn pure<A>(a: A) -> Option<A> {
        Some(a)
    }
}

impl<E> Applicative for ResultF<E> {
    fn pure<A>(a: A) -> Result<A, E> {
        Ok(a)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Applicative for VecF {
    fn pure<A>(a: A) -> Vec<A> {
        vec![a]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_pure() {
        assert_eq!(OptionF::pure(42), Some(42));
    }

    #[test]
    fn result_pure() {
        assert_eq!(ResultF::<String>::pure(42), Ok(42));
    }

    #[test]
    fn vec_pure() {
        assert_eq!(VecF::pure(42), vec![42]);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Identity: ap(pure(id), v) == v
        #[test]
        fn option_identity(x in any::<Option<i32>>()) {
            let id_fn: Option<fn(i32) -> i32> = OptionF::pure(|a| a);
            let result = OptionF::ap(id_fn, x.clone());
            prop_assert_eq!(result, x);
        }

        // Homomorphism: ap(pure(f), pure(x)) == pure(f(x))
        #[test]
        fn option_homomorphism(x in any::<i32>()) {
            let f = |a: i32| a.wrapping_add(1);
            let left = OptionF::ap(OptionF::pure(f as fn(i32) -> i32), OptionF::pure(x));
            let right = OptionF::pure(f(x));
            prop_assert_eq!(left, right);
        }

        // Interchange: ap(u, pure(y)) == ap(pure(|f| f(y)), u)
        #[test]
        fn option_interchange(y in any::<i16>()) {
            let u: Option<fn(i16) -> i16> = Some(|a| a.wrapping_mul(2));
            let left = OptionF::ap(u, OptionF::pure(y));
            let right = OptionF::ap(
                OptionF::pure(move |f: fn(i16) -> i16| f(y)),
                Some(|a: i16| a.wrapping_mul(2) as i16),
            );
            prop_assert_eq!(left, right);
        }

        #[test]
        fn vec_identity(x in prop::collection::vec(any::<i32>(), 0..10)) {
            let id_fn: Vec<fn(i32) -> i32> = VecF::pure(|a| a);
            let result = VecF::ap(id_fn, x.clone());
            prop_assert_eq!(result, x);
        }

        #[test]
        fn vec_homomorphism(x in any::<i32>()) {
            let f = |a: i32| a.wrapping_add(1);
            let left = VecF::ap(VecF::pure(f as fn(i32) -> i32), VecF::pure(x));
            let right = VecF::pure(f(x));
            prop_assert_eq!(left, right);
        }
    }
}
