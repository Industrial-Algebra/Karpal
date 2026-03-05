use core::marker::PhantomData;

use karpal_core::hkt::{HKT, HKT2};

/// Cokleisli arrow: `W<A> -> B` for a Comonad `W`.
///
/// `CokleisliF<W>` is a two-parameter type constructor where
/// `P<A, B> = Box<dyn Fn(W::Of<A>) -> B>`.
///
/// Composition requires `W::Of<A>: Clone`, which can't be expressed generically
/// with GATs. Use the [`impl_cokleisli`] macro to generate impls for specific comonads.
pub struct CokleisliF<W: HKT>(PhantomData<W>);

impl<W: HKT> HKT2 for CokleisliF<W> {
    type P<A, B> = Box<dyn Fn(W::Of<A>) -> B>;
}

/// Generate Semigroupoid + Category impls for `CokleisliF<$W>` where
/// `$W::Of<A>` is known to be `Clone` for `A: Clone`.
///
/// Usage: `impl_cokleisli!(IdentityF, OptionF, NonEmptyVecF, EnvF<E>);`
#[macro_export]
macro_rules! impl_cokleisli {
    ($W:ty) => {
        impl $crate::Semigroupoid for $crate::CokleisliF<$W> {
            fn compose<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
                f: Box<dyn Fn(<$W as karpal_core::hkt::HKT>::Of<B>) -> C>,
                g: Box<dyn Fn(<$W as karpal_core::hkt::HKT>::Of<A>) -> B>,
            ) -> Box<dyn Fn(<$W as karpal_core::hkt::HKT>::Of<A>) -> C>
            where
                <$W as karpal_core::hkt::HKT>::Of<A>: Clone,
                <$W as karpal_core::hkt::HKT>::Of<B>: Clone,
            {
                Box::new(move |wa: <$W as karpal_core::hkt::HKT>::Of<A>| {
                    let wb = <$W as karpal_core::extend::Extend>::extend(
                        wa,
                        |wa: &<$W as karpal_core::hkt::HKT>::Of<A>| g(wa.clone()),
                    );
                    f(wb)
                })
            }
        }

        impl $crate::Category for $crate::CokleisliF<$W> {
            fn id<A: Clone + 'static>() -> Box<dyn Fn(<$W as karpal_core::hkt::HKT>::Of<A>) -> A> {
                Box::new(|wa: <$W as karpal_core::hkt::HKT>::Of<A>| {
                    <$W as karpal_core::comonad::Comonad>::extract(&wa)
                })
            }
        }
    };
}

// Generate impls for standard comonads
impl_cokleisli!(karpal_core::hkt::IdentityF);
impl_cokleisli!(karpal_core::hkt::OptionF);
#[cfg(any(feature = "std", feature = "alloc"))]
impl_cokleisli!(karpal_core::hkt::NonEmptyVecF);

/// Generate CokleisliF impls for `EnvF<E>` with a specific environment type.
///
/// Usage: `impl_cokleisli_env!(String);`
#[macro_export]
macro_rules! impl_cokleisli_env {
    ($E:ty) => {
        impl $crate::Semigroupoid for $crate::CokleisliF<karpal_core::hkt::EnvF<$E>> {
            fn compose<A: Clone + 'static, B: Clone + 'static, C: Clone + 'static>(
                f: Box<dyn Fn(($E, B)) -> C>,
                g: Box<dyn Fn(($E, A)) -> B>,
            ) -> Box<dyn Fn(($E, A)) -> C>
            where
                ($E, A): Clone,
                ($E, B): Clone,
            {
                Box::new(move |wa: ($E, A)| {
                    let wb = <karpal_core::hkt::EnvF<$E> as karpal_core::extend::Extend>::extend(
                        wa,
                        |wa: &($E, A)| g(wa.clone()),
                    );
                    f(wb)
                })
            }
        }

        impl $crate::Category for $crate::CokleisliF<karpal_core::hkt::EnvF<$E>> {
            fn id<A: Clone + 'static>() -> Box<dyn Fn(($E, A)) -> A> {
                Box::new(|wa: ($E, A)| {
                    <karpal_core::hkt::EnvF<$E> as karpal_core::comonad::Comonad>::extract(&wa)
                })
            }
        }
    };
}

// Common EnvF instances
impl_cokleisli_env!(i32);
impl_cokleisli_env!(String);

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::{IdentityF, NonEmptyVec, NonEmptyVecF, OptionF};

    use crate::category::Category;
    use crate::semigroupoid::Semigroupoid;

    type CoId = CokleisliF<IdentityF>;
    type CoOpt = CokleisliF<OptionF>;
    type CoNev = CokleisliF<NonEmptyVecF>;
    type CoEnvI32 = CokleisliF<karpal_core::hkt::EnvF<i32>>;

    #[test]
    fn identity_id() {
        let id = CoId::id::<i32>();
        assert_eq!(id(42), 42);
    }

    #[test]
    fn identity_compose() {
        let f: Box<dyn Fn(i32) -> i32> = Box::new(|x| x + 1);
        let g: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2);
        let fg = CoId::compose(f, g);
        assert_eq!(fg(3), 7); // (3 * 2) + 1
    }

    #[test]
    fn option_id() {
        let id = CoOpt::id::<i32>();
        assert_eq!(id(Some(42)), 42);
    }

    #[test]
    fn option_compose() {
        let f: Box<dyn Fn(Option<i32>) -> i32> = Box::new(|opt| opt.unwrap_or(0) + 1);
        let g: Box<dyn Fn(Option<i32>) -> i32> = Box::new(|opt| opt.unwrap_or(0) * 2);
        let fg = CoOpt::compose(f, g);
        assert_eq!(fg(Some(3)), 7); // extend gives Some(6), then +1 = 7
    }

    #[test]
    fn nonemptyvec_id() {
        let id = CoNev::id::<i32>();
        let nev = NonEmptyVec::new(42, vec![1, 2]);
        assert_eq!(id(nev), 42); // extract = head
    }

    #[test]
    fn nonemptyvec_compose() {
        let f: Box<dyn Fn(NonEmptyVec<i32>) -> i32> = Box::new(|nev| nev.head + 1);
        let g: Box<dyn Fn(NonEmptyVec<i32>) -> i32> = Box::new(|nev| nev.iter().sum());
        let fg = CoNev::compose(f, g);
        let nev = NonEmptyVec::new(1, vec![2, 3]);
        // extend(nev, g) produces sums of suffixes: [6, 5, 3]
        // then f extracts head + 1 = 7
        assert_eq!(fg(nev), 7);
    }

    #[test]
    fn env_id() {
        let id = CoEnvI32::id::<String>();
        assert_eq!(id((42, "hello".to_string())), "hello");
    }

    #[test]
    fn env_compose() {
        let f: Box<dyn Fn((i32, String)) -> String> = Box::new(|(e, s)| format!("{}:{}", e, s));
        let g: Box<dyn Fn((i32, i32)) -> String> = Box::new(|(e, a)| format!("{}+{}", e, a));
        let fg = CoEnvI32::compose(f, g);
        // extend((10, 5), g) = (10, g(&(10, 5))) = (10, "10+5")
        // f((10, "10+5")) = "10:10+5"
        assert_eq!(fg((10, 5)), "10:10+5");
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::hkt::{IdentityF, NonEmptyVec, NonEmptyVecF};

    use crate::category::Category;
    use crate::semigroupoid::Semigroupoid;

    use proptest::prelude::*;

    type CoId = CokleisliF<IdentityF>;
    type CoNev = CokleisliF<NonEmptyVecF>;

    fn nonemptyvec_strategy<T: core::fmt::Debug + Clone + 'static>(
        elem: impl Strategy<Value = T> + Clone + 'static,
    ) -> impl Strategy<Value = NonEmptyVec<T>> {
        (elem.clone(), prop::collection::vec(elem, 0..5))
            .prop_map(|(head, tail)| NonEmptyVec::new(head, tail))
    }

    proptest! {
        // Category left identity: compose(id(), f) == f
        #[test]
        fn identity_left_identity(x in any::<i16>()) {
            let f = || -> Box<dyn Fn(i16) -> i16> { Box::new(|a| a.wrapping_mul(2)) };
            let left = CoId::compose(CoId::id(), f());
            let right = f();
            prop_assert_eq!(left(x), right(x));
        }

        // Category right identity: compose(f, id()) == f
        #[test]
        fn identity_right_identity(x in any::<i16>()) {
            let f = || -> Box<dyn Fn(i16) -> i16> { Box::new(|a| a.wrapping_mul(2)) };
            let left = CoId::compose(f(), CoId::id());
            let right = f();
            prop_assert_eq!(left(x), right(x));
        }

        // Semigroupoid associativity for IdentityF
        #[test]
        fn identity_associativity(x in any::<i16>()) {
            let f = || -> Box<dyn Fn(i16) -> i16> { Box::new(|a| a.wrapping_add(1)) };
            let g = || -> Box<dyn Fn(i16) -> i16> { Box::new(|a| a.wrapping_mul(2)) };
            let h = || -> Box<dyn Fn(i16) -> i16> { Box::new(|a| a.wrapping_sub(3)) };

            let left = CoId::compose(f(), CoId::compose(g(), h()));
            let right = CoId::compose(CoId::compose(f(), g()), h());
            prop_assert_eq!(left(x), right(x));
        }

        // Category left identity for NonEmptyVecF
        #[test]
        fn nev_left_identity(w in nonemptyvec_strategy(any::<i8>())) {
            let f = || -> Box<dyn Fn(NonEmptyVec<i8>) -> i8> { Box::new(|nev| nev.head.wrapping_mul(2)) };
            let left = CoNev::compose(CoNev::id(), f());
            let right = f();
            prop_assert_eq!(left(w.clone()), right(w));
        }

        // Category right identity for NonEmptyVecF
        #[test]
        fn nev_right_identity(w in nonemptyvec_strategy(any::<i8>())) {
            let f = || -> Box<dyn Fn(NonEmptyVec<i8>) -> i8> { Box::new(|nev| nev.head.wrapping_mul(2)) };
            let left = CoNev::compose(f(), CoNev::id());
            let right = f();
            prop_assert_eq!(left(w.clone()), right(w));
        }
    }
}
