#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;

use karpal_core::functor::Functor;
use karpal_core::hkt::HKT;

use crate::fix::Fix;
use crate::schemes::ana;

/// The greatest fixed point of a functor `F`.
///
/// `Nu<F, Seed>` represents a potentially infinite corecursive structure.
/// It stores a seed value and a coalgebra that can observe one layer at a time.
///
/// The `Seed` type parameter is exposed because Rust lacks existential types.
/// This is the same pragmatic compromise used by `Lan` in karpal-free.
#[allow(clippy::type_complexity)]
pub struct Nu<F: HKT, Seed> {
    pub seed: Seed,
    pub coalgebra: Box<dyn Fn(&Seed) -> F::Of<Seed>>,
}

impl<F: HKT, Seed> Nu<F, Seed> {
    /// Create a `Nu` from a seed and a coalgebra.
    pub fn new(seed: Seed, coalgebra: impl Fn(&Seed) -> F::Of<Seed> + 'static) -> Self {
        Nu {
            seed,
            coalgebra: Box::new(coalgebra),
        }
    }

    /// Apply the coalgebra once to observe one layer.
    pub fn observe(&self) -> F::Of<Seed> {
        (self.coalgebra)(&self.seed)
    }

    /// Convert to `Fix<F>` by fully unfolding via anamorphism.
    ///
    /// This will diverge for truly infinite structures — only call on
    /// coalgebras that eventually terminate.
    pub fn to_fix(self) -> Fix<F>
    where
        F: Functor,
        Seed: Clone,
    {
        let coalg = self.coalgebra;
        ana(|s: Seed| coalg(&s), self.seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::OptionF;

    #[test]
    fn observe_once() {
        let nu = Nu::<OptionF, u32>::new(3, |&s| if s == 0 { None } else { Some(s - 1) });
        assert_eq!(nu.observe(), Some(2));
    }

    #[test]
    fn observe_at_zero() {
        let nu = Nu::<OptionF, u32>::new(0, |&s| if s == 0 { None } else { Some(s - 1) });
        assert_eq!(nu.observe(), None);
    }

    #[test]
    fn to_fix_converts() {
        let nu = Nu::<OptionF, u32>::new(3, |&s| if s == 0 { None } else { Some(s - 1) });
        let fixed = nu.to_fix();
        // Should be Succ(Succ(Succ(Zero))) = 3
        fn count(f: Fix<OptionF>) -> u32 {
            match f.unfix() {
                None => 0,
                Some(pred) => 1 + count(pred),
            }
        }
        assert_eq!(count(fixed), 3);
    }
}
