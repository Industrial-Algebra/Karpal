use crate::contravariant::PredicateF;
use crate::decide::Decide;

/// Conclude: the contravariant analogue of Plus.
///
/// Adds a `conclude` operation (the identity for `choose`).
/// `conclude` takes a function `A -> Infallible`, witnessing that `A` is
/// uninhabited — so the resulting predicate is vacuously true.
///
/// Laws:
/// - Left identity: `choose(f, conclude(absurd), fa) ≈ contramap(from_right . f, fa)`
/// - Right identity: `choose(f, fa, conclude(absurd)) ≈ contramap(from_left . f, fa)`
pub trait Conclude: Decide {
    fn conclude<A: 'static>(f: impl Fn(A) -> core::convert::Infallible + 'static) -> Self::Of<A>;
}

impl Conclude for PredicateF {
    fn conclude<A: 'static>(
        _f: impl Fn(A) -> core::convert::Infallible + 'static,
    ) -> Box<dyn Fn(A) -> bool> {
        // Vacuously true: if `f` could produce Infallible, `A` is uninhabited,
        // so this predicate will never actually be called with a real value.
        // For inhabited types, the predicate is simply always true.
        Box::new(|_| true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predicate_conclude() {
        // For any inhabited type, conclude gives a predicate that's always true
        let p: Box<dyn Fn(i32) -> bool> = PredicateF::conclude(|_: i32| unreachable!());
        // We can't actually call it with the intent of testing the Infallible path,
        // but we can test that the predicate is true for any value
        assert!(p(42));
        assert!(p(-1));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Left identity: choose(|a| Err(a), conclude(absurd), fa)(x) == fa(x)
        #[test]
        fn predicate_left_identity(x in any::<i32>()) {
            let fa: Box<dyn Fn(i32) -> bool> = Box::new(|a| a > 0);
            let expected = fa(x);

            let fa2: Box<dyn Fn(i32) -> bool> = Box::new(|a| a > 0);
            let result = PredicateF::choose(
                |a: i32| -> Result<core::convert::Infallible, i32> { Err(a) },
                PredicateF::conclude(|i: core::convert::Infallible| -> core::convert::Infallible { i }),
                fa2,
            );
            prop_assert_eq!(result(x), expected);
        }

        // Right identity: choose(|a| Ok(a), fa, conclude(absurd))(x) == fa(x)
        #[test]
        fn predicate_right_identity(x in any::<i32>()) {
            let fa: Box<dyn Fn(i32) -> bool> = Box::new(|a| a > 0);
            let expected = fa(x);

            let fa2: Box<dyn Fn(i32) -> bool> = Box::new(|a| a > 0);
            let result = PredicateF::choose(
                |a: i32| -> Result<i32, core::convert::Infallible> { Ok(a) },
                fa2,
                PredicateF::conclude(|i: core::convert::Infallible| -> core::convert::Infallible { i }),
            );
            prop_assert_eq!(result(x), expected);
        }
    }
}
