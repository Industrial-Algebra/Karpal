use crate::alt::Alt;
use crate::hkt::OptionF;
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::hkt::VecF;

/// Plus: an Alt with a zero/empty element.
///
/// Laws:
/// - Left identity: `alt(zero(), a) == a`
/// - Right identity: `alt(a, zero()) == a`
/// - Annihilation: `fmap(f, zero()) == zero()`
pub trait Plus: Alt {
    fn zero<A>() -> Self::Of<A>;
}

impl Plus for OptionF {
    fn zero<A>() -> Option<A> {
        None
    }
}

// No Plus for ResultF — can't produce a Result<A, E> without an E value.

#[cfg(any(feature = "std", feature = "alloc"))]
impl Plus for VecF {
    fn zero<A>() -> Vec<A> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_zero() {
        assert_eq!(OptionF::zero::<i32>(), None);
    }

    #[test]
    fn vec_zero() {
        assert_eq!(VecF::zero::<i32>(), Vec::<i32>::new());
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use crate::alt::Alt;
    use crate::functor::Functor;
    use proptest::prelude::*;

    proptest! {
        // Left identity: alt(zero(), a) == a
        #[test]
        fn option_left_identity(a in any::<Option<i32>>()) {
            let left = OptionF::alt(OptionF::zero(), a);
            prop_assert_eq!(left, a);
        }

        // Right identity: alt(a, zero()) == a
        #[test]
        fn option_right_identity(a in any::<Option<i32>>()) {
            let left = OptionF::alt(a, OptionF::zero());
            prop_assert_eq!(left, a);
        }

        // Annihilation: fmap(f, zero()) == zero()
        #[test]
        fn option_annihilation(_x in any::<i32>()) {
            let f = |a: i32| a.wrapping_add(1);
            let left = OptionF::fmap(OptionF::zero::<i32>(), f);
            let right = OptionF::zero::<i32>();
            prop_assert_eq!(left, right);
        }

        #[test]
        fn vec_left_identity(a in prop::collection::vec(any::<i32>(), 0..10)) {
            let left = VecF::alt(VecF::zero(), a.clone());
            prop_assert_eq!(left, a);
        }

        #[test]
        fn vec_right_identity(a in prop::collection::vec(any::<i32>(), 0..10)) {
            let left = VecF::alt(a.clone(), VecF::zero());
            prop_assert_eq!(left, a);
        }
    }
}
