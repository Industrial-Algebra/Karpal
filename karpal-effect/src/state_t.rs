#![allow(clippy::type_complexity)]

use core::marker::PhantomData;

use karpal_core::hkt::HKT;

use crate::classes::{ApplicativeSt, ChainSt, FunctorSt};
use crate::trans::MonadTrans;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::rc::Rc;
#[cfg(feature = "std")]
use std::rc::Rc;

/// StateT monad transformer: adds mutable state to an inner monad.
///
/// `StateTF<S, M>::Of<A> = Box<dyn Fn(S) -> M::Of<(S, A)>>`
///
/// Unlike ReaderT, the state is threaded (modified) through computations.
/// When the inner monad is `IdentityF`, this is equivalent to the State monad
/// from `karpal-core`'s adjunction module.
pub struct StateTF<S, M>(PhantomData<(S, M)>);

impl<S: 'static, M: HKT> HKT for StateTF<S, M>
where
    for<'a> M: 'static,
{
    type Of<A> = Box<dyn Fn(S) -> M::Of<(S, A)>>;
}

impl<S: Clone + 'static, M: FunctorSt + 'static> MonadTrans<M> for StateTF<S, M> {
    fn lift<A: 'static>(ma: M::Of<A>) -> Box<dyn Fn(S) -> M::Of<(S, A)>>
    where
        M::Of<A>: Clone,
    {
        Box::new(move |s| M::fmap_st(ma.clone(), move |a| (s.clone(), a)))
    }
}

/// StateT `pure`: wrap a value without modifying state.
pub fn state_t_pure<S: Clone + 'static, M: ApplicativeSt + 'static, A: Clone + 'static>(
    a: A,
) -> Box<dyn Fn(S) -> M::Of<(S, A)>> {
    Box::new(move |s| M::pure_st((s, a.clone())))
}

/// StateT `fmap`: apply a function to the result, leaving state unchanged.
pub fn state_t_fmap<S: 'static, M: FunctorSt + 'static, A: 'static, B: 'static>(
    fa: Box<dyn Fn(S) -> M::Of<(S, A)>>,
    f: impl Fn(A) -> B + 'static,
) -> Box<dyn Fn(S) -> M::Of<(S, B)>> {
    let f_rc = Rc::new(f);
    Box::new(move |s| {
        let f_inner = f_rc.clone();
        M::fmap_st(fa(s), move |(s2, a)| (s2, f_inner(a)))
    })
}

/// StateT `chain`: sequence stateful computations, threading state.
///
/// The state from the first computation is passed to the second.
pub fn state_t_chain<S: Clone + 'static, M: ChainSt + 'static, A: 'static, B: 'static>(
    fa: Box<dyn Fn(S) -> M::Of<(S, A)>>,
    f: impl Fn(A) -> Box<dyn Fn(S) -> M::Of<(S, B)>> + 'static,
) -> Box<dyn Fn(S) -> M::Of<(S, B)>> {
    let f_rc = Rc::new(f);
    Box::new(move |s| {
        let f_inner = f_rc.clone();
        M::chain_st(fa(s), move |(s2, a)| {
            let state_b = f_inner(a);
            state_b(s2)
        })
    })
}

/// StateT `get`: read the current state.
pub fn state_t_get<S: Clone + 'static, M: ApplicativeSt + 'static>()
-> Box<dyn Fn(S) -> M::Of<(S, S)>> {
    Box::new(|s: S| {
        let s2 = s.clone();
        M::pure_st((s, s2))
    })
}

/// StateT `put`: replace the state.
pub fn state_t_put<S: Clone + 'static, M: ApplicativeSt + 'static>(
    new_state: S,
) -> Box<dyn Fn(S) -> M::Of<(S, ())>> {
    Box::new(move |_| M::pure_st((new_state.clone(), ())))
}

/// StateT `modify`: apply a function to the state.
pub fn state_t_modify<S: Clone + 'static, M: ApplicativeSt + 'static>(
    f: impl Fn(S) -> S + 'static,
) -> Box<dyn Fn(S) -> M::Of<(S, ())>> {
    Box::new(move |s| {
        let new_s = f(s);
        M::pure_st((new_s, ()))
    })
}

/// StateT `run`: run the computation with initial state.
pub fn state_t_run<S, M: HKT, A>(state: &dyn Fn(S) -> M::Of<(S, A)>, initial: S) -> M::Of<(S, A)> {
    state(initial)
}

// --- FunctorSt / ApplicativeSt / ChainSt for StateTF ---

impl<S: 'static, M: FunctorSt + 'static> FunctorSt for StateTF<S, M> {
    fn fmap_st<A: 'static, B: 'static>(
        fa: Box<dyn Fn(S) -> M::Of<(S, A)>>,
        f: impl Fn(A) -> B + 'static,
    ) -> Box<dyn Fn(S) -> M::Of<(S, B)>> {
        state_t_fmap::<S, M, A, B>(fa, f)
    }
}

// Note: ApplicativeSt is not implemented for StateTF because pure_st
// cannot produce a Box<dyn Fn(S) -> M::Of<(S, A)>> from a single A without Clone.
// Use the standalone state_t_pure function instead (which requires A: Clone).

impl<S: Clone + 'static, M: ChainSt + 'static> ChainSt for StateTF<S, M> {
    fn chain_st<A: 'static, B: 'static>(
        fa: Box<dyn Fn(S) -> M::Of<(S, A)>>,
        f: impl Fn(A) -> Box<dyn Fn(S) -> M::Of<(S, B)>> + 'static,
    ) -> Box<dyn Fn(S) -> M::Of<(S, B)>> {
        state_t_chain::<S, M, A, B>(fa, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::{IdentityF, OptionF};

    #[test]
    fn state_t_pure_identity() {
        let s = state_t_pure::<i32, IdentityF, _>(42);
        assert_eq!(s(0), (0, 42));
    }

    #[test]
    fn state_t_pure_option() {
        let s = state_t_pure::<i32, OptionF, _>(42);
        assert_eq!(s(0), Some((0, 42)));
    }

    #[test]
    fn state_t_get_test() {
        let g = state_t_get::<i32, OptionF>();
        assert_eq!(g(42), Some((42, 42)));
    }

    #[test]
    fn state_t_put_test() {
        let p = state_t_put::<i32, OptionF>(99);
        assert_eq!(p(0), Some((99, ())));
    }

    #[test]
    fn state_t_modify_test() {
        let m = state_t_modify::<i32, OptionF>(|s| s + 1);
        assert_eq!(m(5), Some((6, ())));
    }

    #[test]
    fn state_t_fmap_test() {
        let s = state_t_pure::<i32, OptionF, _>(10);
        let mapped = state_t_fmap::<i32, OptionF, _, _>(s, |x| x * 3);
        assert_eq!(mapped(0), Some((0, 30)));
    }

    #[test]
    fn state_t_chain_threads_state() {
        // get, then add state to value
        let program = state_t_chain::<i32, OptionF, _, _>(state_t_get::<i32, OptionF>(), |x| {
            state_t_chain::<i32, OptionF, _, _>(
                state_t_modify::<i32, OptionF>(move |s| s + x),
                |_| state_t_get::<i32, OptionF>(),
            )
        });
        assert_eq!(program(10), Some((20, 20))); // get 10, modify +10, get 20
    }

    #[test]
    fn state_t_chain_with_none() {
        // OptionF inner monad can short-circuit
        let program = state_t_chain::<i32, OptionF, _, _>(
            state_t_get::<i32, OptionF>(),
            |x| -> Box<dyn Fn(i32) -> Option<(i32, i32)>> {
                if x > 100 {
                    state_t_pure::<i32, OptionF, _>(x)
                } else {
                    Box::new(|_| None) // short-circuit
                }
            },
        );
        assert_eq!(program(10), None);
        assert_eq!(program(200), Some((200, 200)));
    }

    #[test]
    fn state_t_lift_option() {
        let lifted = StateTF::<i32, OptionF>::lift(Some(42));
        assert_eq!(lifted(99), Some((99, 42)));
    }

    #[test]
    fn state_t_lift_none() {
        let lifted = StateTF::<i32, OptionF>::lift(None::<i32>);
        assert_eq!(lifted(99), None);
    }

    #[test]
    fn state_t_run_test() {
        let s = state_t_pure::<i32, OptionF, _>(42);
        assert_eq!(state_t_run::<i32, OptionF, i32>(&*s, 0), Some((0, 42)));
    }

    // Trait impls

    #[test]
    fn state_t_functor_st_trait() {
        let s = state_t_pure::<i32, OptionF, _>(5);
        let mapped = StateTF::<i32, OptionF>::fmap_st(s, |x| x + 1);
        assert_eq!(mapped(0), Some((0, 6)));
    }

    #[test]
    fn state_t_chain_st_trait() {
        let s = state_t_pure::<i32, OptionF, _>(5);
        let chained =
            StateTF::<i32, OptionF>::chain_st(s, |x| state_t_pure::<i32, OptionF, _>(x + 10));
        assert_eq!(chained(0), Some((0, 15)));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::hkt::OptionF;
    use proptest::prelude::*;

    proptest! {
        // Monad left identity: chain(pure(a), f) == f(a)
        #[test]
        fn state_t_monad_left_identity(a in -100i32..100, s in -100i32..100) {
            let f = |x: i32| -> Box<dyn Fn(i32) -> Option<(i32, i32)>> {
                state_t_pure::<i32, OptionF, _>(x + 1)
            };
            let left = state_t_chain::<i32, OptionF, _, _>(
                state_t_pure::<i32, OptionF, _>(a),
                f,
            );
            let right = f(a);
            prop_assert_eq!(left(s), right(s));
        }

        // Monad right identity: chain(m, pure) == m
        #[test]
        fn state_t_monad_right_identity(a in -100i32..100, s in -100i32..100) {
            let m = state_t_pure::<i32, OptionF, _>(a);
            let left = state_t_chain::<i32, OptionF, _, _>(
                state_t_pure::<i32, OptionF, _>(a),
                |x| state_t_pure::<i32, OptionF, _>(x),
            );
            prop_assert_eq!(left(s), m(s));
        }

        // Functor identity
        #[test]
        fn state_t_functor_identity(a in -100i32..100, s in -100i32..100) {
            let m = state_t_pure::<i32, OptionF, _>(a);
            let mapped = state_t_fmap::<i32, OptionF, _, _>(
                state_t_pure::<i32, OptionF, _>(a),
                |x| x,
            );
            prop_assert_eq!(mapped(s), m(s));
        }

        // State: get then put restores
        #[test]
        fn state_t_get_put(s in -100i32..100) {
            let program = state_t_chain::<i32, OptionF, _, _>(
                state_t_get::<i32, OptionF>(),
                |x| state_t_put::<i32, OptionF>(x),
            );
            prop_assert_eq!(program(s), Some((s, ())));
        }

        // State: put then get returns what was put
        #[test]
        fn state_t_put_get(s in -100i32..100, new_s in -100i32..100) {
            let program = state_t_chain::<i32, OptionF, _, _>(
                state_t_put::<i32, OptionF>(new_s),
                |_| state_t_get::<i32, OptionF>(),
            );
            prop_assert_eq!(program(s), Some((new_s, new_s)));
        }
    }
}
