#[cfg(feature = "std")]
use std::{vec, vec::Vec};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{vec, vec::Vec};

use core::marker::PhantomData;

use karpal_core::alternative::Alternative;
use karpal_core::hkt::HKT;

use crate::free_ap::FreeAp;

/// Free Alternative Functor — build alternative computations as data.
///
/// `FreeAlt<F, A>` represents a choice among zero or more applicative
/// computations, each built from `FreeAp<F, A>`. This gives a free
/// `Alternative` instance for any functor `F`.
///
/// ```text
/// FreeAlt f a ≅ [FreeAp f a]
/// ```
///
/// An empty list represents `zero` (failure), a single element is a
/// single computation, and multiple elements represent `alt` (choice).
///
/// # Interpretation
///
/// `retract` collapses the structure into `F`'s own Alternative,
/// combining branches with `F::alt` and handling empty lists with
/// `F::zero`.
pub struct FreeAlt<F: HKT + 'static, A: 'static> {
    alternatives: Vec<FreeAp<F, A>>,
}

impl<F: HKT + 'static, A: 'static> FreeAlt<F, A> {
    /// Wrap a pure value.
    pub fn pure(a: A) -> Self {
        FreeAlt {
            alternatives: vec![FreeAp::pure(a)],
        }
    }

    /// Lift a single effect into the free alternative.
    pub fn lift_f(fa: F::Of<A>) -> Self
    where
        A: Clone,
        F::Of<A>: 'static,
    {
        FreeAlt {
            alternatives: vec![FreeAp::lift_f(fa)],
        }
    }

    /// The empty alternative (zero / failure).
    pub fn zero() -> Self {
        FreeAlt {
            alternatives: Vec::new(),
        }
    }

    /// Combine two alternatives (choice).
    pub fn alt(self, other: FreeAlt<F, A>) -> Self {
        let mut alts = self.alternatives;
        alts.extend(other.alternatives);
        FreeAlt { alternatives: alts }
    }

    /// Map a function over all branches.
    pub fn fmap<B: 'static>(self, f: impl Fn(A) -> B + 'static) -> FreeAlt<F, B> {
        // We need to share f across all branches.
        #[cfg(all(not(feature = "std"), feature = "alloc"))]
        use alloc::rc::Rc;
        #[cfg(feature = "std")]
        use std::rc::Rc;

        let f = Rc::new(f);
        let alternatives = self
            .alternatives
            .into_iter()
            .map(|branch| {
                let f = f.clone();
                branch.fmap(move |a| f(a))
            })
            .collect();
        FreeAlt { alternatives }
    }

    /// Interpret by collapsing into `F`'s own Alternative.
    ///
    /// Requires `F: Alternative`.
    pub fn retract(self) -> F::Of<A>
    where
        F: Alternative,
    {
        let mut iter = self.alternatives.into_iter();
        match iter.next() {
            None => F::zero(),
            Some(first) => {
                let mut result = first.retract();
                for branch in iter {
                    result = F::alt(result, branch.retract());
                }
                result
            }
        }
    }

    /// Count the total number of alternative branches.
    pub fn count_alternatives(&self) -> usize {
        self.alternatives.len()
    }

    /// Count the total number of effects across all branches.
    pub fn count_effects(&self) -> usize {
        self.alternatives.iter().map(|b| b.count_effects()).sum()
    }
}

/// Marker type for `FreeAlt<F, _>`.
///
/// Note: Cannot implement `HKT`, `Functor`, `Alt`, or `Plus` due to Rust's
/// GAT limitations (`type Of<T>` cannot add `T: 'static` in impl when trait
/// doesn't have it). Use `FreeAlt` methods directly.
pub struct FreeAltF<F: HKT + 'static>(PhantomData<F>);

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::OptionF;

    #[test]
    fn pure_retract() {
        let fa = FreeAlt::<OptionF, i32>::pure(42);
        let result = fa.retract();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn lift_f_retract() {
        let fa = FreeAlt::<OptionF, i32>::lift_f(Some(10));
        let result = fa.retract();
        assert_eq!(result, Some(10));
    }

    #[test]
    fn zero_retract() {
        let fa = FreeAlt::<OptionF, i32>::zero();
        let result = fa.retract();
        assert_eq!(result, None);
    }

    #[test]
    fn alt_some_none() {
        let fa = FreeAlt::<OptionF, i32>::lift_f(Some(1));
        let fb = FreeAlt::<OptionF, i32>::lift_f(None);
        // Both are Some(1) and None; alt picks first non-None
        let result = fa.alt(fb).retract();
        assert_eq!(result, Some(1));
    }

    #[test]
    fn alt_none_some() {
        let fa = FreeAlt::<OptionF, i32>::lift_f(None);
        let fb = FreeAlt::<OptionF, i32>::lift_f(Some(2));
        let result = fa.alt(fb).retract();
        assert_eq!(result, Some(2));
    }

    #[test]
    fn alt_none_none() {
        let fa = FreeAlt::<OptionF, i32>::lift_f(None);
        let fb = FreeAlt::<OptionF, i32>::lift_f(None);
        let result = fa.alt(fb).retract();
        assert_eq!(result, None);
    }

    #[test]
    fn alt_some_some() {
        let fa = FreeAlt::<OptionF, i32>::lift_f(Some(1));
        let fb = FreeAlt::<OptionF, i32>::lift_f(Some(2));
        // Option::alt takes the first Some
        let result = fa.alt(fb).retract();
        assert_eq!(result, Some(1));
    }

    #[test]
    fn fmap_alt() {
        let fa = FreeAlt::<OptionF, i32>::lift_f(Some(5)).fmap(|x| x * 2);
        let result = fa.retract();
        assert_eq!(result, Some(10));
    }

    #[test]
    fn count_alternatives_zero() {
        let fa = FreeAlt::<OptionF, i32>::zero();
        assert_eq!(fa.count_alternatives(), 0);
    }

    #[test]
    fn count_alternatives_single() {
        let fa = FreeAlt::<OptionF, i32>::pure(1);
        assert_eq!(fa.count_alternatives(), 1);
    }

    #[test]
    fn count_alternatives_multiple() {
        let fa = FreeAlt::<OptionF, i32>::lift_f(Some(1))
            .alt(FreeAlt::lift_f(Some(2)))
            .alt(FreeAlt::lift_f(Some(3)));
        assert_eq!(fa.count_alternatives(), 3);
    }

    #[test]
    fn count_effects_across_branches() {
        let fa = FreeAlt::<OptionF, i32>::lift_f(Some(1))
            .alt(FreeAlt::pure(2))
            .alt(FreeAlt::lift_f(Some(3)));
        // lift_f: 1 effect, pure: 0 effects, lift_f: 1 effect
        assert_eq!(fa.count_effects(), 2);
    }

    #[test]
    fn alt_multiple() {
        let fa = FreeAlt::<OptionF, i32>::lift_f(Some(1));
        let fb = FreeAlt::<OptionF, i32>::lift_f(Some(2));
        let result = fa.alt(fb);
        assert_eq!(result.count_alternatives(), 2);
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
        fn functor_identity(a in any::<i32>()) {
            let original = FreeAlt::<OptionF, i32>::lift_f(Some(a)).retract();
            let mapped = FreeAlt::<OptionF, i32>::lift_f(Some(a)).fmap(|x| x).retract();
            prop_assert_eq!(original, mapped);
        }

        // Functor composition
        #[test]
        fn functor_composition(a in any::<i32>()) {
            let f = |x: i32| x.wrapping_add(1);
            let g = |x: i32| x.wrapping_mul(2);

            let left = FreeAlt::<OptionF, i32>::lift_f(Some(a))
                .fmap(move |x| g(f(x)))
                .retract();
            let right = FreeAlt::<OptionF, i32>::lift_f(Some(a))
                .fmap(f)
                .fmap(g)
                .retract();
            prop_assert_eq!(left, right);
        }

        // Alt associativity
        #[test]
        fn alt_associativity(a in any::<i32>(), b in any::<i32>(), c in any::<i32>()) {
            let left = FreeAlt::<OptionF, i32>::lift_f(Some(a))
                .alt(FreeAlt::lift_f(Some(b)))
                .alt(FreeAlt::lift_f(Some(c)))
                .retract();
            let right = FreeAlt::<OptionF, i32>::lift_f(Some(a))
                .alt(FreeAlt::lift_f(Some(b)).alt(FreeAlt::lift_f(Some(c))))
                .retract();
            prop_assert_eq!(left, right);
        }

        // Plus identity: alt(zero, x) == x and alt(x, zero) == x
        #[test]
        fn plus_left_identity(a in any::<i32>()) {
            let left = FreeAlt::<OptionF, i32>::zero()
                .alt(FreeAlt::lift_f(Some(a)))
                .retract();
            let right = FreeAlt::<OptionF, i32>::lift_f(Some(a)).retract();
            prop_assert_eq!(left, right);
        }

        #[test]
        fn plus_right_identity(a in any::<i32>()) {
            let left = FreeAlt::<OptionF, i32>::lift_f(Some(a))
                .alt(FreeAlt::zero())
                .retract();
            let right = FreeAlt::<OptionF, i32>::lift_f(Some(a)).retract();
            prop_assert_eq!(left, right);
        }
    }
}
