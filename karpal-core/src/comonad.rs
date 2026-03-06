use crate::extend::Extend;
use crate::hkt::{EnvF, IdentityF, OptionF};
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::hkt::{NonEmptyVec, NonEmptyVecF};

/// Comonad: the categorical dual of Monad.
///
/// A Comonad can `extract` a value from a context, and `extend` a
/// context-aware function over the entire structure.
///
/// Laws:
/// - Left identity: `extract(&extend(w, f)) == f(&w)`
/// - Right identity: `extend(w, |w| extract(w)) == w`
/// - Associativity (inherited from Extend)
pub trait Comonad: Extend {
    fn extract<A: Clone>(wa: &Self::Of<A>) -> A;
}

impl Comonad for IdentityF {
    fn extract<A: Clone>(wa: &A) -> A {
        wa.clone()
    }
}

impl Comonad for OptionF {
    fn extract<A: Clone>(wa: &Option<A>) -> A {
        wa.as_ref()
            .expect("Comonad::extract called on None")
            .clone()
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Comonad for NonEmptyVecF {
    fn extract<A: Clone>(wa: &NonEmptyVec<A>) -> A {
        wa.head.clone()
    }
}

impl<E> Comonad for EnvF<E> {
    fn extract<A: Clone>(wa: &(E, A)) -> A {
        wa.1.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_extract() {
        assert_eq!(IdentityF::extract(&42), 42);
    }

    #[test]
    fn option_extract() {
        assert_eq!(OptionF::extract(&Some(42)), 42);
    }

    #[test]
    #[should_panic(expected = "Comonad::extract called on None")]
    fn option_extract_none_panics() {
        OptionF::extract(&None::<i32>);
    }

    #[test]
    fn nonemptyvec_extract() {
        let nev = NonEmptyVec::new(1, vec![2, 3]);
        assert_eq!(NonEmptyVecF::extract(&nev), 1);
    }

    #[test]
    fn env_extract() {
        assert_eq!(EnvF::<&str>::extract(&("hello", 42)), 42);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    fn nonemptyvec_strategy<T: core::fmt::Debug + Clone + 'static>(
        elem: impl Strategy<Value = T> + Clone + 'static,
    ) -> impl Strategy<Value = NonEmptyVec<T>> {
        (elem.clone(), prop::collection::vec(elem, 0..5))
            .prop_map(|(head, tail)| NonEmptyVec::new(head, tail))
    }

    proptest! {
        // Left identity: extract(&extend(w, f)) == f(&w)
        #[test]
        fn identity_left_identity(x in any::<i32>()) {
            let f = |w: &i32| w.wrapping_add(1);
            let left = IdentityF::extract(&IdentityF::extend(x, f));
            let right = f(&x);
            prop_assert_eq!(left, right);
        }

        // Right identity: extend(w, |w| extract(w)) == w
        #[test]
        fn identity_right_identity(x in any::<i32>()) {
            let result = IdentityF::extend(x, |w| IdentityF::extract(w));
            prop_assert_eq!(result, x);
        }

        #[test]
        fn nonemptyvec_left_identity(w in nonemptyvec_strategy(any::<i16>())) {
            let f = |nev: &NonEmptyVec<i16>| nev.head.wrapping_add(1);
            let left = NonEmptyVecF::extract(&NonEmptyVecF::extend(w.clone(), f));
            let right = f(&w);
            prop_assert_eq!(left, right);
        }

        #[test]
        fn nonemptyvec_right_identity(w in nonemptyvec_strategy(any::<i16>())) {
            let result = NonEmptyVecF::extend(w.clone(), |w| NonEmptyVecF::extract(w));
            prop_assert_eq!(result, w);
        }

        #[test]
        fn env_left_identity(e in any::<i8>(), a in any::<i16>()) {
            let w = (e, a);
            let f = |wa: &(i8, i16)| wa.1.wrapping_add(1);
            let left = EnvF::<i8>::extract(&EnvF::<i8>::extend(w, f));
            let right = f(&(e, a));
            prop_assert_eq!(left, right);
        }

        #[test]
        fn env_right_identity(e in any::<i8>(), a in any::<i16>()) {
            let w = (e, a);
            let result = EnvF::<i8>::extend(w, |w| EnvF::<i8>::extract(w));
            prop_assert_eq!(result, (e, a));
        }
    }
}
