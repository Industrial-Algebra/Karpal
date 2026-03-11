use core::marker::PhantomData;

use karpal_core::hkt::HKT;

use crate::classes::{ApplicativeSt, ChainSt, FunctorSt};
use crate::trans::MonadTrans;

/// ExceptT monad transformer: adds error handling to an inner monad.
///
/// `ExceptTF<E, M>::Of<A> = M::Of<Result<A, E>>`
///
/// When the inner monad is `IdentityF`, this is equivalent to `ResultF<E>`.
pub struct ExceptTF<E, M>(PhantomData<(E, M)>);

impl<E: 'static, M: HKT> HKT for ExceptTF<E, M> {
    type Of<A> = M::Of<Result<A, E>>;
}

impl<E: 'static, M: FunctorSt> MonadTrans<M> for ExceptTF<E, M> {
    fn lift<A: 'static>(ma: M::Of<A>) -> M::Of<Result<A, E>> {
        M::fmap_st(ma, Ok)
    }
}

/// ExceptT `pure`: wrap a value in `Ok` inside the inner monad.
pub fn except_t_pure<E: 'static, M: ApplicativeSt, A: 'static>(a: A) -> M::Of<Result<A, E>> {
    M::pure_st(Ok(a))
}

/// ExceptT `fmap`: apply a function to the `Ok` value.
pub fn except_t_fmap<E: 'static, M: FunctorSt, A: 'static, B: 'static>(
    fa: M::Of<Result<A, E>>,
    f: impl Fn(A) -> B + 'static,
) -> M::Of<Result<B, E>> {
    M::fmap_st(fa, move |r| r.map(&f))
}

/// ExceptT `chain`: sequence error-handling computations.
///
/// Short-circuits on `Err` — the function `f` is only called for `Ok` values.
pub fn except_t_chain<E: 'static, M: ChainSt + ApplicativeSt, A: 'static, B: 'static>(
    fa: M::Of<Result<A, E>>,
    f: impl Fn(A) -> M::Of<Result<B, E>> + 'static,
) -> M::Of<Result<B, E>> {
    M::chain_st(fa, move |r| match r {
        Ok(a) => f(a),
        Err(e) => M::pure_st(Err(e)),
    })
}

/// ExceptT `throw_error`: produce an error value inside the transformer.
pub fn except_t_throw<E: 'static, M: ApplicativeSt, A: 'static>(e: E) -> M::Of<Result<A, E>> {
    M::pure_st(Err(e))
}

/// ExceptT `catch_error`: handle an error by running a recovery function.
pub fn except_t_catch<E: 'static, M: ChainSt + ApplicativeSt, A: 'static>(
    fa: M::Of<Result<A, E>>,
    handler: impl Fn(E) -> M::Of<Result<A, E>> + 'static,
) -> M::Of<Result<A, E>> {
    M::chain_st(fa, move |r| match r {
        Ok(a) => M::pure_st(Ok(a)),
        Err(e) => handler(e),
    })
}

/// ExceptT `run`: unwrap the transformer (identity — included for API symmetry).
pub fn except_t_run<E, M: HKT, A>(fa: M::Of<Result<A, E>>) -> M::Of<Result<A, E>> {
    fa
}

// --- FunctorSt / ApplicativeSt / ChainSt for ExceptTF ---

impl<E: 'static, M: FunctorSt> FunctorSt for ExceptTF<E, M> {
    fn fmap_st<A: 'static, B: 'static>(
        fa: M::Of<Result<A, E>>,
        f: impl Fn(A) -> B + 'static,
    ) -> M::Of<Result<B, E>> {
        except_t_fmap::<E, M, A, B>(fa, f)
    }
}

impl<E: 'static, M: ApplicativeSt> ApplicativeSt for ExceptTF<E, M> {
    fn pure_st<A: 'static>(a: A) -> M::Of<Result<A, E>> {
        except_t_pure::<E, M, A>(a)
    }
}

impl<E: 'static, M: ChainSt + ApplicativeSt> ChainSt for ExceptTF<E, M> {
    fn chain_st<A: 'static, B: 'static>(
        fa: M::Of<Result<A, E>>,
        f: impl Fn(A) -> M::Of<Result<B, E>> + 'static,
    ) -> M::Of<Result<B, E>> {
        except_t_chain::<E, M, A, B>(fa, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::{IdentityF, OptionF};

    #[test]
    fn except_t_pure_identity() {
        let result = except_t_pure::<&str, IdentityF, i32>(42);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn except_t_pure_option() {
        let result = except_t_pure::<&str, OptionF, i32>(42);
        assert_eq!(result, Some(Ok(42)));
    }

    #[test]
    fn except_t_fmap_ok() {
        let val = except_t_pure::<&str, OptionF, i32>(10);
        let result = except_t_fmap::<&str, OptionF, _, _>(val, |x| x * 3);
        assert_eq!(result, Some(Ok(30)));
    }

    #[test]
    fn except_t_fmap_err() {
        let val: Option<Result<i32, &str>> = Some(Err("bad"));
        let result = except_t_fmap::<&str, OptionF, _, _>(val, |x: i32| x * 3);
        assert_eq!(result, Some(Err("bad")));
    }

    #[test]
    fn except_t_fmap_none() {
        let val: Option<Result<i32, &str>> = None;
        let result = except_t_fmap::<&str, OptionF, _, _>(val, |x: i32| x * 3);
        assert_eq!(result, None);
    }

    #[test]
    fn except_t_chain_ok() {
        let val = except_t_pure::<&str, OptionF, i32>(5);
        let result = except_t_chain::<&str, OptionF, _, _>(val, |x| Some(Ok(x + 10)));
        assert_eq!(result, Some(Ok(15)));
    }

    #[test]
    fn except_t_chain_err_short_circuits() {
        let val: Option<Result<i32, &str>> = Some(Err("fail"));
        let result = except_t_chain::<&str, OptionF, _, _>(val, |x| Some(Ok(x + 10)));
        assert_eq!(result, Some(Err("fail")));
    }

    #[test]
    fn except_t_chain_none() {
        let val: Option<Result<i32, &str>> = None;
        let result = except_t_chain::<&str, OptionF, _, _>(val, |x| Some(Ok(x + 10)));
        assert_eq!(result, None);
    }

    #[test]
    fn except_t_throw_test() {
        let result = except_t_throw::<&str, OptionF, i32>("oops");
        assert_eq!(result, Some(Err("oops")));
    }

    #[test]
    fn except_t_catch_recovers() {
        let val: Option<Result<i32, &str>> = Some(Err("bad"));
        let result = except_t_catch::<&str, OptionF, i32>(val, |_| Some(Ok(42)));
        assert_eq!(result, Some(Ok(42)));
    }

    #[test]
    fn except_t_catch_ok_passes_through() {
        let val = except_t_pure::<&str, OptionF, i32>(10);
        let result = except_t_catch::<&str, OptionF, i32>(val, |_| Some(Ok(42)));
        assert_eq!(result, Some(Ok(10)));
    }

    #[test]
    fn except_t_lift_option() {
        let lifted = ExceptTF::<&str, OptionF>::lift(Some(42));
        assert_eq!(lifted, Some(Ok(42)));
    }

    #[test]
    fn except_t_lift_none() {
        let lifted = ExceptTF::<&str, OptionF>::lift(None::<i32>);
        assert_eq!(lifted, None);
    }

    // Trait impls

    #[test]
    fn except_t_functor_st_trait() {
        let val = except_t_pure::<&str, OptionF, i32>(5);
        let result = ExceptTF::<&str, OptionF>::fmap_st(val, |x| x + 1);
        assert_eq!(result, Some(Ok(6)));
    }

    #[test]
    fn except_t_applicative_st_trait() {
        let result = ExceptTF::<&str, OptionF>::pure_st(99);
        assert_eq!(result, Some(Ok(99)));
    }

    #[test]
    fn except_t_chain_st_trait() {
        let val = ExceptTF::<&str, OptionF>::pure_st(5);
        let result = ExceptTF::<&str, OptionF>::chain_st(val, |x| Some(Ok(x + 10)));
        assert_eq!(result, Some(Ok(15)));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::hkt::OptionF;
    use proptest::prelude::*;

    proptest! {
        // Functor identity
        #[test]
        fn except_t_functor_identity(x in any::<Option<Result<i32, i32>>>()) {
            let left = except_t_fmap::<i32, OptionF, _, _>(x.clone(), |a| a);
            prop_assert_eq!(left, x);
        }

        // Functor composition
        #[test]
        fn except_t_functor_composition(x in any::<Option<Result<i16, i16>>>()) {
            let f = |a: i16| a.wrapping_add(1);
            let g = |a: i16| a.wrapping_mul(2);
            let left = except_t_fmap::<i16, OptionF, _, _>(x.clone(), move |a| g(f(a)));
            let right = except_t_fmap::<i16, OptionF, _, _>(
                except_t_fmap::<i16, OptionF, _, _>(x, f),
                g,
            );
            prop_assert_eq!(left, right);
        }

        // Monad left identity: chain(pure(a), f) == f(a)
        #[test]
        fn except_t_monad_left_identity(a in -100i32..100) {
            let f = |x: i32| -> Option<Result<i32, &str>> { Some(Ok(x + 1)) };
            let left = except_t_chain::<&str, OptionF, _, _>(
                except_t_pure::<&str, OptionF, _>(a),
                f,
            );
            let right = f(a);
            prop_assert_eq!(left, right);
        }

        // Monad right identity: chain(m, pure) == m
        #[test]
        fn except_t_monad_right_identity(x in any::<Option<Result<i32, i32>>>()) {
            let left = except_t_chain::<i32, OptionF, _, _>(
                x.clone(),
                |a| except_t_pure::<i32, OptionF, _>(a),
            );
            prop_assert_eq!(left, x);
        }

        // MonadTrans: lift(pure(a)) == pure(a)
        #[test]
        fn except_t_lift_pure(a in any::<i32>()) {
            let lift_pure = ExceptTF::<&str, OptionF>::lift(Some(a));
            let pure_a = except_t_pure::<&str, OptionF, _>(a);
            prop_assert_eq!(lift_pure, pure_a);
        }
    }
}
