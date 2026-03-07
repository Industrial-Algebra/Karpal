#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;

use core::marker::PhantomData;

use karpal_core::applicative::Applicative;
use karpal_core::hkt::HKT;

// ---- Private dyn-safe trait for existential encoding ----

/// Dyn-safe trait for a node in the Free Applicative tree.
///
/// Each node erases some intermediate type `B` via trait-object dispatch.
/// Only methods that use types already in the trait parameters (F, A) are
/// dyn-compatible.
trait FreeApNode<F: HKT + 'static, A: 'static> {
    /// Retract into F's own Applicative. Dyn-safe because F and A are
    /// trait-level type parameters.
    fn retract_node(self: Box<Self>) -> F::Of<A>
    where
        F: Applicative;

    /// Count the number of `lift_f` effects in this subtree.
    fn count_effects(&self) -> usize;
}

/// Lift node: stores `F<B>` and continuation `FreeAp<F, Box<dyn Fn(B) -> A>>`.
///
/// Represents the GADT constructor: `Ap :: f b -> FreeAp f (b -> a) -> FreeAp f a`
struct LiftNode<F: HKT + 'static, A: 'static, B: Clone + 'static> {
    effect: F::Of<B>,
    rest: FreeAp<F, Box<dyn Fn(B) -> A>>,
}

impl<F: HKT + 'static, A: 'static, B: Clone + 'static> FreeApNode<F, A> for LiftNode<F, A, B> {
    fn retract_node(self: Box<Self>) -> F::Of<A>
    where
        F: Applicative,
    {
        let f_fn: F::Of<Box<dyn Fn(B) -> A>> = self.rest.retract();
        let f_b: F::Of<B> = self.effect;
        F::ap(f_fn, f_b)
    }

    fn count_effects(&self) -> usize {
        1 + self.rest.count_effects()
    }
}

/// Fmap node: deferred map operation.
struct FmapNode<F: HKT + 'static, Src: 'static, A: 'static> {
    inner: FreeAp<F, Src>,
    transform: Box<dyn Fn(Src) -> A>,
}

impl<F: HKT + 'static, Src: 'static, A: 'static> FreeApNode<F, A> for FmapNode<F, Src, A> {
    fn retract_node(self: Box<Self>) -> F::Of<A>
    where
        F: Applicative,
    {
        let f_src: F::Of<Src> = self.inner.retract();
        F::fmap(f_src, self.transform)
    }

    fn count_effects(&self) -> usize {
        self.inner.count_effects()
    }
}

/// Ap node: deferred applicative application.
struct ApNode<F: HKT + 'static, Src: Clone + 'static, A: 'static> {
    ff: FreeAp<F, Box<dyn Fn(Src) -> A>>,
    fa: FreeAp<F, Src>,
}

impl<F: HKT + 'static, Src: Clone + 'static, A: 'static> FreeApNode<F, A> for ApNode<F, Src, A> {
    fn retract_node(self: Box<Self>) -> F::Of<A>
    where
        F: Applicative,
    {
        let f_fn: F::Of<Box<dyn Fn(Src) -> A>> = self.ff.retract();
        let f_src: F::Of<Src> = self.fa.retract();
        F::ap(f_fn, f_src)
    }

    fn count_effects(&self) -> usize {
        self.ff.count_effects() + self.fa.count_effects()
    }
}

// ---- Public FreeAp type ----

#[allow(private_interfaces)]
/// Free Applicative Functor — build applicative computations as data.
///
/// `FreeAp<F, A>` stores a computation tree where effects from `F`
/// can be statically analyzed before interpretation. Unlike `Free<F, A>`
/// (the free monad), effects in `FreeAp` do not depend on the results
/// of previous effects.
///
/// ```text
/// Pure(a)     — a finished computation
/// Ap(node)    — an effect step (existentially quantified)
/// ```
///
/// # Interpretation
///
/// The primary eliminator is `retract`, which collapses the tree into
/// `F`'s own applicative. To interpret into a *different* applicative `M`
/// via a natural transformation `NT: F ~> M`, apply `NT` at each
/// `lift_f` call site:
///
/// ```text
/// // Instead of fold_map:
/// let free_m: FreeAp<M, A> = build_tree_with(|effect| lift_f(NT::transform(effect)));
/// let result: M::Of<A> = free_m.retract();
/// ```
///
/// This decomposition (`fold_map nt ≡ retract . hoist nt`) is necessary
/// because Rust's type system cannot dispatch a generic natural
/// transformation through trait objects (the intermediate type `B` is
/// erased, preventing compile-time monomorphization of `NT::transform<B>`).
///
/// # When to use FreeAp vs Free
///
/// - Use `FreeAp<F, A>` when effects are independent and you want
///   static analysis of the effect structure.
/// - Use `Free<F, A>` when later effects depend on earlier results
///   (monadic sequencing).
pub enum FreeAp<F: HKT + 'static, A: 'static> {
    /// A pure value.
    Pure(A),
    /// An effect step with erased intermediate type.
    Ap(Box<dyn FreeApNode<F, A>>),
}

impl<F: HKT + 'static, A: 'static> FreeAp<F, A> {
    /// Wrap a pure value into the free applicative.
    pub fn pure(a: A) -> Self {
        FreeAp::Pure(a)
    }

    /// Lift a single effect `F<A>` into the free applicative.
    ///
    /// `A: Clone` is required because `Apply::ap` needs it.
    pub fn lift_f(fa: F::Of<A>) -> Self
    where
        A: Clone,
        F::Of<A>: 'static,
    {
        FreeAp::Ap(Box::new(LiftNode {
            effect: fa,
            rest: FreeAp::Pure(Box::new(|b| b) as Box<dyn Fn(A) -> A>),
        }))
    }

    /// Map a function over the result. No bounds on `F` required.
    pub fn fmap<B: 'static>(self, f: impl Fn(A) -> B + 'static) -> FreeAp<F, B> {
        match self {
            FreeAp::Pure(a) => FreeAp::Pure(f(a)),
            FreeAp::Ap(node) => FreeAp::Ap(Box::new(FmapNode {
                inner: FreeAp::Ap(node),
                transform: Box::new(f),
            })),
        }
    }

    /// Applicative `ap`: apply a wrapped function to this value.
    ///
    /// `ff` contains functions `A → B`, `self` contains `A` values.
    pub fn ap<B: 'static>(ff: FreeAp<F, Box<dyn Fn(A) -> B>>, fa: FreeAp<F, A>) -> FreeAp<F, B>
    where
        A: Clone,
    {
        match ff {
            FreeAp::Pure(f) => fa.fmap(f),
            FreeAp::Ap(node) => FreeAp::Ap(Box::new(ApNode {
                ff: FreeAp::Ap(node),
                fa,
            })),
        }
    }

    /// Interpret by collapsing back into `F` itself.
    ///
    /// Requires `F: Applicative`.
    pub fn retract(self) -> F::Of<A>
    where
        F: Applicative,
    {
        match self {
            FreeAp::Pure(a) => F::pure(a),
            FreeAp::Ap(node) => node.retract_node(),
        }
    }

    /// Count the number of `lift_f` effects in this computation tree.
    ///
    /// This demonstrates the key advantage of free applicatives over
    /// free monads: the tree structure can be statically analyzed
    /// without interpretation.
    pub fn count_effects(&self) -> usize {
        match self {
            FreeAp::Pure(_) => 0,
            FreeAp::Ap(node) => node.count_effects(),
        }
    }
}

/// Marker type for `FreeAp<F, _>`.
///
/// Note: Cannot implement `HKT` or `Functor` due to Rust's GAT limitations
/// (`type Of<T>` cannot add `T: 'static` in impl when trait doesn't have it).
/// Use `FreeAp::fmap` directly.
pub struct FreeApF<F: HKT + 'static>(PhantomData<F>);

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::OptionF;

    #[test]
    fn pure_retract() {
        let fa = FreeAp::<OptionF, i32>::pure(42);
        let result = fa.retract();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn lift_f_retract() {
        let fa = FreeAp::<OptionF, i32>::lift_f(Some(10));
        let result = fa.retract();
        assert_eq!(result, Some(10));
    }

    #[test]
    fn lift_f_none() {
        let fa = FreeAp::<OptionF, i32>::lift_f(None);
        let result = fa.retract();
        assert_eq!(result, None);
    }

    #[test]
    fn fmap_pure_retract() {
        let fa = FreeAp::<OptionF, i32>::pure(5).fmap(|x| x * 3);
        let result = fa.retract();
        assert_eq!(result, Some(15));
    }

    #[test]
    fn fmap_lift_retract() {
        let fa = FreeAp::<OptionF, i32>::lift_f(Some(4)).fmap(|x| x + 10);
        let result = fa.retract();
        assert_eq!(result, Some(14));
    }

    #[test]
    fn ap_pure_pure() {
        let ff = FreeAp::<OptionF, Box<dyn Fn(i32) -> i32>>::pure(
            Box::new(|x| x * 2) as Box<dyn Fn(i32) -> i32>
        );
        let fa = FreeAp::<OptionF, i32>::pure(21);
        let result = FreeAp::ap(ff, fa).retract();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn ap_lift_lift() {
        let ff = FreeAp::<OptionF, Box<dyn Fn(i32) -> i32>>::pure(
            Box::new(|x| x + 100) as Box<dyn Fn(i32) -> i32>
        );
        let fa = FreeAp::<OptionF, i32>::lift_f(Some(5));
        let result = FreeAp::ap(ff, fa).retract();
        assert_eq!(result, Some(105));
    }

    #[test]
    fn count_effects_pure() {
        let fa = FreeAp::<OptionF, i32>::pure(42);
        assert_eq!(fa.count_effects(), 0);
    }

    #[test]
    fn count_effects_single() {
        let fa = FreeAp::<OptionF, i32>::lift_f(Some(1));
        assert_eq!(fa.count_effects(), 1);
    }

    #[test]
    fn count_effects_fmapped() {
        let fa = FreeAp::<OptionF, i32>::lift_f(Some(1)).fmap(|x| x + 1);
        assert_eq!(fa.count_effects(), 1);
    }

    #[test]
    fn count_effects_ap() {
        let ff =
            FreeAp::<OptionF, Box<dyn Fn(i32) -> String>>::pure(
                Box::new(|x: i32| format!("{x}")) as Box<dyn Fn(i32) -> String>
            );
        let fa = FreeAp::<OptionF, i32>::lift_f(Some(5));
        let apped = FreeAp::ap(ff, fa);
        // pure contributes 0, lift_f contributes 1
        assert_eq!(apped.count_effects(), 1);
    }

    #[test]
    fn fmap_identity() {
        let fa = FreeAp::<OptionF, i32>::lift_f(Some(7));
        let mapped = fa.fmap(|x| x);
        let result = mapped.retract();
        assert_eq!(result, Some(7));
    }

    #[test]
    fn fmap_composition() {
        let f = |x: i32| x + 1;
        let g = |x: i32| x * 2;

        let left = FreeAp::<OptionF, i32>::lift_f(Some(3))
            .fmap(move |a| g(f(a)))
            .retract();
        let right = FreeAp::<OptionF, i32>::lift_f(Some(3))
            .fmap(f)
            .fmap(g)
            .retract();
        assert_eq!(left, right);
    }

    #[test]
    fn multiple_fmaps() {
        let fa = FreeAp::<OptionF, i32>::lift_f(Some(2))
            .fmap(|x| x * 10)
            .fmap(|x| x + 5);
        let result = fa.retract();
        assert_eq!(result, Some(25)); // 2*10+5
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::hkt::OptionF;
    use proptest::prelude::*;

    proptest! {
        // Functor identity: fmap(id)(x) == x
        #[test]
        fn functor_identity(a in any::<i32>()) {
            let original = FreeAp::<OptionF, i32>::lift_f(Some(a)).retract();
            let mapped = FreeAp::<OptionF, i32>::lift_f(Some(a)).fmap(|x| x).retract();
            prop_assert_eq!(original, mapped);
        }

        // Functor composition: fmap(g . f) == fmap(f) . fmap(g)
        #[test]
        fn functor_composition(a in any::<i32>()) {
            let f = |x: i32| x.wrapping_add(1);
            let g = |x: i32| x.wrapping_mul(2);

            let left = FreeAp::<OptionF, i32>::lift_f(Some(a))
                .fmap(move |x| g(f(x)))
                .retract();
            let right = FreeAp::<OptionF, i32>::lift_f(Some(a))
                .fmap(f)
                .fmap(g)
                .retract();
            prop_assert_eq!(left, right);
        }

        // Applicative identity: ap(pure(id), x) == x
        #[test]
        fn applicative_identity(a in any::<i32>()) {
            let id_fn = FreeAp::<OptionF, Box<dyn Fn(i32) -> i32>>::pure(
                Box::new(|x| x) as Box<dyn Fn(i32) -> i32>,
            );
            let fa = FreeAp::<OptionF, i32>::lift_f(Some(a));
            let result = FreeAp::ap(id_fn, fa).retract();
            let expected = FreeAp::<OptionF, i32>::lift_f(Some(a)).retract();
            prop_assert_eq!(result, expected);
        }

        // Applicative homomorphism: ap(pure(f), pure(x)) == pure(f(x))
        #[test]
        fn applicative_homomorphism(a in any::<i32>()) {
            let ff = FreeAp::<OptionF, Box<dyn Fn(i32) -> i32>>::pure(
                Box::new(|x: i32| x.wrapping_mul(3)) as Box<dyn Fn(i32) -> i32>,
            );
            let fa = FreeAp::<OptionF, i32>::pure(a);
            let left = FreeAp::ap(ff, fa).retract();
            let right = FreeAp::<OptionF, i32>::pure(a.wrapping_mul(3)).retract();
            prop_assert_eq!(left, right);
        }
    }
}
