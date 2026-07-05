// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use karpal_core::applicative::Applicative;
use karpal_core::chain::Chain;
use karpal_core::functor::Functor;
use karpal_core::hkt::HKT;

// ---------------------------------------------------------------------------
// 'static-bounded typeclass hierarchy (St variants)
// ---------------------------------------------------------------------------

/// Functor with `'static` bounds on type parameters.
///
/// This mirrors [`Functor`] but adds `'static` bounds required by types that
/// use `Box<dyn Fn>` internally (monad transformers, `ReaderF`, etc.).
///
/// # Bridge to the base hierarchy
///
/// Any type that implements [`Functor`] automatically implements `FunctorSt`
/// via a blanket impl. You do NOT need to implement both — just implement
/// `Functor`, and `FunctorSt` is provided for free.
///
/// Types that CANNOT implement `Functor` (because they use `Box<dyn Fn>` and
/// need `'static`) should implement `FunctorSt` directly. This includes the
/// monad transformers in this crate.
pub trait FunctorSt: HKT {
    fn fmap_st<A: 'static, B: 'static>(
        fa: Self::Of<A>,
        f: impl Fn(A) -> B + 'static,
    ) -> Self::Of<B>;
}

/// Applicative with `'static` bounds on type parameters.
///
/// Automatically implemented for any type that implements [`Applicative`].
/// See [`FunctorSt`] for the rationale behind the `St` hierarchy.
pub trait ApplicativeSt: FunctorSt {
    fn pure_st<A: 'static>(a: A) -> Self::Of<A>;
}

/// Chain (monadic bind) with `'static` bounds on type parameters.
///
/// Automatically implemented for any type that implements [`Chain`].
/// See [`FunctorSt`] for the rationale behind the `St` hierarchy.
pub trait ChainSt: FunctorSt {
    fn chain_st<A: 'static, B: 'static>(
        fa: Self::Of<A>,
        f: impl Fn(A) -> Self::Of<B> + 'static,
    ) -> Self::Of<B>;
}

// ---------------------------------------------------------------------------
// Blanket impls: bridge base hierarchy → St hierarchy
// ---------------------------------------------------------------------------

/// Any `Functor` is automatically a `FunctorSt`.
///
/// This eliminates the need for manual `FunctorSt` impls on base types
/// (`OptionF`, `ResultF`, `IdentityF`, `VecF`, etc.). The `'static` bounds
/// in `fmap_st` are strictly stronger than what `fmap` requires, so the
/// delegation is always sound.
impl<F: Functor> FunctorSt for F {
    fn fmap_st<A: 'static, B: 'static>(
        fa: Self::Of<A>,
        f: impl Fn(A) -> B + 'static,
    ) -> Self::Of<B> {
        F::fmap(fa, f)
    }
}

/// Any `Applicative` is automatically an `ApplicativeSt`.
impl<F: Applicative> ApplicativeSt for F {
    fn pure_st<A: 'static>(a: A) -> Self::Of<A> {
        F::pure(a)
    }
}

/// Any `Chain` is automatically a `ChainSt`.
impl<F: Chain> ChainSt for F {
    fn chain_st<A: 'static, B: 'static>(
        fa: Self::Of<A>,
        f: impl Fn(A) -> Self::Of<B> + 'static,
    ) -> Self::Of<B> {
        F::chain(fa, f)
    }
}

// ---------------------------------------------------------------------------
// Note on transformer types
// ---------------------------------------------------------------------------
//
// Monad transformers (ReaderTF, StateTF, WriterTF, ExceptTF) implement
// FunctorSt/ChainSt DIRECTLY because they use Box<dyn Fn> internally and
// cannot implement the base Functor trait (which lacks 'static bounds).
// These impls live in each transformer's module and do NOT conflict with
// the blanket impls above, because transformers do not implement Functor.

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::{IdentityF, OptionF, ResultF};

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

    // --- Bridge tests: verify base types work with St bounds via blanket ---

    #[test]
    fn blanket_bridge_option() {
        // OptionF implements Functor (base), so FunctorSt is automatic.
        fn requires_functor_st<F: FunctorSt>(fa: F::Of<i32>) -> F::Of<String> {
            F::fmap_st(fa, |x| format!("val={}", x))
        }
        assert_eq!(
            requires_functor_st::<OptionF>(Some(42)),
            Some("val=42".to_string())
        );
    }

    #[test]
    fn blanket_bridge_result() {
        fn requires_chain_st<E: 'static, F: ChainSt + ApplicativeSt>(fa: F::Of<i32>) -> F::Of<i32> {
            F::chain_st(fa, |x| {
                F::chain_st(F::pure_st::<i32>(x), |y| F::pure_st(y + 1))
            })
        }
        assert_eq!(requires_chain_st::<&str, ResultF<&str>>(Ok(5)), Ok(6));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::hkt::OptionF;
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
