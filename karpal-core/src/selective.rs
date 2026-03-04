use crate::applicative::Applicative;
use crate::hkt::OptionF;

/// Selective: an Applicative that can conditionally apply effects.
///
/// Laws:
/// - Identity: `select(fmap(Right, x), _) == fmap(id, x)`
///   (where Right means Ok variant)
pub trait Selective: Applicative {
    fn select<A, B, F>(fab: Self::Of<Result<A, B>>, ff: Self::Of<F>) -> Self::Of<B>
    where
        A: Clone,
        F: Fn(A) -> B;
}

impl Selective for OptionF {
    fn select<A, B, F>(fab: Option<Result<A, B>>, ff: Option<F>) -> Option<B>
    where
        A: Clone,
        F: Fn(A) -> B,
    {
        match fab {
            None => None,
            Some(Ok(a)) => ff.map(|f| f(a)),
            Some(Err(b)) => Some(b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_select_right() {
        let result = OptionF::select(Some(Err(42i32)), Some(|_x: i32| 0));
        assert_eq!(result, Some(42));
    }

    #[test]
    fn option_select_left_with_fn() {
        let result = OptionF::select(Some(Ok(3i32)), Some(|x: i32| x * 2));
        assert_eq!(result, Some(6));
    }

    #[test]
    fn option_select_left_no_fn() {
        let result = OptionF::select(Some(Ok(3i32)), None::<fn(i32) -> i32>);
        assert_eq!(result, None);
    }

    #[test]
    fn option_select_none() {
        let result = OptionF::select(None::<Result<i32, i32>>, Some(|x: i32| x * 2));
        assert_eq!(result, None);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use crate::functor::Functor;
    use proptest::prelude::*;

    proptest! {
        // Identity: select(fmap(Err, x), _) == x
        // When all values are Err (already resolved), the function is never applied
        #[test]
        fn option_identity(x in any::<Option<i32>>()) {
            let fab = OptionF::fmap(x, |b| Err::<i32, i32>(b));
            let dummy: Option<fn(i32) -> i32> = Some(|_| panic!("should not be called"));
            let result = OptionF::select(fab, dummy);
            prop_assert_eq!(result, x);
        }
    }
}
