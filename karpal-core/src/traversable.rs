use crate::applicative::Applicative;
use crate::foldable::Foldable;
use crate::functor::Functor;
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::hkt::VecF;
use crate::hkt::{OptionF, ResultF};

/// Traversable: a Functor + Foldable that can be traversed with an effect.
///
/// Laws:
/// - Identity: `traverse::<IdentityF, _, _, _, _>(fa, pure) == pure(fa)`
///   (We test with OptionF as the effect.)
pub trait Traversable: Functor + Foldable {
    fn traverse<G, A, B, F>(fa: Self::Of<A>, f: F) -> G::Of<Self::Of<B>>
    where
        G: Applicative,
        F: Fn(A) -> G::Of<B>,
        B: Clone;
}

impl Traversable for OptionF {
    fn traverse<G, A, B, F>(fa: Option<A>, f: F) -> G::Of<Option<B>>
    where
        G: Applicative,
        F: Fn(A) -> G::Of<B>,
        B: Clone,
    {
        match fa {
            None => G::pure(None),
            Some(a) => G::fmap(f(a), |b| Some(b)),
        }
    }
}

impl<E: Clone> Traversable for ResultF<E> {
    fn traverse<G, A, B, F>(fa: Result<A, E>, f: F) -> G::Of<Result<B, E>>
    where
        G: Applicative,
        F: Fn(A) -> G::Of<B>,
        B: Clone,
    {
        match fa {
            Err(e) => G::pure(Err(e)),
            Ok(a) => G::fmap(f(a), Ok),
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Traversable for VecF {
    fn traverse<G, A, B, F>(fa: Vec<A>, f: F) -> G::Of<Vec<B>>
    where
        G: Applicative,
        F: Fn(A) -> G::Of<B>,
        B: Clone,
    {
        let init: G::Of<Vec<B>> = G::pure(Vec::new());
        fa.into_iter().fold(init, |acc, a| {
            let gb = f(a);
            G::ap(
                G::fmap(acc, |v: Vec<B>| {
                    move |b: B| {
                        let mut v = v.clone();
                        v.push(b);
                        v
                    }
                }),
                gb,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_traverse_some() {
        let result = OptionF::traverse::<OptionF, _, _, _>(Some(3), |x| Some(x * 2));
        assert_eq!(result, Some(Some(6)));
    }

    #[test]
    fn option_traverse_none() {
        let result = OptionF::traverse::<OptionF, i32, i32, _>(None, |x| Some(x * 2));
        assert_eq!(result, Some(None));
    }

    #[test]
    fn option_traverse_effect_fails() {
        let result = OptionF::traverse::<OptionF, _, _, _>(Some(3), |_x| None::<i32>);
        assert_eq!(result, None);
    }

    #[test]
    fn result_traverse_ok() {
        let result = ResultF::<&str>::traverse::<OptionF, _, _, _>(Ok(3), |x| Some(x * 2));
        assert_eq!(result, Some(Ok(6)));
    }

    #[test]
    fn result_traverse_err() {
        let result = ResultF::<&str>::traverse::<OptionF, i32, i32, _>(Err("bad"), |x| Some(x * 2));
        assert_eq!(result, Some(Err("bad")));
    }

    #[test]
    fn vec_traverse_all_some() {
        let result = VecF::traverse::<OptionF, _, _, _>(vec![1, 2, 3], |x| Some(x * 2));
        assert_eq!(result, Some(vec![2, 4, 6]));
    }

    #[test]
    fn vec_traverse_one_none() {
        let result = VecF::traverse::<OptionF, _, _, _>(vec![1, 2, 3], |x| {
            if x == 2 { None } else { Some(x * 2) }
        });
        assert_eq!(result, None);
    }

    #[test]
    fn vec_traverse_empty() {
        let result = VecF::traverse::<OptionF, i32, i32, _>(vec![], |x| Some(x * 2));
        assert_eq!(result, Some(vec![]));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Identity: traverse(fa, Some) == Some(fa)
        #[test]
        fn option_identity(x in any::<Option<i32>>()) {
            let result = OptionF::traverse::<OptionF, _, _, _>(x, Some);
            prop_assert_eq!(result, Some(x));
        }

        #[test]
        fn vec_identity(x in prop::collection::vec(any::<i32>(), 0..10)) {
            let result = VecF::traverse::<OptionF, _, _, _>(x.clone(), Some);
            prop_assert_eq!(result, Some(x));
        }
    }
}
