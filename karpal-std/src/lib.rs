// Standard prelude for the Karpal ecosystem.
//
// Re-exports the most commonly used types and traits from
// `karpal-core`, `karpal-profunctor`, and `karpal-optics`.

/// Prelude module — `use karpal_std::prelude::*` to import everything.
pub mod prelude {
    // HKT encoding
    pub use karpal_core::hkt::{
        EnvF, HKT, HKT2, IdentityF, NonEmptyVec, NonEmptyVecF, OptionF, ResultBF, ResultF, StoreF,
        TracedF, TupleF, VecF,
    };

    // Functor hierarchy
    pub use karpal_core::{
        Alt, Alternative, Applicative, Apply, Chain, Functor, FunctorFilter, Monad,
    };

    // Foldable / Traversable
    pub use karpal_core::{Foldable, Traversable};

    // Comonad hierarchy
    pub use karpal_core::{Comonad, ComonadEnv, ComonadStore, ComonadTraced, Extend};

    // Bifunctor, Selective, NaturalTransformation
    pub use karpal_core::{Bifunctor, NaturalTransformation, Selective};

    // Invariant
    pub use karpal_core::Invariant;

    // Contravariant hierarchy
    pub use karpal_core::{Conclude, Contravariant, Decide, Divide, Divisible, PredicateF};

    // Plus
    pub use karpal_core::Plus;

    // Algebraic typeclasses
    pub use karpal_core::{Monoid, Semigroup};

    // Profunctor
    pub use karpal_profunctor::{Choice, FnP, Profunctor, Strong};

    // Optics
    pub use karpal_optics::lens::ComposedLens;
    pub use karpal_optics::lens::{Lens, SimpleLens};
    pub use karpal_optics::optic::Optic;
    pub use karpal_optics::{Prism, SimplePrism};

    // Arrow hierarchy
    pub use karpal_arrow::{
        Arrow, ArrowApply, ArrowChoice, ArrowLoop, ArrowPlus, ArrowZero, Category, CokleisliF, FnA,
        KleisliF, Semigroupoid,
    };

    // Arrow macros
    pub use karpal_arrow::{impl_cokleisli, impl_cokleisli_env};

    // Free constructions
    pub use karpal_free::{
        Codensity, CodensityF, Cofree, CofreeF, Coyoneda, CoyonedaF, Day, DayF, Density, DensityF,
        Free, FreeAlt, FreeAltF, FreeAp, FreeApF, FreeF, Freer, FreerF, Lan, LanF, Ran, RanMapped,
        Yoneda, YonedaF,
    };

    // Recursion schemes
    pub use karpal_recursion::{
        Either, Fix, FixF, Mu, Nu, ana, apo, cata, chrono, futu, histo, hylo, para, zygo,
    };

    // Macros
    pub use karpal_core::{ado_, do_};
}

// Crate re-exports for qualified access
pub use karpal_arrow;
pub use karpal_core;
pub use karpal_free;
pub use karpal_optics;
pub use karpal_profunctor;
pub use karpal_recursion;

// Macro re-exports
pub use karpal_core::ado_;
pub use karpal_core::do_;

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn prelude_hkt_accessible() {
        let _: <OptionF as HKT>::Of<i32> = Some(42);
        let _: <VecF as HKT>::Of<i32> = vec![1, 2, 3];
        let _: <ResultF<String> as HKT>::Of<i32> = Ok(42);
    }

    #[test]
    fn prelude_functor_accessible() {
        let result = <OptionF as Functor>::fmap(Some(1), |x| x + 1);
        assert_eq!(result, Some(2));
    }

    #[test]
    fn prelude_lens_accessible() {
        let lens: SimpleLens<(i32, i32), i32> = Lens::new(|p: &(i32, i32)| p.0, |p, x| (x, p.1));
        assert_eq!(lens.get(&(1, 2)), 1);
    }

    #[test]
    fn prelude_prism_accessible() {
        let prism: SimplePrism<Option<i32>, i32> = Prism::new(
            |s| match s {
                Some(v) => Ok(v),
                None => Err(None),
            },
            Some,
        );
        assert_eq!(prism.preview(&Some(42)), Some(42));
        assert_eq!(prism.preview(&None), None);
    }

    #[test]
    fn prelude_profunctor_accessible() {
        let f: <FnP as HKT2>::P<i32, i32> = Box::new(|x| x + 1);
        assert_eq!(f(1), 2);
    }

    #[test]
    fn prelude_arrow_accessible() {
        let double = <FnA as Arrow>::arr(|x: i32| x * 2);
        let inc = <FnA as Arrow>::arr(|x: i32| x + 1);
        let composed = <FnA as Semigroupoid>::compose(inc, double);
        assert_eq!(composed(5), 11); // (5 * 2) + 1
    }
}
