#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;

use core::marker::PhantomData;

use karpal_core::applicative::Applicative;
use karpal_core::chain::Chain;
use karpal_core::hkt::HKT;

/// Private dyn-safe trait for the Codensity computation tree.
///
/// The only eliminator is `to_monad`, which collapses the tree into `F::Of<A>`
/// using `F::pure` and `F::chain`. The generic `lower_with` (∀R) cannot be
/// made dyn-safe, so we only support lowering into F's own monad.
trait CodensityInner<F: HKT + 'static, A: 'static> {
    /// Collapse this computation into `F::Of<A>`.
    fn to_monad(self: Box<Self>) -> F::Of<A>
    where
        F: Applicative + Chain;
}

/// Pure layer: stores a value.
struct CodensityPure<F: HKT + 'static, A: 'static> {
    value: A,
    _marker: PhantomData<F>,
}

impl<F: HKT + 'static, A: 'static> CodensityInner<F, A> for CodensityPure<F, A> {
    fn to_monad(self: Box<Self>) -> F::Of<A>
    where
        F: Applicative + Chain,
    {
        F::pure(self.value)
    }
}

/// Map layer: wraps an inner computation with a transform.
struct CodensityMap<F: HKT + 'static, Src: 'static, A: 'static> {
    inner: Box<dyn CodensityInner<F, Src>>,
    transform: Box<dyn Fn(Src) -> A>,
}

impl<F: HKT + 'static, Src: 'static, A: 'static> CodensityInner<F, A> for CodensityMap<F, Src, A> {
    fn to_monad(self: Box<Self>) -> F::Of<A>
    where
        F: Applicative + Chain,
    {
        let f_src: F::Of<Src> = self.inner.to_monad();
        F::fmap(f_src, self.transform)
    }
}

/// Bind layer: wraps an inner computation with a monadic bind function.
struct CodensityBind<F: HKT + 'static, Src: 'static, A: 'static> {
    inner: Box<dyn CodensityInner<F, Src>>,
    bind_fn: Box<dyn Fn(Src) -> Codensity<F, A>>,
}

impl<F: HKT + 'static, Src: 'static, A: 'static> CodensityInner<F, A> for CodensityBind<F, Src, A> {
    fn to_monad(self: Box<Self>) -> F::Of<A>
    where
        F: Applicative + Chain,
    {
        let f_src: F::Of<Src> = self.inner.to_monad();
        let bind_fn = self.bind_fn;
        F::chain(f_src, move |src| (bind_fn)(src).inner.to_monad())
    }
}

/// Codensity Monad — the CPS transform of a type constructor `F`.
///
/// `Codensity<F, A> ≅ ∀R. (A → F R) → F R`
///
/// This is the right Kan extension of `F` along itself (`Ran F F A`),
/// specialised into a concrete type. The key property: `pure`, `fmap`,
/// and `chain` require **no bounds on `F`** — only `to_monad` needs
/// `F: Applicative + Chain`.
///
/// # Use cases
///
/// - **Monad transformer improvement**: wrapping a free monad in Codensity
///   can improve asymptotic performance of left-associated binds.
/// - **CPS conversion**: build computations in CPS, then interpret into
///   the target monad via `to_monad`.
///
/// Note: Due to Rust's GAT limitations (`type Of<T>` cannot add `T: 'static`),
/// `CodensityF` does not implement `HKT` or `Monad`. Use inherent methods.
pub struct Codensity<F: HKT + 'static, A: 'static> {
    inner: Box<dyn CodensityInner<F, A>>,
}

impl<F: HKT + 'static, A: 'static> Codensity<F, A> {
    /// Wrap a pure value. No bounds on `F` required.
    pub fn pure(a: A) -> Self {
        Codensity {
            inner: Box::new(CodensityPure {
                value: a,
                _marker: PhantomData,
            }),
        }
    }

    /// Map a function over the result. No bounds on `F` required.
    pub fn fmap<B: 'static>(self, f: impl Fn(A) -> B + 'static) -> Codensity<F, B> {
        Codensity {
            inner: Box::new(CodensityMap {
                inner: self.inner,
                transform: Box::new(f),
            }),
        }
    }

    /// Monadic bind. No bounds on `F` required.
    pub fn chain<B: 'static>(self, f: impl Fn(A) -> Codensity<F, B> + 'static) -> Codensity<F, B> {
        Codensity {
            inner: Box::new(CodensityBind {
                inner: self.inner,
                bind_fn: Box::new(f),
            }),
        }
    }

    /// Collapse the computation into `F::Of<A>`.
    ///
    /// This is the standard way to extract a result from Codensity.
    /// Requires `F: Applicative + Chain` (i.e., F must be a monad).
    pub fn to_monad(self) -> F::Of<A>
    where
        F: Applicative + Chain,
    {
        self.inner.to_monad()
    }
}

/// Marker type for `Codensity<F, _>`.
///
/// Note: Cannot implement `HKT` or `Monad` due to Rust's GAT limitations.
/// Use `Codensity::pure`, `Codensity::fmap`, `Codensity::chain` directly.
pub struct CodensityF<F: HKT + 'static>(PhantomData<F>);

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::OptionF;

    #[test]
    fn pure_to_monad() {
        let c = Codensity::<OptionF, i32>::pure(42);
        let result = c.to_monad();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn fmap_to_monad() {
        let c = Codensity::<OptionF, i32>::pure(5).fmap(|x| x * 3);
        let result = c.to_monad();
        assert_eq!(result, Some(15));
    }

    #[test]
    fn chain_to_monad() {
        let c = Codensity::<OptionF, i32>::pure(10).chain(|x| Codensity::pure(x + 1));
        let result = c.to_monad();
        assert_eq!(result, Some(11));
    }

    #[test]
    fn chain_multiple() {
        let c = Codensity::<OptionF, i32>::pure(1)
            .chain(|x| Codensity::pure(x + 1))
            .chain(|x| Codensity::pure(x * 10))
            .chain(|x| Codensity::pure(x + 5));
        let result = c.to_monad();
        // (1 + 1) * 10 + 5 = 25
        assert_eq!(result, Some(25));
    }

    #[test]
    fn fmap_then_chain() {
        let c = Codensity::<OptionF, i32>::pure(3)
            .fmap(|x| x * 2)
            .chain(|x| Codensity::pure(x + 100));
        let result = c.to_monad();
        assert_eq!(result, Some(106));
    }

    #[test]
    fn chain_associativity() {
        // (m >>= f) >>= g
        let left = Codensity::<OptionF, i32>::pure(5)
            .chain(|x| Codensity::pure(x + 1))
            .chain(|x| Codensity::pure(x * 2));

        // m >>= (\x -> f(x) >>= g)
        let right = Codensity::<OptionF, i32>::pure(5)
            .chain(|x| Codensity::<OptionF, i32>::pure(x + 1).chain(|y| Codensity::pure(y * 2)));

        assert_eq!(left.to_monad(), right.to_monad());
        // Both should be Some((5 + 1) * 2) = Some(12)
    }

    #[test]
    fn monad_left_identity() {
        let left = Codensity::<OptionF, i32>::pure(4).chain(|x| Codensity::pure(x * 3));
        assert_eq!(left.to_monad(), Some(12));
    }

    #[test]
    fn monad_right_identity() {
        let m = Codensity::<OptionF, i32>::pure(42);
        let result = m.chain(Codensity::pure);
        assert_eq!(result.to_monad(), Some(42));
    }

    #[test]
    fn fmap_changes_type() {
        let c = Codensity::<OptionF, i32>::pure(42).fmap(|x| format!("val={x}"));
        assert_eq!(c.to_monad(), Some("val=42".to_string()));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::hkt::OptionF;
    use proptest::prelude::*;

    proptest! {
        // Functor identity: fmap(id) == id
        #[test]
        fn functor_identity(x in any::<i32>()) {
            let result = Codensity::<OptionF, i32>::pure(x)
                .fmap(|a| a)
                .to_monad();
            prop_assert_eq!(result, Some(x));
        }

        // Functor composition: fmap(g . f) == fmap(f) . fmap(g)
        #[test]
        fn functor_composition(x in any::<i32>()) {
            let f = |a: i32| a.wrapping_add(1);
            let g = |a: i32| a.wrapping_mul(2);

            let left = Codensity::<OptionF, i32>::pure(x)
                .fmap(move |a| g(f(a)))
                .to_monad();
            let right = Codensity::<OptionF, i32>::pure(x)
                .fmap(f)
                .fmap(g)
                .to_monad();
            prop_assert_eq!(left, right);
        }

        // Monad left identity: pure(a) >>= f == f(a)
        #[test]
        fn monad_left_identity(x in any::<i32>()) {
            let left = Codensity::<OptionF, i32>::pure(x)
                .chain(|a| Codensity::pure(a.wrapping_mul(2)))
                .to_monad();
            let right = Codensity::<OptionF, i32>::pure(x.wrapping_mul(2)).to_monad();
            prop_assert_eq!(left, right);
        }

        // Monad right identity: m >>= pure == m
        #[test]
        fn monad_right_identity(x in any::<i32>()) {
            let result = Codensity::<OptionF, i32>::pure(x)
                .chain(Codensity::pure)
                .to_monad();
            prop_assert_eq!(result, Some(x));
        }

        // Monad associativity: (m >>= f) >>= g == m >>= (\x -> f(x) >>= g)
        #[test]
        fn monad_associativity(x in any::<i32>()) {
            let left = Codensity::<OptionF, i32>::pure(x)
                .chain(|a| Codensity::pure(a.wrapping_add(1)))
                .chain(|a| Codensity::pure(a.wrapping_mul(2)))
                .to_monad();

            let right = Codensity::<OptionF, i32>::pure(x)
                .chain(|a| {
                    Codensity::<OptionF, i32>::pure(a.wrapping_add(1))
                        .chain(|b| Codensity::pure(b.wrapping_mul(2)))
                })
                .to_monad();

            prop_assert_eq!(left, right);
        }
    }
}
