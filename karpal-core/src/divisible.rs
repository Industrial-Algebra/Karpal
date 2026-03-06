use crate::contravariant::PredicateF;
use crate::divide::Divide;

/// Divisible: the contravariant analogue of Applicative.
///
/// Adds a `conquer` operation (the identity for `divide`), analogous to `pure`.
///
/// Laws:
/// - Left identity: `divide(f, conquer(), fa) ≈ contramap(snd . f, fa)`
/// - Right identity: `divide(f, fa, conquer()) ≈ contramap(fst . f, fa)`
pub trait Divisible: Divide {
    fn conquer<A: 'static>() -> Self::Of<A>;
}

impl Divisible for PredicateF {
    fn conquer<A: 'static>() -> Box<dyn Fn(A) -> bool> {
        Box::new(|_| true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predicate_conquer() {
        let p: Box<dyn Fn(i32) -> bool> = PredicateF::conquer();
        assert!(p(42));
        assert!(p(-1));
        assert!(p(0));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Left identity: divide(|a| ((), a), conquer(), fa)(x) == fa(x)
        #[test]
        fn predicate_left_identity(x in any::<i32>()) {
            let fa: Box<dyn Fn(i32) -> bool> = Box::new(|a| a > 0);
            let expected = fa(x);

            let fa2: Box<dyn Fn(i32) -> bool> = Box::new(|a| a > 0);
            let result = PredicateF::divide(
                |a: i32| ((), a),
                PredicateF::conquer::<()>(),
                fa2,
            );
            prop_assert_eq!(result(x), expected);
        }

        // Right identity: divide(|a| (a, ()), fa, conquer())(x) == fa(x)
        #[test]
        fn predicate_right_identity(x in any::<i32>()) {
            let fa: Box<dyn Fn(i32) -> bool> = Box::new(|a| a > 0);
            let expected = fa(x);

            let fa2: Box<dyn Fn(i32) -> bool> = Box::new(|a| a > 0);
            let result = PredicateF::divide(
                |a: i32| (a, ()),
                fa2,
                PredicateF::conquer::<()>(),
            );
            prop_assert_eq!(result(x), expected);
        }
    }
}
