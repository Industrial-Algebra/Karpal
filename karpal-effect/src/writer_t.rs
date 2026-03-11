use core::marker::PhantomData;

use karpal_core::hkt::HKT;
use karpal_core::monoid::Monoid;
use karpal_core::semigroup::Semigroup;

use crate::classes::{ApplicativeSt, ChainSt, FunctorSt};
use crate::trans::MonadTrans;

/// WriterT monad transformer: adds log accumulation to an inner monad.
///
/// `WriterTF<W, M>::Of<A> = M::Of<(A, W)>`
///
/// The log type `W` must be a `Monoid` (for `pure`/`lift`) and a `Semigroup`
/// (for `chain`, which combines logs). This is the strict (non-lazy) variant.
pub struct WriterTF<W, M>(PhantomData<(W, M)>);

impl<W: 'static, M: HKT> HKT for WriterTF<W, M> {
    type Of<A> = M::Of<(A, W)>;
}

impl<W: Monoid + 'static, M: FunctorSt> MonadTrans<M> for WriterTF<W, M> {
    fn lift<A: 'static>(ma: M::Of<A>) -> M::Of<(A, W)> {
        M::fmap_st(ma, |a| (a, W::empty()))
    }
}

/// WriterT `pure`: wrap a value with an empty log.
pub fn writer_t_pure<W: Monoid + 'static, M: ApplicativeSt, A: 'static>(a: A) -> M::Of<(A, W)> {
    M::pure_st((a, W::empty()))
}

/// WriterT `fmap`: apply a function to the value, preserving the log.
pub fn writer_t_fmap<W: 'static, M: FunctorSt, A: 'static, B: 'static>(
    fa: M::Of<(A, W)>,
    f: impl Fn(A) -> B + 'static,
) -> M::Of<(B, W)> {
    M::fmap_st(fa, move |(a, w)| (f(a), w))
}

/// WriterT `chain`: sequence log-accumulating computations.
///
/// The logs from both computations are combined using `Semigroup::combine`.
pub fn writer_t_chain<W: Semigroup + Clone + 'static, M: ChainSt, A: 'static, B: 'static>(
    fa: M::Of<(A, W)>,
    f: impl Fn(A) -> M::Of<(B, W)> + 'static,
) -> M::Of<(B, W)> {
    M::chain_st(fa, move |(a, w1)| {
        M::fmap_st(f(a), move |(b, w2)| {
            let w1_owned = w1.clone();
            (b, w1_owned.combine(w2))
        })
    })
}

/// WriterT `tell`: append a log value, producing `()`.
pub fn writer_t_tell<W: 'static, M: ApplicativeSt>(w: W) -> M::Of<((), W)> {
    M::pure_st(((), w))
}

/// WriterT `listen`: expose the log alongside the value.
pub fn writer_t_listen<W: Clone + 'static, M: FunctorSt, A: 'static>(
    fa: M::Of<(A, W)>,
) -> M::Of<((A, W), W)> {
    M::fmap_st(fa, |(a, w): (A, W)| {
        let w2 = w.clone();
        ((a, w), w2)
    })
}

/// WriterT `pass`: apply a function to the log.
///
/// The value is a pair `(a, f)` where `f` transforms the log.
#[allow(clippy::type_complexity)]
pub fn writer_t_pass<W: 'static, M: FunctorSt, A: 'static>(
    fa: M::Of<((A, Box<dyn Fn(W) -> W>), W)>,
) -> M::Of<(A, W)> {
    M::fmap_st(fa, |((a, f), w)| (a, f(w)))
}

/// WriterT `run`: unwrap the transformer (identity — included for API symmetry).
pub fn writer_t_run<W, M: HKT, A>(fa: M::Of<(A, W)>) -> M::Of<(A, W)> {
    fa
}

// --- FunctorSt / ApplicativeSt / ChainSt for WriterTF ---

impl<W: 'static, M: FunctorSt> FunctorSt for WriterTF<W, M> {
    fn fmap_st<A: 'static, B: 'static>(
        fa: M::Of<(A, W)>,
        f: impl Fn(A) -> B + 'static,
    ) -> M::Of<(B, W)> {
        writer_t_fmap::<W, M, A, B>(fa, f)
    }
}

impl<W: Monoid + 'static, M: ApplicativeSt> ApplicativeSt for WriterTF<W, M> {
    fn pure_st<A: 'static>(a: A) -> M::Of<(A, W)> {
        writer_t_pure::<W, M, A>(a)
    }
}

impl<W: Semigroup + Clone + 'static, M: ChainSt + FunctorSt> ChainSt for WriterTF<W, M> {
    fn chain_st<A: 'static, B: 'static>(
        fa: M::Of<(A, W)>,
        f: impl Fn(A) -> M::Of<(B, W)> + 'static,
    ) -> M::Of<(B, W)> {
        writer_t_chain::<W, M, A, B>(fa, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::{IdentityF, OptionF};

    #[test]
    fn writer_t_pure_identity() {
        let result = writer_t_pure::<String, IdentityF, i32>(42);
        assert_eq!(result, (42, String::new()));
    }

    #[test]
    fn writer_t_pure_option() {
        let result = writer_t_pure::<String, OptionF, i32>(42);
        assert_eq!(result, Some((42, String::new())));
    }

    #[test]
    fn writer_t_fmap_test() {
        let val = writer_t_pure::<String, OptionF, i32>(10);
        let result = writer_t_fmap::<String, OptionF, _, _>(val, |x| x * 3);
        assert_eq!(result, Some((30, String::new())));
    }

    #[test]
    fn writer_t_tell_test() {
        let told = writer_t_tell::<String, OptionF>("hello".to_string());
        assert_eq!(told, Some(((), "hello".to_string())));
    }

    #[test]
    fn writer_t_chain_accumulates_log() {
        let m1 = writer_t_tell::<String, OptionF>("a".to_string());
        let result = writer_t_chain::<String, OptionF, _, _>(m1, |()| {
            writer_t_tell::<String, OptionF>("b".to_string())
        });
        assert_eq!(result, Some(((), "ab".to_string())));
    }

    #[test]
    fn writer_t_chain_with_value() {
        let m1: Option<(i32, String)> = Some((10, "start".to_string()));
        let result =
            writer_t_chain::<String, OptionF, _, _>(m1, |x| Some((x + 5, " end".to_string())));
        assert_eq!(result, Some((15, "start end".to_string())));
    }

    #[test]
    fn writer_t_chain_none() {
        let m1: Option<(i32, String)> = None;
        let result =
            writer_t_chain::<String, OptionF, _, _>(m1, |x| Some((x + 5, "end".to_string())));
        assert_eq!(result, None);
    }

    #[test]
    fn writer_t_listen_test() {
        let val: Option<(i32, String)> = Some((42, "log".to_string()));
        let result = writer_t_listen::<String, OptionF, i32>(val);
        assert_eq!(result, Some(((42, "log".to_string()), "log".to_string())));
    }

    #[test]
    fn writer_t_pass_test() {
        let f: Box<dyn Fn(String) -> String> = Box::new(|w| w.to_uppercase());
        let val: Option<((i32, Box<dyn Fn(String) -> String>), String)> =
            Some(((42, f), "hello".to_string()));
        let result = writer_t_pass::<String, OptionF, i32>(val);
        assert_eq!(result, Some((42, "HELLO".to_string())));
    }

    #[test]
    fn writer_t_lift_option() {
        let lifted = WriterTF::<String, OptionF>::lift(Some(42));
        assert_eq!(lifted, Some((42, String::new())));
    }

    #[test]
    fn writer_t_lift_none() {
        let lifted = WriterTF::<String, OptionF>::lift(None::<i32>);
        assert_eq!(lifted, None);
    }

    // Trait impls

    #[test]
    fn writer_t_functor_st_trait() {
        let val = writer_t_pure::<String, OptionF, i32>(5);
        let result = WriterTF::<String, OptionF>::fmap_st(val, |x| x + 1);
        assert_eq!(result, Some((6, String::new())));
    }

    #[test]
    fn writer_t_chain_st_trait() {
        let val = WriterTF::<String, OptionF>::pure_st(5);
        let result =
            WriterTF::<String, OptionF>::chain_st(val, |x| Some((x + 10, "log".to_string())));
        assert_eq!(result, Some((15, "log".to_string())));
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
        fn writer_t_functor_identity(a in any::<i16>(), w in "[a-z]{0,5}") {
            let val: Option<(i16, String)> = Some((a, w.clone()));
            let left = writer_t_fmap::<String, OptionF, _, _>(val.clone(), |x| x);
            prop_assert_eq!(left, val);
        }

        // Monad left identity: chain(pure(a), f) == f(a)
        #[test]
        fn writer_t_monad_left_identity(a in -100i32..100) {
            let f = |x: i32| -> Option<(i32, String)> {
                Some((x + 1, "f".to_string()))
            };
            let left = writer_t_chain::<String, OptionF, _, _>(
                writer_t_pure::<String, OptionF, _>(a),
                f,
            );
            let right = f(a);
            prop_assert_eq!(left, right);
        }

        // Monad right identity: chain(m, pure) == m
        #[test]
        fn writer_t_monad_right_identity(a in any::<i16>(), w in "[a-z]{0,5}") {
            let val: Option<(i16, String)> = Some((a, w));
            let left = writer_t_chain::<String, OptionF, _, _>(
                val.clone(),
                |x| writer_t_pure::<String, OptionF, _>(x),
            );
            prop_assert_eq!(left, val);
        }

        // MonadTrans: lift(pure(a)) == pure(a)
        #[test]
        fn writer_t_lift_pure(a in any::<i32>()) {
            let lift_pure = WriterTF::<String, OptionF>::lift(Some(a));
            let pure_a = writer_t_pure::<String, OptionF, _>(a);
            prop_assert_eq!(lift_pure, pure_a);
        }
    }
}
