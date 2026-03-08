#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;

use core::marker::PhantomData;

use karpal_core::applicative::Applicative;
use karpal_core::chain::Chain;
use karpal_core::functor::Functor;
use karpal_core::hkt::HKT;
use karpal_core::natural::NaturalTransformation;

/// Free Monad — builds a monadic computation as a data structure.
///
/// `Free<F, A>` represents a program where `F` describes the available
/// effects and `A` is the result type. Programs are built with `pure`
/// and `lift_f`, composed with `chain`, and interpreted with `fold_map`
/// using a natural transformation into any target monad.
///
/// ```text
/// Pure(a)              — a finished computation returning a
/// Roll(F<Free<F, A>>)  — one layer of effect wrapping a continuation
/// ```
pub enum Free<F: HKT, A> {
    /// A pure value — the computation is finished.
    Pure(A),
    /// A layer of effect `F` wrapping a continuation.
    Roll(Box<F::Of<Free<F, A>>>),
}

impl<F: HKT, A> Free<F, A> {
    /// Wrap a pure value into the free monad.
    pub fn pure(a: A) -> Self {
        Free::Pure(a)
    }
}

impl<F: HKT + Functor, A> Free<F, A> {
    /// Lift a single effect `F<A>` into the free monad.
    pub fn lift_f(fa: F::Of<A>) -> Self {
        Free::Roll(Box::new(F::fmap(fa, Free::Pure)))
    }

    /// Map a function over the result of this computation.
    pub fn fmap<B>(self, f: impl Fn(A) -> B) -> Free<F, B> {
        self.fmap_inner(&f)
    }

    fn fmap_inner<B>(self, f: &dyn Fn(A) -> B) -> Free<F, B> {
        match self {
            Free::Pure(a) => Free::Pure(f(a)),
            Free::Roll(ff) => Free::Roll(Box::new(F::fmap(*ff, |child| child.fmap_inner(f)))),
        }
    }

    /// Monadic bind — sequence this computation with a function that
    /// produces the next computation.
    pub fn chain<B>(self, f: impl Fn(A) -> Free<F, B>) -> Free<F, B> {
        self.chain_inner(&f)
    }

    fn chain_inner<B>(self, f: &dyn Fn(A) -> Free<F, B>) -> Free<F, B> {
        match self {
            Free::Pure(a) => f(a),
            Free::Roll(ff) => Free::Roll(Box::new(F::fmap(*ff, |child| child.chain_inner(f)))),
        }
    }

    /// Interpret this free monad into a target monad `M` using a natural
    /// transformation `NT: F ~> M`.
    ///
    /// This is the core interpreter: it collapses the free structure by
    /// translating each `F` effect into `M` and sequencing with `M::chain`.
    pub fn fold_map<M, NT>(self) -> M::Of<A>
    where
        M: Applicative + Chain,
        NT: NaturalTransformation<F, M>,
    {
        match self {
            Free::Pure(a) => M::pure(a),
            Free::Roll(ff) => {
                let mapped = F::fmap(*ff, |child| child.fold_map::<M, NT>());
                let m_ma: M::Of<M::Of<A>> = NT::transform(mapped);
                M::chain(m_ma, |x| x)
            }
        }
    }
}

/// HKT marker for `Free<F, _>`.
pub struct FreeF<F: HKT>(PhantomData<F>);

impl<F: HKT> HKT for FreeF<F> {
    type Of<T> = Free<F, T>;
}

impl<F: HKT + Functor> Functor for FreeF<F> {
    fn fmap<A, B>(fa: Free<F, A>, f: impl Fn(A) -> B) -> Free<F, B> {
        fa.fmap(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::OptionF;

    #[test]
    fn pure_value() {
        let free = Free::<OptionF, i32>::pure(42);
        match free {
            Free::Pure(v) => assert_eq!(v, 42),
            Free::Roll(_) => panic!("expected Pure"),
        }
    }

    #[test]
    fn lift_f_some() {
        let free = Free::<OptionF, i32>::lift_f(Some(1));
        match free {
            Free::Roll(ff) => match *ff {
                Some(Free::Pure(v)) => assert_eq!(v, 1),
                _ => panic!("expected Some(Pure(1))"),
            },
            Free::Pure(_) => panic!("expected Roll"),
        }
    }

    #[test]
    fn fmap_pure() {
        let free = Free::<OptionF, i32>::pure(2).fmap(|x| x * 3);
        match free {
            Free::Pure(v) => assert_eq!(v, 6),
            Free::Roll(_) => panic!("expected Pure"),
        }
    }

    #[test]
    fn fmap_roll() {
        let free = Free::<OptionF, i32>::lift_f(Some(5)).fmap(|x| x + 10);
        match free {
            Free::Roll(ff) => match *ff {
                Some(Free::Pure(v)) => assert_eq!(v, 15),
                _ => panic!("expected Some(Pure(15))"),
            },
            Free::Pure(_) => panic!("expected Roll"),
        }
    }

    #[test]
    fn chain_pure() {
        let free = Free::<OptionF, i32>::pure(1).chain(|x| Free::pure(x + 1));
        match free {
            Free::Pure(v) => assert_eq!(v, 2),
            Free::Roll(_) => panic!("expected Pure"),
        }
    }

    #[test]
    fn chain_roll() {
        let free = Free::<OptionF, i32>::lift_f(Some(10)).chain(|x| Free::pure(x * 2));
        // Roll(Some(Pure(10))).chain(f)
        // = Roll(fmap(Some(Pure(10)), |child| child.chain(f)))
        // = Roll(Some(Pure(10).chain(f)))
        // = Roll(Some(f(10)))
        // = Roll(Some(Pure(20)))
        match free {
            Free::Roll(ff) => match *ff {
                Some(Free::Pure(v)) => assert_eq!(v, 20),
                _ => panic!("expected Some(Pure(20))"),
            },
            Free::Pure(_) => panic!("expected Roll"),
        }
    }

    #[test]
    fn chain_associativity() {
        let _m = Free::<OptionF, i32>::pure(5);
        let _f = |x: i32| Free::<OptionF, i32>::pure(x + 1);
        let _g = |x: i32| Free::<OptionF, i32>::pure(x * 2);

        // m.chain(f).chain(g)
        let left = Free::<OptionF, i32>::pure(5)
            .chain(|x| Free::pure(x + 1))
            .chain(|x| Free::pure(x * 2));

        // m.chain(|x| f(x).chain(g))
        let right = Free::<OptionF, i32>::pure(5)
            .chain(|x| Free::<OptionF, i32>::pure(x + 1).chain(|y| Free::pure(y * 2)));

        match (left, right) {
            (Free::Pure(l), Free::Pure(r)) => assert_eq!(l, r),
            _ => panic!("expected both Pure"),
        }
    }

    // Natural transformation: Option ~> Option (identity)
    struct OptionId;
    impl NaturalTransformation<OptionF, OptionF> for OptionId {
        fn transform<A>(fa: Option<A>) -> Option<A> {
            fa
        }
    }

    #[test]
    fn fold_map_pure() {
        let free = Free::<OptionF, i32>::pure(42);
        let result = free.fold_map::<OptionF, OptionId>();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn fold_map_roll() {
        let free = Free::<OptionF, i32>::lift_f(Some(10));
        let result = free.fold_map::<OptionF, OptionId>();
        assert_eq!(result, Some(10));
    }

    #[test]
    fn fold_map_chain_then_interpret() {
        let free = Free::<OptionF, i32>::lift_f(Some(3)).chain(|x| Free::lift_f(Some(x * 10)));
        let result = free.fold_map::<OptionF, OptionId>();
        assert_eq!(result, Some(30));
    }

    #[test]
    fn functor_impl_works() {
        let free = Free::<OptionF, i32>::pure(5);
        let result = <FreeF<OptionF> as Functor>::fmap(free, |x| x + 10);
        match result {
            Free::Pure(v) => assert_eq!(v, 15),
            Free::Roll(_) => panic!("expected Pure"),
        }
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::hkt::OptionF;
    use proptest::prelude::*;

    // Helper to extract Pure value for comparison
    fn extract_pure<F: HKT, A>(free: Free<F, A>) -> Option<A> {
        match free {
            Free::Pure(a) => Some(a),
            Free::Roll(_) => None,
        }
    }

    proptest! {
        // Functor identity: fmap(id, fa) == fa
        #[test]
        fn functor_identity(x in any::<i32>()) {
            let free = Free::<OptionF, i32>::pure(x);
            let result = free.fmap(|a| a);
            prop_assert_eq!(extract_pure(result), Some(x));
        }

        // Functor composition: fmap(g . f, fa) == fmap(g, fmap(f, fa))
        #[test]
        fn functor_composition(x in any::<i32>()) {
            let f = |a: i32| a.wrapping_add(1);
            let g = |a: i32| a.wrapping_mul(2);

            let left = Free::<OptionF, i32>::pure(x).fmap(|a| g(f(a)));
            let right = Free::<OptionF, i32>::pure(x).fmap(f).fmap(g);
            prop_assert_eq!(extract_pure(left), extract_pure(right));
        }

        // Monad left identity: pure(a).chain(f) == f(a)
        #[test]
        fn monad_left_identity(x in any::<i32>()) {
            let f = |a: i32| Free::<OptionF, i32>::pure(a.wrapping_mul(2));
            let left = Free::<OptionF, i32>::pure(x).chain(&f);
            let right = f(x);
            prop_assert_eq!(extract_pure(left), extract_pure(right));
        }

        // Monad right identity: m.chain(pure) == m
        #[test]
        fn monad_right_identity(x in any::<i32>()) {
            let m = Free::<OptionF, i32>::pure(x);
            let result = m.chain(Free::pure);
            prop_assert_eq!(extract_pure(result), Some(x));
        }
    }
}
