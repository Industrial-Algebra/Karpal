use crate::comonad::Comonad;
use crate::hkt::EnvF;

/// ComonadEnv: a Comonad with access to an environment value.
///
/// Laws:
/// - `extract(local(wa, f)) == extract(wa)` (local doesn't change the focus)
pub trait ComonadEnv<E>: Comonad {
    fn ask<A>(wa: &Self::Of<A>) -> E;
    fn local<A>(wa: Self::Of<A>, f: impl Fn(E) -> E) -> Self::Of<A>;
}

impl<E: Clone> ComonadEnv<E> for EnvF<E> {
    fn ask<A>(wa: &(E, A)) -> E {
        wa.0.clone()
    }

    fn local<A>(wa: (E, A), f: impl Fn(E) -> E) -> (E, A) {
        (f(wa.0), wa.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_ask() {
        let w = ("hello", 42);
        assert_eq!(EnvF::<&str>::ask(&w), "hello");
    }

    #[test]
    fn env_local() {
        let w = (10i32, "value");
        let result = EnvF::<i32>::local(w, |e| e * 2);
        assert_eq!(result, (20, "value"));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // extract(local(wa, f)) == extract(wa)
        #[test]
        fn env_local_preserves_extract(e in any::<i16>(), a in any::<i32>()) {
            let w = (e, a);
            let localed = EnvF::<i16>::local(w, |e| e.wrapping_add(1));
            let left = EnvF::<i16>::extract(&localed);
            let right = EnvF::<i16>::extract(&(e, a));
            prop_assert_eq!(left, right);
        }
    }
}
