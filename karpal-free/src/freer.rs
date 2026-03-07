#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;

#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::rc::Rc;

use core::marker::PhantomData;

use karpal_core::applicative::Applicative;
use karpal_core::chain::Chain;
use karpal_core::functor::Functor;
use karpal_core::hkt::HKT;
use karpal_core::natural::NaturalTransformation;

// ---- Private step types for the existential encoding ----

/// Dyn-safe trait for an effect step in the Freer monad.
///
/// Each step stores `∃B. (F::Of<B>, B → Freer<F, A>)` — an effect value
/// and a continuation. The intermediate type `B` is hidden behind the
/// trait object.
trait FreerStep<F: HKT + 'static, A: 'static> {
    /// Lower this step to `F::Of<Freer<F, A>>` by applying F::fmap.
    /// Only called during `fold_map`, where `F: Functor` is available.
    fn lower_step(self: Box<Self>) -> F::Of<Freer<F, A>>
    where
        F: Functor;
}

/// A leaf step: stores an effect `F::Of<B>` and a continuation `B → Freer<F, A>`.
struct ImpureStep<F: HKT + 'static, A: 'static, B: 'static> {
    effect: F::Of<B>,
    cont: Box<dyn Fn(B) -> Freer<F, A>>,
}

impl<F: HKT + 'static, A: 'static, B: 'static> FreerStep<F, A> for ImpureStep<F, A, B> {
    fn lower_step(self: Box<Self>) -> F::Of<Freer<F, A>>
    where
        F: Functor,
    {
        F::fmap(self.effect, self.cont)
    }
}

/// A chained step: wraps an inner step with a deferred chain operation.
///
/// When lowered, first lowers the inner step to get `F::Of<Freer<F, Src>>`,
/// then fmaps `chain_rc` over it to produce `F::Of<Freer<F, A>>`.
struct ChainedStep<F: HKT + 'static, Src: 'static, A: 'static> {
    inner: Box<dyn FreerStep<F, Src>>,
    chain_fn: Rc<dyn Fn(Src) -> Freer<F, A>>,
}

impl<F: HKT + 'static, Src: 'static, A: 'static> FreerStep<F, A> for ChainedStep<F, Src, A> {
    fn lower_step(self: Box<Self>) -> F::Of<Freer<F, A>>
    where
        F: Functor,
    {
        let f_freer_src: F::Of<Freer<F, Src>> = self.inner.lower_step();
        let chain_fn = self.chain_fn;
        F::fmap(f_freer_src, move |freer_src: Freer<F, Src>| {
            freer_src.chain_rc(chain_fn.clone())
        })
    }
}

// ---- Public Freer type ----

#[allow(private_interfaces)]
/// Freer Monad — a free monad that does not require `F: Functor`.
///
/// `Freer<F, A>` stores a computation as a tree of effect steps, each
/// containing `∃B. (F B, B → Freer F A)`. The existential `B` is erased
/// via a dyn-safe trait, deferring the `Functor` requirement to `fold_map`.
///
/// ```text
/// Pure(a)       — a finished computation
/// Impure(step)  — an effect step with continuation
/// ```
///
/// # When to use Freer vs Free
///
/// - Use `Free<F, A>` when `F: Functor` — simpler, no overhead.
/// - Use `Freer<F, A>` when `F` is NOT a functor, or when you want to
///   build computations without the functor constraint.
pub enum Freer<F: HKT + 'static, A: 'static> {
    /// A pure value — the computation is finished.
    Pure(A),
    /// An effect step with erased intermediate type.
    Impure(Box<dyn FreerStep<F, A>>),
}

impl<F: HKT + 'static, A: 'static> Freer<F, A> {
    /// Wrap a pure value into the freer monad.
    pub fn pure(a: A) -> Self {
        Freer::Pure(a)
    }

    /// Lift a single effect `F<A>` into the freer monad.
    ///
    /// No `F: Functor` required.
    pub fn lift_f(fa: F::Of<A>) -> Self
    where
        F::Of<A>: 'static,
    {
        Freer::Impure(Box::new(ImpureStep {
            effect: fa,
            cont: Box::new(Freer::Pure),
        }))
    }

    /// Map a function over the result of this computation.
    ///
    /// No `F: Functor` required. Implemented via `chain`.
    pub fn fmap<B: 'static>(self, f: impl Fn(A) -> B + 'static) -> Freer<F, B> {
        self.chain(move |a| Freer::Pure(f(a)))
    }

    /// Monadic bind — sequence this computation with a function that
    /// produces the next computation.
    ///
    /// No `F: Functor` required. The closure is shared via `Rc` across
    /// deferred chain layers.
    pub fn chain<B: 'static>(self, f: impl Fn(A) -> Freer<F, B> + 'static) -> Freer<F, B> {
        let f = Rc::new(f);
        self.chain_rc(f)
    }

    fn chain_rc<B: 'static>(self, f: Rc<dyn Fn(A) -> Freer<F, B>>) -> Freer<F, B> {
        match self {
            Freer::Pure(a) => f(a),
            Freer::Impure(step) => Freer::Impure(Box::new(ChainedStep {
                inner: step,
                chain_fn: f,
            })),
        }
    }

    /// Interpret this freer monad into a target monad `M` using a natural
    /// transformation `NT: F ~> M`.
    ///
    /// This requires `F: Functor` (to lower each step) and
    /// `M: Applicative + Chain` (for the target monad operations).
    pub fn fold_map<M, NT>(self) -> M::Of<A>
    where
        F: Functor,
        M: Applicative + Chain,
        NT: NaturalTransformation<F, M>,
    {
        match self {
            Freer::Pure(a) => M::pure(a),
            Freer::Impure(step) => {
                // Lower the step to get F<Freer<F, A>>
                let f_freer: F::Of<Freer<F, A>> = step.lower_step();
                // Apply NT to get M<Freer<F, A>>
                let m_freer: M::Of<Freer<F, A>> = NT::transform(f_freer);
                // Chain with recursive fold_map
                M::chain(m_freer, |freer| freer.fold_map::<M, NT>())
            }
        }
    }
}

/// HKT marker for `Freer<F, _>`.
///
/// Note: Cannot implement `HKT` or `Functor` due to Rust's GAT limitations
/// (`type Of<T>` cannot add `T: 'static` in impl when trait doesn't have it).
/// Use `Freer::fmap` directly.
pub struct FreerF<F: HKT + 'static>(PhantomData<F>);

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::OptionF;

    #[test]
    fn pure_value() {
        let freer = Freer::<OptionF, i32>::pure(42);
        match freer {
            Freer::Pure(v) => assert_eq!(v, 42),
            Freer::Impure(_) => panic!("expected Pure"),
        }
    }

    #[test]
    fn lift_f_some() {
        let freer = Freer::<OptionF, i32>::lift_f(Some(10));
        match freer {
            Freer::Impure(_) => {} // correct
            Freer::Pure(_) => panic!("expected Impure"),
        }
    }

    #[test]
    fn chain_pure() {
        let freer = Freer::<OptionF, i32>::pure(1).chain(|x| Freer::pure(x + 1));
        match freer {
            Freer::Pure(v) => assert_eq!(v, 2),
            _ => panic!("expected Pure"),
        }
    }

    #[test]
    fn fmap_pure() {
        let freer = Freer::<OptionF, i32>::pure(5).fmap(|x| x * 3);
        match freer {
            Freer::Pure(v) => assert_eq!(v, 15),
            _ => panic!("expected Pure"),
        }
    }

    #[test]
    fn chain_associativity() {
        // (m >>= f) >>= g
        let left = Freer::<OptionF, i32>::pure(5)
            .chain(|x| Freer::pure(x + 1))
            .chain(|x| Freer::pure(x * 2));

        // m >>= (\x -> f(x) >>= g)
        let right = Freer::<OptionF, i32>::pure(5)
            .chain(|x| Freer::<OptionF, i32>::pure(x + 1).chain(|y| Freer::pure(y * 2)));

        match (left, right) {
            (Freer::Pure(l), Freer::Pure(r)) => assert_eq!(l, r),
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
        let freer = Freer::<OptionF, i32>::pure(42);
        let result = freer.fold_map::<OptionF, OptionId>();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn fold_map_lift() {
        let freer = Freer::<OptionF, i32>::lift_f(Some(10));
        let result = freer.fold_map::<OptionF, OptionId>();
        assert_eq!(result, Some(10));
    }

    #[test]
    fn fold_map_chain() {
        let freer = Freer::<OptionF, i32>::lift_f(Some(3)).chain(|x| Freer::lift_f(Some(x * 10)));
        let result = freer.fold_map::<OptionF, OptionId>();
        assert_eq!(result, Some(30));
    }

    #[test]
    fn fold_map_lift_none() {
        let freer = Freer::<OptionF, i32>::lift_f(None);
        let result = freer.fold_map::<OptionF, OptionId>();
        assert_eq!(result, None);
    }

    #[test]
    fn fmap_lift_then_fold() {
        let freer = Freer::<OptionF, i32>::lift_f(Some(5)).fmap(|x| x + 10);
        let result = freer.fold_map::<OptionF, OptionId>();
        assert_eq!(result, Some(15));
    }

    #[test]
    fn chain_lift_multiple() {
        let freer = Freer::<OptionF, i32>::lift_f(Some(1))
            .chain(|x| Freer::lift_f(Some(x + 1)))
            .chain(|x| Freer::lift_f(Some(x * 10)));
        let result = freer.fold_map::<OptionF, OptionId>();
        assert_eq!(result, Some(20)); // (1+1)*10
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::hkt::OptionF;
    use proptest::prelude::*;

    fn extract_pure<F: HKT + 'static, A: 'static>(freer: Freer<F, A>) -> Option<A> {
        match freer {
            Freer::Pure(a) => Some(a),
            Freer::Impure(_) => None,
        }
    }

    proptest! {
        // Monad left identity: pure(a) >>= f == f(a)
        #[test]
        fn monad_left_identity(x in any::<i32>()) {
            let left = extract_pure(
                Freer::<OptionF, i32>::pure(x)
                    .chain(|a| Freer::pure(a.wrapping_mul(2))),
            );
            let right = Some(x.wrapping_mul(2));
            prop_assert_eq!(left, right);
        }

        // Monad right identity: m >>= pure == m
        #[test]
        fn monad_right_identity(x in any::<i32>()) {
            let result = extract_pure(Freer::<OptionF, i32>::pure(x).chain(Freer::pure));
            prop_assert_eq!(result, Some(x));
        }
    }
}
