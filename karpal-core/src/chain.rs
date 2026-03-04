use crate::apply::Apply;
use crate::hkt::OptionF;
use crate::hkt::ResultF;
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::hkt::VecF;

/// Chain (FlatMap): an Apply with monadic bind.
///
/// Laws:
/// - Associativity: `chain(chain(m, f), g) == chain(m, |x| chain(f(x), g))`
pub trait Chain: Apply {
    fn chain<A, B>(fa: Self::Of<A>, f: impl Fn(A) -> Self::Of<B>) -> Self::Of<B>;
}

impl Chain for OptionF {
    fn chain<A, B>(fa: Option<A>, f: impl Fn(A) -> Option<B>) -> Option<B> {
        fa.and_then(f)
    }
}

impl<E> Chain for ResultF<E> {
    fn chain<A, B>(fa: Result<A, E>, f: impl Fn(A) -> Result<B, E>) -> Result<B, E> {
        fa.and_then(f)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Chain for VecF {
    fn chain<A, B>(fa: Vec<A>, f: impl Fn(A) -> Vec<B>) -> Vec<B> {
        fa.into_iter().flat_map(f).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_chain_some() {
        let result = OptionF::chain(Some(3), |x| if x > 0 { Some(x * 2) } else { None });
        assert_eq!(result, Some(6));
    }

    #[test]
    fn option_chain_none() {
        let result = OptionF::chain(None::<i32>, |x| Some(x * 2));
        assert_eq!(result, None);
    }

    #[test]
    fn result_chain_ok() {
        let result = ResultF::<&str>::chain(Ok(3), |x| Ok(x + 1));
        assert_eq!(result, Ok(4));
    }

    #[test]
    fn result_chain_err() {
        let result = ResultF::<&str>::chain(Err("bad"), |x: i32| Ok(x + 1));
        assert_eq!(result, Err("bad"));
    }

    #[test]
    fn vec_chain() {
        let result = VecF::chain(vec![1, 2, 3], |x| vec![x, x * 10]);
        assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Associativity: chain(chain(m, f), g) == chain(m, |x| chain(f(x), g))
        #[test]
        fn option_associativity(x in any::<i16>()) {
            let m = Some(x);
            let f = |a: i16| Some(a.wrapping_add(1));
            let g = |a: i16| Some(a.wrapping_mul(2));

            let left = OptionF::chain(OptionF::chain(m, f), g);
            let right = OptionF::chain(m, |a| OptionF::chain(f(a), g));
            prop_assert_eq!(left, right);
        }

        #[test]
        fn vec_associativity(x in prop::collection::vec(any::<i8>(), 0..5)) {
            let f = |a: i8| vec![a, a.wrapping_add(1)];
            let g = |a: i8| vec![a.wrapping_mul(2)];

            let left = VecF::chain(VecF::chain(x.clone(), f), g);
            let right = VecF::chain(x, |a| VecF::chain(f(a), g));
            prop_assert_eq!(left, right);
        }
    }
}
