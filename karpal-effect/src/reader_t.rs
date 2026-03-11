use core::marker::PhantomData;

use karpal_core::hkt::HKT;

use crate::classes::{ApplicativeSt, ChainSt, FunctorSt};
use crate::trans::MonadTrans;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::rc::Rc;
#[cfg(feature = "std")]
use std::rc::Rc;

/// ReaderT monad transformer: adds environment-passing to an inner monad.
///
/// `ReaderTF<E, M>::Of<A> = Box<dyn Fn(E) -> M::Of<A>>`
///
/// When the inner monad is `IdentityF`, this is equivalent to `ReaderF<E>`.
pub struct ReaderTF<E, M>(PhantomData<(E, M)>);

impl<E: 'static, M: HKT> HKT for ReaderTF<E, M>
where
    for<'a> M: 'static,
{
    type Of<A> = Box<dyn Fn(E) -> M::Of<A>>;
}

impl<E: 'static, M: FunctorSt + 'static> MonadTrans<M> for ReaderTF<E, M> {
    fn lift<A: 'static>(ma: M::Of<A>) -> Box<dyn Fn(E) -> M::Of<A>>
    where
        M::Of<A>: Clone,
    {
        Box::new(move |_| ma.clone())
    }
}

/// ReaderT `pure`: wrap a value, ignoring the environment.
pub fn reader_t_pure<E: 'static, M: ApplicativeSt + 'static, A: Clone + 'static>(
    a: A,
) -> Box<dyn Fn(E) -> M::Of<A>> {
    Box::new(move |_| M::pure_st(a.clone()))
}

/// ReaderT `fmap`: apply a function to the result.
pub fn reader_t_fmap<E: 'static, M: FunctorSt + 'static, A: 'static, B: 'static>(
    fa: Box<dyn Fn(E) -> M::Of<A>>,
    f: impl Fn(A) -> B + 'static,
) -> Box<dyn Fn(E) -> M::Of<B>> {
    let f_rc = Rc::new(f);
    Box::new(move |e| {
        let ma = fa(e);
        let f_inner = f_rc.clone();
        M::fmap_st(ma, move |a| f_inner(a))
    })
}

/// ReaderT `chain`: sequence environment-passing computations.
///
/// The environment is shared (not threaded) between computations.
pub fn reader_t_chain<E: Clone + 'static, M: ChainSt + 'static, A: 'static, B: 'static>(
    fa: Box<dyn Fn(E) -> M::Of<A>>,
    f: impl Fn(A) -> Box<dyn Fn(E) -> M::Of<B>> + 'static,
) -> Box<dyn Fn(E) -> M::Of<B>> {
    let f_rc = Rc::new(f);
    Box::new(move |e: E| {
        let ma = fa(e.clone());
        let e2 = e;
        let f_inner = f_rc.clone();
        M::chain_st(ma, move |a| {
            let reader_b = f_inner(a);
            reader_b(e2.clone())
        })
    })
}

/// ReaderT `ask`: get the current environment.
pub fn reader_t_ask<E: Clone + 'static, M: ApplicativeSt + 'static>() -> Box<dyn Fn(E) -> M::Of<E>>
{
    Box::new(|e| M::pure_st(e))
}

/// ReaderT `local`: modify the environment for a sub-computation.
pub fn reader_t_local<E: 'static, M: HKT + 'static, A: 'static>(
    f: impl Fn(E) -> E + 'static,
    reader: Box<dyn Fn(E) -> M::Of<A>>,
) -> Box<dyn Fn(E) -> M::Of<A>> {
    Box::new(move |e| reader(f(e)))
}

/// ReaderT `reader`: create a computation from a function on the environment.
pub fn reader_t_reader<E: 'static, M: ApplicativeSt + 'static, A: 'static>(
    f: impl Fn(E) -> A + 'static,
) -> Box<dyn Fn(E) -> M::Of<A>> {
    Box::new(move |e| M::pure_st(f(e)))
}

/// ReaderT `run`: run the computation with an environment.
pub fn reader_t_run<E, M: HKT, A>(reader: &dyn Fn(E) -> M::Of<A>, env: E) -> M::Of<A> {
    reader(env)
}

// --- FunctorSt / ChainSt for ReaderTF ---
// Note: ApplicativeSt is not implemented for ReaderTF because pure_st
// cannot produce a Box<dyn Fn(E) -> M::Of<A>> from a single A without Clone.
// Use the standalone reader_t_pure function instead (which requires A: Clone).

impl<E: 'static, M: FunctorSt + 'static> FunctorSt for ReaderTF<E, M> {
    fn fmap_st<A: 'static, B: 'static>(
        fa: Box<dyn Fn(E) -> M::Of<A>>,
        f: impl Fn(A) -> B + 'static,
    ) -> Box<dyn Fn(E) -> M::Of<B>> {
        reader_t_fmap::<E, M, A, B>(fa, f)
    }
}

impl<E: Clone + 'static, M: ChainSt + 'static> ChainSt for ReaderTF<E, M> {
    fn chain_st<A: 'static, B: 'static>(
        fa: Box<dyn Fn(E) -> M::Of<A>>,
        f: impl Fn(A) -> Box<dyn Fn(E) -> M::Of<B>> + 'static,
    ) -> Box<dyn Fn(E) -> M::Of<B>> {
        reader_t_chain::<E, M, A, B>(fa, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::OptionF;

    #[test]
    fn reader_t_pure_test() {
        let r = reader_t_pure::<i32, OptionF, _>(42);
        assert_eq!(r(0), Some(42));
        assert_eq!(r(999), Some(42));
    }

    #[test]
    fn reader_t_ask_test() {
        let r = reader_t_ask::<i32, OptionF>();
        assert_eq!(r(42), Some(42));
    }

    #[test]
    fn reader_t_fmap_test() {
        let r = reader_t_ask::<i32, OptionF>();
        let mapped = reader_t_fmap::<i32, OptionF, _, _>(r, |x| x * 2);
        assert_eq!(mapped(5), Some(10));
    }

    #[test]
    fn reader_t_chain_test() {
        let r = reader_t_ask::<i32, OptionF>();
        let chained =
            reader_t_chain::<i32, OptionF, _, _>(r, |x| reader_t_pure::<i32, OptionF, _>(x + 10));
        assert_eq!(chained(5), Some(15));
    }

    #[test]
    fn reader_t_chain_shares_env() {
        let r = reader_t_chain::<i32, OptionF, _, _>(reader_t_ask::<i32, OptionF>(), |x| {
            let x_captured = x;
            reader_t_fmap::<i32, OptionF, _, _>(reader_t_ask::<i32, OptionF>(), move |e| {
                e + x_captured
            })
        });
        assert_eq!(r(10), Some(20));
    }

    #[test]
    fn reader_t_local_test() {
        let r = reader_t_ask::<i32, OptionF>();
        let localized = reader_t_local::<i32, OptionF, i32>(|e| e + 100, r);
        assert_eq!(localized(5), Some(105));
    }

    #[test]
    fn reader_t_reader_test() {
        let r = reader_t_reader::<String, OptionF, _>(|s: String| s.len());
        assert_eq!(r("hello".to_string()), Some(5));
    }

    #[test]
    fn reader_t_lift_test() {
        let lifted = ReaderTF::<i32, OptionF>::lift(Some(42));
        assert_eq!(lifted(999), Some(42));
    }

    #[test]
    fn reader_t_lift_none() {
        let lifted = ReaderTF::<i32, OptionF>::lift(None::<i32>);
        assert_eq!(lifted(999), None);
    }

    #[test]
    fn reader_t_run_test() {
        let r = reader_t_ask::<i32, OptionF>();
        assert_eq!(reader_t_run::<i32, OptionF, i32>(&*r, 42), Some(42));
    }

    #[test]
    fn reader_t_functor_st_trait() {
        let r = reader_t_pure::<i32, OptionF, _>(5);
        let mapped = ReaderTF::<i32, OptionF>::fmap_st(r, |x| x + 1);
        assert_eq!(mapped(0), Some(6));
    }

    #[test]
    fn reader_t_chain_st_trait() {
        let r = reader_t_pure::<i32, OptionF, _>(5);
        let chained =
            ReaderTF::<i32, OptionF>::chain_st(r, |x| reader_t_pure::<i32, OptionF, _>(x + 10));
        assert_eq!(chained(0), Some(15));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::hkt::OptionF;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn reader_t_monad_left_identity(a in -100i32..100, e in -100i32..100) {
            let f = |x: i32| -> Box<dyn Fn(i32) -> Option<i32>> {
                reader_t_pure::<i32, OptionF, _>(x + 1)
            };
            let left = reader_t_chain::<i32, OptionF, _, _>(
                reader_t_pure::<i32, OptionF, _>(a),
                f,
            );
            let right = f(a);
            prop_assert_eq!(left(e), right(e));
        }

        #[test]
        fn reader_t_monad_right_identity(a in -100i32..100, e in -100i32..100) {
            let m = reader_t_pure::<i32, OptionF, _>(a);
            let left = reader_t_chain::<i32, OptionF, _, _>(
                reader_t_pure::<i32, OptionF, _>(a),
                |x| reader_t_pure::<i32, OptionF, _>(x),
            );
            prop_assert_eq!(left(e), m(e));
        }

        #[test]
        fn reader_t_functor_identity(a in -100i32..100, e in -100i32..100) {
            let m = reader_t_pure::<i32, OptionF, _>(a);
            let mapped = reader_t_fmap::<i32, OptionF, _, _>(
                reader_t_pure::<i32, OptionF, _>(a),
                |x| x,
            );
            prop_assert_eq!(mapped(e), m(e));
        }

        #[test]
        fn reader_t_lift_pure(a in any::<i32>(), e in any::<i32>()) {
            let lift_pure = ReaderTF::<i32, OptionF>::lift(Some(a));
            let pure_a = reader_t_pure::<i32, OptionF, _>(a);
            prop_assert_eq!(lift_pure(e), pure_a(e));
        }
    }
}
