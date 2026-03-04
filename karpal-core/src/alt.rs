use crate::functor::Functor;
use crate::hkt::OptionF;
use crate::hkt::ResultF;
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::hkt::VecF;

/// Alt: a Functor with an associative choice operation.
///
/// Laws:
/// - Associativity: `alt(alt(a, b), c) == alt(a, alt(b, c))`
/// - Distributivity: `fmap(f, alt(a, b)) == alt(fmap(f, a), fmap(f, b))`
pub trait Alt: Functor {
    fn alt<A>(fa1: Self::Of<A>, fa2: Self::Of<A>) -> Self::Of<A>;
}

impl Alt for OptionF {
    fn alt<A>(fa1: Option<A>, fa2: Option<A>) -> Option<A> {
        fa1.or(fa2)
    }
}

impl<E> Alt for ResultF<E> {
    fn alt<A>(fa1: Result<A, E>, fa2: Result<A, E>) -> Result<A, E> {
        fa1.or(fa2)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Alt for VecF {
    fn alt<A>(mut fa1: Vec<A>, fa2: Vec<A>) -> Vec<A> {
        fa1.extend(fa2);
        fa1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_alt_some_some() {
        assert_eq!(OptionF::alt(Some(1), Some(2)), Some(1));
    }

    #[test]
    fn option_alt_none_some() {
        assert_eq!(OptionF::alt(None, Some(2)), Some(2));
    }

    #[test]
    fn option_alt_none_none() {
        assert_eq!(OptionF::alt(None::<i32>, None), None);
    }

    #[test]
    fn result_alt() {
        assert_eq!(ResultF::<&str>::alt(Err("a"), Ok(2)), Ok(2));
        assert_eq!(ResultF::<&str>::alt(Ok(1), Ok(2)), Ok(1));
    }

    #[test]
    fn vec_alt() {
        assert_eq!(VecF::alt(vec![1, 2], vec![3, 4]), vec![1, 2, 3, 4]);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use crate::functor::Functor;
    use proptest::prelude::*;

    proptest! {
        // Associativity: alt(alt(a, b), c) == alt(a, alt(b, c))
        #[test]
        fn option_associativity(
            a in any::<Option<i32>>(),
            b in any::<Option<i32>>(),
            c in any::<Option<i32>>()
        ) {
            let left = OptionF::alt(OptionF::alt(a, b), c);
            let right = OptionF::alt(a, OptionF::alt(b, c));
            prop_assert_eq!(left, right);
        }

        // Distributivity: fmap(f, alt(a, b)) == alt(fmap(f, a), fmap(f, b))
        #[test]
        fn option_distributivity(
            a in any::<Option<i32>>(),
            b in any::<Option<i32>>()
        ) {
            let f = |x: i32| x.wrapping_add(1);
            let left = OptionF::fmap(OptionF::alt(a, b), f);
            let right = OptionF::alt(OptionF::fmap(a, f), OptionF::fmap(b, f));
            prop_assert_eq!(left, right);
        }

        #[test]
        fn vec_associativity(
            a in prop::collection::vec(any::<i32>(), 0..5),
            b in prop::collection::vec(any::<i32>(), 0..5),
            c in prop::collection::vec(any::<i32>(), 0..5)
        ) {
            let left = VecF::alt(VecF::alt(a.clone(), b.clone()), c.clone());
            let right = VecF::alt(a, VecF::alt(b, c));
            prop_assert_eq!(left, right);
        }
    }
}
