use karpal_core::hkt::{HKT, IdentityF, OptionF, ResultF};

/// Functor with `'static` bounds on type parameters.
///
/// This mirrors [`karpal_core::Functor`] but adds `'static` bounds required by
/// types that use `Box<dyn Fn>` internally (monad transformers, `ReaderF`, etc.).
/// Base types that implement `Functor` can implement this trivially since the
/// `'static` bound is strictly weaker.
pub trait FunctorSt: HKT {
    fn fmap_st<A: 'static, B: 'static>(
        fa: Self::Of<A>,
        f: impl Fn(A) -> B + 'static,
    ) -> Self::Of<B>;
}

/// Applicative with `'static` bounds on type parameters.
///
/// The `Clone` bound on `A` is required because some types (ReaderTF, StateTF)
/// wrap the value in a closure that may be called multiple times.
pub trait ApplicativeSt: FunctorSt {
    fn pure_st<A: 'static>(a: A) -> Self::Of<A>;
}

/// Chain (monadic bind) with `'static` bounds on type parameters.
pub trait ChainSt: FunctorSt {
    fn chain_st<A: 'static, B: 'static>(
        fa: Self::Of<A>,
        f: impl Fn(A) -> Self::Of<B> + 'static,
    ) -> Self::Of<B>;
}

// --- Base type implementations ---

impl FunctorSt for OptionF {
    fn fmap_st<A: 'static, B: 'static>(fa: Option<A>, f: impl Fn(A) -> B + 'static) -> Option<B> {
        fa.map(f)
    }
}

impl ApplicativeSt for OptionF {
    fn pure_st<A: 'static>(a: A) -> Option<A> {
        Some(a)
    }
}

impl ChainSt for OptionF {
    fn chain_st<A: 'static, B: 'static>(
        fa: Option<A>,
        f: impl Fn(A) -> Option<B> + 'static,
    ) -> Option<B> {
        fa.and_then(f)
    }
}

impl<E: 'static> FunctorSt for ResultF<E> {
    fn fmap_st<A: 'static, B: 'static>(
        fa: Result<A, E>,
        f: impl Fn(A) -> B + 'static,
    ) -> Result<B, E> {
        fa.map(f)
    }
}

impl<E: 'static> ApplicativeSt for ResultF<E> {
    fn pure_st<A: 'static>(a: A) -> Result<A, E> {
        Ok(a)
    }
}

impl<E: 'static> ChainSt for ResultF<E> {
    fn chain_st<A: 'static, B: 'static>(
        fa: Result<A, E>,
        f: impl Fn(A) -> Result<B, E> + 'static,
    ) -> Result<B, E> {
        fa.and_then(f)
    }
}

impl FunctorSt for IdentityF {
    fn fmap_st<A: 'static, B: 'static>(fa: A, f: impl Fn(A) -> B + 'static) -> B {
        f(fa)
    }
}

impl ApplicativeSt for IdentityF {
    fn pure_st<A: 'static>(a: A) -> A {
        a
    }
}

impl ChainSt for IdentityF {
    fn chain_st<A: 'static, B: 'static>(fa: A, f: impl Fn(A) -> B + 'static) -> B {
        f(fa)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl FunctorSt for karpal_core::hkt::VecF {
    fn fmap_st<A: 'static, B: 'static>(fa: Vec<A>, f: impl Fn(A) -> B + 'static) -> Vec<B> {
        fa.into_iter().map(f).collect()
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl ApplicativeSt for karpal_core::hkt::VecF {
    fn pure_st<A: 'static>(a: A) -> Vec<A> {
        vec![a]
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl ChainSt for karpal_core::hkt::VecF {
    fn chain_st<A: 'static, B: 'static>(fa: Vec<A>, f: impl Fn(A) -> Vec<B> + 'static) -> Vec<B> {
        fa.into_iter().flat_map(f).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_functor_st() {
        assert_eq!(OptionF::fmap_st(Some(3), |x| x + 1), Some(4));
        assert_eq!(OptionF::fmap_st(None::<i32>, |x| x + 1), None);
    }

    #[test]
    fn option_applicative_st() {
        assert_eq!(OptionF::pure_st(42), Some(42));
    }

    #[test]
    fn option_chain_st() {
        assert_eq!(OptionF::chain_st(Some(3), |x| Some(x * 2)), Some(6));
        assert_eq!(OptionF::chain_st(None::<i32>, |x| Some(x * 2)), None);
    }

    #[test]
    fn result_functor_st() {
        assert_eq!(ResultF::<&str>::fmap_st(Ok(3), |x| x + 1), Ok(4));
    }

    #[test]
    fn identity_chain_st() {
        assert_eq!(IdentityF::chain_st(5, |x| x + 1), 6);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Functor identity: fmap(id, fa) == fa
        #[test]
        fn option_functor_identity(x in any::<Option<i32>>()) {
            prop_assert_eq!(OptionF::fmap_st(x.clone(), |a| a), x);
        }

        // Functor composition: fmap(g . f, fa) == fmap(g, fmap(f, fa))
        #[test]
        fn option_functor_composition(x in any::<Option<i16>>()) {
            let f = |a: i16| a.wrapping_add(1);
            let g = |a: i16| a.wrapping_mul(2);
            let left = OptionF::fmap_st(x.clone(), move |a| g(f(a)));
            let right = OptionF::fmap_st(OptionF::fmap_st(x, f), g);
            prop_assert_eq!(left, right);
        }

        // Chain associativity
        #[test]
        fn option_chain_associativity(x in any::<Option<i16>>()) {
            let f = |a: i16| Some(a.wrapping_add(1));
            let g = |a: i16| Some(a.wrapping_mul(2));
            let left = OptionF::chain_st(OptionF::chain_st(x.clone(), f), g);
            let right = OptionF::chain_st(x, move |a| OptionF::chain_st(f(a), g));
            prop_assert_eq!(left, right);
        }

        // Monad left identity: chain(pure(a), f) == f(a)
        #[test]
        fn option_monad_left_identity(a in any::<i16>()) {
            let f = |x: i16| Some(x.wrapping_add(1));
            let left = OptionF::chain_st(OptionF::pure_st(a), f);
            let right = f(a);
            prop_assert_eq!(left, right);
        }

        // Monad right identity: chain(m, pure) == m
        #[test]
        fn option_monad_right_identity(x in any::<Option<i32>>()) {
            let left = OptionF::chain_st(x.clone(), |a| OptionF::pure_st(a));
            prop_assert_eq!(left, x);
        }
    }
}
