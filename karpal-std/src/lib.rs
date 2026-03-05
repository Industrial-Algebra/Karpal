// Standard prelude for the Karpal ecosystem.
//
// Re-exports the most commonly used types and traits from
// `karpal-core`, `karpal-profunctor`, and `karpal-optics`.

pub mod prelude {
    // HKT encoding
    pub use karpal_core::hkt::{HKT, HKT2, OptionF, ResultBF, ResultF, TupleF, VecF};

    // Functor hierarchy
    pub use karpal_core::{
        Alt, Alternative, Applicative, Apply, Bifunctor, Chain, Contravariant, Foldable, Functor,
        FunctorFilter, Monad, Monoid, NaturalTransformation, Plus, Selective, Semigroup,
        Traversable,
    };

    // Profunctor
    pub use karpal_profunctor::{Choice, FnP, Profunctor, Strong};

    // Optics
    pub use karpal_optics::lens::ComposedLens;
    pub use karpal_optics::{Lens, Prism, SimpleLens, SimplePrism};
}

// Crate re-exports for qualified access
pub use karpal_core;
pub use karpal_optics;
pub use karpal_profunctor;

// Macro re-exports
pub use karpal_core::ado_;
pub use karpal_core::do_;

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn prelude_hkt_accessible() {
        // Verify key HKT types are accessible
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
}
