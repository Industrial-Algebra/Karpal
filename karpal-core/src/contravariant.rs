use crate::hkt::HKT;

/// Contravariant functor: lifts a function `B -> A` into `F<A> -> F<B>`.
///
/// Laws:
/// - Identity: `contramap(id, fa) == fa`
/// - Composition: `contramap(f . g, fa) == contramap(g, contramap(f, fa))`
pub trait Contravariant: HKT {
    fn contramap<A: 'static, B>(fa: Self::Of<A>, f: impl Fn(B) -> A + 'static) -> Self::Of<B>;
}

/// Type constructor for predicates: `Of<A> = Box<dyn Fn(A) -> bool>`.
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct PredicateF;

#[cfg(any(feature = "std", feature = "alloc"))]
impl HKT for PredicateF {
    #[cfg(feature = "std")]
    type Of<T> = Box<dyn Fn(T) -> bool>;

    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    type Of<T> = alloc::boxed::Box<dyn Fn(T) -> bool>;
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl Contravariant for PredicateF {
    fn contramap<A: 'static, B>(fa: Self::Of<A>, f: impl Fn(B) -> A + 'static) -> Self::Of<B> {
        Box::new(move |b| fa(f(b)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predicate_contramap() {
        let is_positive: Box<dyn Fn(i32) -> bool> = Box::new(|x| x > 0);
        let str_len_positive = PredicateF::contramap(is_positive, |s: &str| s.len() as i32);
        assert!(str_len_positive("hello"));
        assert!(!str_len_positive(""));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Identity: contramap(id, fa)(x) == fa(x)
        #[test]
        fn predicate_identity(x in any::<i32>()) {
            let pred: Box<dyn Fn(i32) -> bool> = Box::new(|a| a > 0);
            let expected = pred(x);
            let result = PredicateF::contramap(pred, |a: i32| a);
            prop_assert_eq!(result(x), expected);
        }

        // Composition: contramap(f . g, fa) == contramap(g, contramap(f, fa))
        #[test]
        fn predicate_composition(x in any::<i16>()) {
            let pred: Box<dyn Fn(i32) -> bool> = Box::new(|a| a > 0);
            let f = |a: i16| a as i32;
            let g = |a: i16| a.wrapping_add(1);

            // contramap(f . g, pred)
            let pred1: Box<dyn Fn(i32) -> bool> = Box::new(|a| a > 0);
            let left = PredicateF::contramap(pred1, move |a: i16| f(g(a)));

            // contramap(g, contramap(f, pred))
            let inner = PredicateF::contramap(pred, f);
            let right = PredicateF::contramap(inner, g);

            prop_assert_eq!(left(x), right(x));
        }
    }
}
