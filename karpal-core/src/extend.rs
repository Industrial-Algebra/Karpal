use crate::functor::Functor;
use crate::hkt::{EnvF, IdentityF, OptionF};
#[cfg(any(feature = "std", feature = "alloc"))]
use crate::hkt::{NonEmptyVec, NonEmptyVecF};

/// Extend: the dual of Chain. Enables cooperative "context-aware" computation.
///
/// Given a value in context `W<A>` and a function `&W<A> -> B` that can
/// inspect the full context, `extend` applies that function at every
/// "position" in the structure, producing `W<B>`.
///
/// Laws:
/// - Associativity: `extend(f, extend(g, w)) == extend(|w| f(&extend(g, w.clone())), w)`
pub trait Extend: Functor {
    fn extend<A, B>(wa: Self::Of<A>, f: impl Fn(&Self::Of<A>) -> B) -> Self::Of<B>
    where
        A: Clone;

    fn duplicate<A>(wa: Self::Of<A>) -> Self::Of<Self::Of<A>>
    where
        A: Clone,
        Self::Of<A>: Clone,
    {
        Self::extend(wa, |w| w.clone())
    }
}

impl Extend for IdentityF {
    fn extend<A, B>(wa: A, f: impl Fn(&A) -> B) -> B
    where
        A: Clone,
    {
        f(&wa)
    }
}

impl Extend for OptionF {
    fn extend<A, B>(wa: Option<A>, f: impl Fn(&Option<A>) -> B) -> Option<B>
    where
        A: Clone,
    {
        if wa.is_some() { Some(f(&wa)) } else { None }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Extend for NonEmptyVecF {
    fn extend<A, B>(wa: NonEmptyVec<A>, f: impl Fn(&NonEmptyVec<A>) -> B) -> NonEmptyVec<B>
    where
        A: Clone,
    {
        // Apply f to each suffix of the NonEmptyVec
        let suffixes = wa.tails();
        let head = f(&suffixes.head);
        let tail = suffixes.tail.iter().map(&f).collect();
        NonEmptyVec::new(head, tail)
    }
}

impl<E> Extend for EnvF<E> {
    fn extend<A, B>(wa: (E, A), f: impl Fn(&(E, A)) -> B) -> (E, B)
    where
        A: Clone,
    {
        let b = f(&wa);
        (wa.0, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_extend() {
        let result = IdentityF::extend(42, |w| w + 1);
        assert_eq!(result, 43);
    }

    #[test]
    fn option_extend_some() {
        let result = OptionF::extend(Some(3), |opt| match opt {
            Some(x) => x * 2,
            None => 0,
        });
        assert_eq!(result, Some(6));
    }

    #[test]
    fn option_extend_none() {
        let result = OptionF::extend(None::<i32>, |opt| match opt {
            Some(x) => x * 2,
            None => 0,
        });
        assert_eq!(result, None);
    }

    #[test]
    fn nonemptyvec_extend() {
        let nev = NonEmptyVec::new(1, vec![2, 3]);
        // Sum of each suffix
        let result = NonEmptyVecF::extend(nev, |w| w.iter().sum::<i32>());
        // Suffixes: [1,2,3], [2,3], [3]
        // Sums: 6, 5, 3
        assert_eq!(result, NonEmptyVec::new(6, vec![5, 3]));
    }

    #[test]
    fn env_extend() {
        let result = EnvF::<&str>::extend(("hello", 42), |&(env, val)| format!("{}: {}", env, val));
        assert_eq!(result, ("hello", "hello: 42".to_string()));
    }

    #[test]
    fn identity_duplicate() {
        let result = IdentityF::duplicate(42);
        assert_eq!(result, 42);
    }

    #[test]
    fn option_duplicate() {
        let result = OptionF::duplicate(Some(42));
        assert_eq!(result, Some(Some(42)));
    }

    #[test]
    fn nonemptyvec_duplicate() {
        let nev = NonEmptyVec::new(1, vec![2, 3]);
        let result = NonEmptyVecF::duplicate(nev);
        assert_eq!(result.head, NonEmptyVec::new(1, vec![2, 3]));
        assert_eq!(result.tail.len(), 2);
        assert_eq!(result.tail[0], NonEmptyVec::new(2, vec![3]));
        assert_eq!(result.tail[1], NonEmptyVec::new(3, vec![]));
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
        // Associativity: extend(f, extend(g, w)) == extend(|w| f(&extend(g, w.clone())), w)
        #[test]
        fn option_associativity(x in any::<Option<i16>>()) {
            let f = |opt: &Option<i16>| opt.map_or(0i16, |v| v.wrapping_add(1));
            let g = |opt: &Option<i16>| opt.map_or(0i16, |v| v.wrapping_mul(2));

            let left = OptionF::extend(OptionF::extend(x.clone(), g), f);
            let right = OptionF::extend(x, |w| f(&OptionF::extend(w.clone(), g)));
            prop_assert_eq!(left, right);
        }

        #[test]
        fn nonemptyvec_associativity(w in nonemptyvec_strategy(any::<i8>())) {
            let f = |nev: &NonEmptyVec<i8>| nev.head.wrapping_add(1);
            let g = |nev: &NonEmptyVec<i8>| nev.head.wrapping_mul(2);

            let left = NonEmptyVecF::extend(NonEmptyVecF::extend(w.clone(), g), f);
            let right = NonEmptyVecF::extend(w, |w| f(&NonEmptyVecF::extend(w.clone(), g)));
            prop_assert_eq!(left, right);
        }
    }
}
