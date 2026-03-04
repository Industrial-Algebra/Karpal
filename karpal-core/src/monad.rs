use crate::applicative::Applicative;
use crate::chain::Chain;

/// Monad: Applicative + Chain with no extra methods (blanket impl).
///
/// Laws:
/// - Left identity: `chain(pure(a), f) == f(a)`
/// - Right identity: `chain(m, pure) == m`
pub trait Monad: Applicative + Chain {}

impl<F: Applicative + Chain> Monad for F {}

#[cfg(test)]
mod law_tests {
    use crate::applicative::Applicative;
    use crate::chain::Chain;
    use crate::hkt::OptionF;
    #[cfg(any(feature = "std", feature = "alloc"))]
    use crate::hkt::VecF;
    use proptest::prelude::*;

    proptest! {
        // Left identity: chain(pure(a), f) == f(a)
        #[test]
        fn option_left_identity(a in any::<i32>()) {
            let f = |x: i32| Some(x.wrapping_mul(2));
            let left = OptionF::chain(OptionF::pure(a), f);
            let right = f(a);
            prop_assert_eq!(left, right);
        }

        // Right identity: chain(m, pure) == m
        #[test]
        fn option_right_identity(m in any::<Option<i32>>()) {
            let left = OptionF::chain(m, OptionF::pure);
            prop_assert_eq!(left, m);
        }

        #[test]
        fn vec_left_identity(a in any::<i32>()) {
            let f = |x: i32| vec![x, x.wrapping_add(1)];
            let left = VecF::chain(VecF::pure(a), f);
            let right = f(a);
            prop_assert_eq!(left, right);
        }

        #[test]
        fn vec_right_identity(m in prop::collection::vec(any::<i32>(), 0..10)) {
            let left = VecF::chain(m.clone(), VecF::pure);
            prop_assert_eq!(left, m);
        }
    }
}
