#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::rc::Rc;

use core::marker::PhantomData;

use karpal_core::functor::Functor;
use karpal_core::hkt::HKT;

/// The fixed point of a functor `F`.
///
/// `Fix<F>` ties the recursive knot: `Fix<F> ≅ F<Fix<F>>`.
/// It is the core type for recursion schemes — catamorphisms fold
/// a `Fix<F>` down, anamorphisms build one up.
///
/// Uses `Rc` for indirection, which makes cloning cheap (reference count
/// bump). This is essential for paramorphism, which needs to both
/// preserve and consume each subterm.
pub struct Fix<F: HKT>(Rc<F::Of<Fix<F>>>);

impl<F: HKT> Clone for Fix<F> {
    fn clone(&self) -> Self {
        Fix(Rc::clone(&self.0))
    }
}

impl<F: HKT> Fix<F> {
    /// Wrap one layer of `F<Fix<F>>` into `Fix<F>`.
    pub fn new(f: F::Of<Fix<F>>) -> Self {
        Fix(Rc::new(f))
    }

    /// Unwrap one layer, consuming the `Fix`.
    ///
    /// If this is the sole owner, the inner value is moved out.
    /// Otherwise a shallow clone is made (each child `Fix` inside
    /// is `Rc`-cloned, which is O(1)).
    pub fn unfix(self) -> F::Of<Fix<F>>
    where
        F::Of<Fix<F>>: Clone,
    {
        match Rc::try_unwrap(self.0) {
            Ok(val) => val,
            Err(rc) => (*rc).clone(),
        }
    }

    /// Borrow one layer without consuming.
    pub fn unfix_ref(&self) -> &F::Of<Fix<F>> {
        &self.0
    }
}

/// `Mu<F>` is the least fixed point — structurally identical to `Fix<F>` in Rust,
/// since Rust cannot enforce finiteness at the type level.
pub type Mu<F> = Fix<F>;

/// HKT marker for `Fix<F>`. Primarily used for type-level programming.
pub struct FixF<F: HKT>(PhantomData<F>);

impl<F: HKT> HKT for FixF<F> {
    type Of<T> = Fix<F>;
}

impl<F: HKT + Functor> Functor for FixF<F> {
    fn fmap<A, B>(fa: Fix<F>, _f: impl Fn(A) -> B) -> Fix<F> {
        fa
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::OptionF;

    fn zero() -> Fix<OptionF> {
        Fix::new(None)
    }

    fn succ(n: Fix<OptionF>) -> Fix<OptionF> {
        Fix::new(Some(n))
    }

    fn nat(n: u32) -> Fix<OptionF> {
        let mut result = zero();
        for _ in 0..n {
            result = succ(result);
        }
        result
    }

    fn to_u32(n: Fix<OptionF>) -> u32 {
        match n.unfix() {
            None => 0,
            Some(pred) => 1 + to_u32(pred),
        }
    }

    #[test]
    fn fix_unfix_roundtrip() {
        assert_eq!(to_u32(nat(5)), 5);
    }

    #[test]
    fn zero_is_none() {
        let z = zero();
        assert!(z.unfix_ref().is_none());
    }

    #[test]
    fn succ_is_some() {
        let one = succ(zero());
        assert!(one.unfix_ref().is_some());
    }

    #[test]
    fn mu_is_fix() {
        let n: Mu<OptionF> = nat(3);
        assert_eq!(to_u32(n), 3);
    }

    #[test]
    fn clone_fix() {
        let n = nat(4);
        let n2 = n.clone();
        assert_eq!(to_u32(n), 4);
        assert_eq!(to_u32(n2), 4);
    }
}
