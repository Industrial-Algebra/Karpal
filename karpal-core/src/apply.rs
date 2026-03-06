use crate::functor::Functor;
use crate::hkt::OptionF;
use crate::hkt::ResultF;
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::hkt::VecF;

/// Apply: a Functor that can apply a wrapped function to a wrapped value.
///
/// Laws:
/// - Composition: `ap(ap(fmap(compose, f), g), x) == ap(f, ap(g, x))`
pub trait Apply: Functor {
    fn ap<A, B, F>(ff: Self::Of<F>, fa: Self::Of<A>) -> Self::Of<B>
    where
        A: Clone,
        F: Fn(A) -> B;
}

impl Apply for OptionF {
    fn ap<A, B, F>(ff: Option<F>, fa: Option<A>) -> Option<B>
    where
        A: Clone,
        F: Fn(A) -> B,
    {
        match (ff, fa) {
            (Some(f), Some(a)) => Some(f(a)),
            _ => None,
        }
    }
}

impl<E> Apply for ResultF<E> {
    fn ap<A, B, F>(ff: Result<F, E>, fa: Result<A, E>) -> Result<B, E>
    where
        A: Clone,
        F: Fn(A) -> B,
    {
        match (ff, fa) {
            (Ok(f), Ok(a)) => Ok(f(a)),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Apply for VecF {
    fn ap<A, B, F>(ff: Vec<F>, fa: Vec<A>) -> Vec<B>
    where
        A: Clone,
        F: Fn(A) -> B,
    {
        let mut result = Vec::with_capacity(ff.len() * fa.len());
        for f in &ff {
            for a in &fa {
                result.push(f(a.clone()));
            }
        }
        result
    }
}

impl Apply for crate::hkt::IdentityF {
    fn ap<A, B, F>(ff: F, fa: A) -> B
    where
        A: Clone,
        F: Fn(A) -> B,
    {
        ff(fa)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Apply for crate::hkt::NonEmptyVecF {
    fn ap<A, B, F>(
        ff: crate::hkt::NonEmptyVec<F>,
        fa: crate::hkt::NonEmptyVec<A>,
    ) -> crate::hkt::NonEmptyVec<B>
    where
        A: Clone,
        F: Fn(A) -> B,
    {
        // Apply each function to each value (cartesian product)
        let head = (ff.head)(fa.head.clone());
        let mut tail = Vec::new();
        for a in &fa.tail {
            tail.push((ff.head)(a.clone()));
        }
        for f in &ff.tail {
            tail.push(f(fa.head.clone()));
            for a in &fa.tail {
                tail.push(f(a.clone()));
            }
        }
        crate::hkt::NonEmptyVec::new(head, tail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_ap_some() {
        let f: Option<fn(i32) -> i32> = Some(|x| x * 2);
        assert_eq!(OptionF::ap(f, Some(3)), Some(6));
    }

    #[test]
    fn option_ap_none_fn() {
        let f: Option<fn(i32) -> i32> = None;
        assert_eq!(OptionF::ap(f, Some(3)), None);
    }

    #[test]
    fn option_ap_none_val() {
        let f: Option<fn(i32) -> i32> = Some(|x| x * 2);
        assert_eq!(OptionF::ap(f, None), None);
    }

    #[test]
    fn result_ap_ok() {
        let f: Result<fn(i32) -> i32, &str> = Ok(|x| x + 1);
        assert_eq!(ResultF::<&str>::ap(f, Ok(5)), Ok(6));
    }

    #[test]
    fn result_ap_err() {
        let f: Result<fn(i32) -> i32, &str> = Err("bad");
        assert_eq!(ResultF::<&str>::ap(f, Ok(5)), Err("bad"));
    }

    #[test]
    fn vec_ap() {
        let fs: Vec<fn(i32) -> i32> = vec![|x| x + 1, |x| x * 10];
        assert_eq!(VecF::ap(fs, vec![1, 2, 3]), vec![2, 3, 4, 10, 20, 30]);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    // Composition law: ap(ap(fmap(compose, f), g), x) == ap(f, ap(g, x))
    // Simplified test: ap(fmap(compose, f), g) applied to x
    proptest! {
        #[test]
        fn option_composition(x in any::<i16>()) {
            let f: Option<fn(i16) -> i16> = Some(|a| a.wrapping_add(1));
            let g: Option<fn(i16) -> i16> = Some(|a| a.wrapping_mul(2));

            // ap(f, ap(g, x))
            let right = OptionF::ap(f, OptionF::ap(g, Some(x)));

            // ap(ap(fmap(compose, f), g), x) where compose = |f| |g| |x| f(g(x))
            let composed: Option<fn(i16) -> i16> = Some(|a| a.wrapping_mul(2).wrapping_add(1));
            let left = OptionF::ap(composed, Some(x));

            prop_assert_eq!(left, right);
        }

        #[test]
        fn vec_composition(x in prop::collection::vec(any::<i8>(), 0..5)) {
            let f: Vec<fn(i8) -> i8> = vec![|a| a.wrapping_add(1)];
            let g: Vec<fn(i8) -> i8> = vec![|a| a.wrapping_mul(2)];

            let right = VecF::ap(f, VecF::ap(g, x.clone()));

            let composed: Vec<fn(i8) -> i8> = vec![|a| a.wrapping_mul(2).wrapping_add(1)];
            let left = VecF::ap(composed, x);

            prop_assert_eq!(left, right);
        }
    }
}
