#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;

use core::marker::PhantomData;

use karpal_core::hkt::{HKT, IdentityF};
use karpal_core::natural::NaturalTransformation;

use crate::coyoneda::Coyoneda;

/// Left Kan Extension — `Lan G H A ≅ ∃B. (G B → A, H B)`.
///
/// Encodes a value `H B` together with a way to extract `A` from `G B`.
/// The intermediate type `B` is visible in Rust (unlike Haskell's existential)
/// because Rust lacks first-class existential types.
///
/// # Key properties
///
/// - **Functor in A**: `fmap` composes onto the extract function, no bounds needed.
/// - **Specialisation**: `Lan<IdentityF, F, A, B>` is isomorphic to `Coyoneda<F, A>`.
/// - **Lower**: Given a natural transformation `H ~> G`, collapse to `A`.
pub struct Lan<G: HKT, H: HKT, A, B> {
    extract_fn: Box<dyn Fn(G::Of<B>) -> A>,
    source: H::Of<B>,
    _marker: PhantomData<G>,
}

impl<G: HKT + 'static, H: HKT + 'static, A: 'static, B: 'static> Lan<G, H, A, B> {
    /// Construct a Lan from a source value and an extract function.
    pub fn new(source: H::Of<B>, f: impl Fn(G::Of<B>) -> A + 'static) -> Self {
        Lan {
            extract_fn: Box::new(f),
            source,
            _marker: PhantomData,
        }
    }

    /// Map a function over the result type. No bounds on G or H required.
    pub fn fmap<C: 'static>(self, f: impl Fn(A) -> C + 'static) -> Lan<G, H, C, B> {
        let old_extract = self.extract_fn;
        Lan {
            extract_fn: Box::new(move |gb| f(old_extract(gb))),
            source: self.source,
            _marker: PhantomData,
        }
    }

    /// Collapse the Lan using a natural transformation `NT: H ~> G`.
    ///
    /// Applies the nat-trans to the source, then extracts the result.
    pub fn lower<NT: NaturalTransformation<H, G>>(self) -> A {
        (self.extract_fn)(NT::transform(self.source))
    }
}

/// When `G = IdentityF`, `Lan<IdentityF, H, A, B>` is isomorphic to `Coyoneda<H, A, B>`.
impl<H: HKT + 'static, A: 'static, B: 'static> Lan<IdentityF, H, A, B>
where
    H::Of<B>: 'static,
{
    /// Convert to `Coyoneda<H, A, B>`.
    ///
    /// `Lan<IdentityF, H, A, B> ≅ (B → A, H B) = Coyoneda<H, A, B>`
    pub fn to_coyoneda(self) -> Coyoneda<H, A, B> {
        let extract = self.extract_fn;
        Coyoneda::<H, _, _>::lift(self.source).fmap(move |b| (*extract)(b))
    }
}

/// Marker type for `Lan<G, H, _, B>`.
///
/// Note: Cannot implement `HKT` or `Functor` due to Rust's GAT limitations
/// (extra type parameters G, H, B cannot be threaded through `type Of<T>`).
/// Use `Lan::fmap` directly.
pub struct LanF<G: HKT, H: HKT, B>(PhantomData<(G, H, B)>);

#[cfg(test)]
mod tests {
    use super::*;
    use karpal_core::hkt::OptionF;

    #[test]
    fn new_and_lower() {
        struct OptToId;
        impl NaturalTransformation<OptionF, IdentityF> for OptToId {
            fn transform<A>(fa: Option<A>) -> A {
                fa.unwrap()
            }
        }

        let lan = Lan::<IdentityF, OptionF, i32, i32>::new(Some(42), |x| x);
        let result = lan.lower::<OptToId>();
        assert_eq!(result, 42);
    }

    #[test]
    fn fmap_lan() {
        struct OptToId;
        impl NaturalTransformation<OptionF, IdentityF> for OptToId {
            fn transform<A>(fa: Option<A>) -> A {
                fa.unwrap()
            }
        }

        let lan = Lan::<IdentityF, OptionF, i32, i32>::new(Some(5), |x| x);
        let mapped = lan.fmap(|x| x * 3);
        let result = mapped.lower::<OptToId>();
        assert_eq!(result, 15);
    }

    #[test]
    fn fmap_composition() {
        struct OptToId;
        impl NaturalTransformation<OptionF, IdentityF> for OptToId {
            fn transform<A>(fa: Option<A>) -> A {
                fa.unwrap()
            }
        }

        let f = |x: i32| x + 1;
        let g = |x: i32| x * 2;

        let left = Lan::<IdentityF, OptionF, i32, i32>::new(Some(5), |x| x).fmap(move |a| g(f(a)));
        let right = Lan::<IdentityF, OptionF, i32, i32>::new(Some(5), |x| x)
            .fmap(f)
            .fmap(g);

        assert_eq!(left.lower::<OptToId>(), right.lower::<OptToId>());
    }

    #[test]
    fn fmap_identity() {
        struct OptToId;
        impl NaturalTransformation<OptionF, IdentityF> for OptToId {
            fn transform<A>(fa: Option<A>) -> A {
                fa.unwrap()
            }
        }

        let lan = Lan::<IdentityF, OptionF, i32, i32>::new(Some(7), |x| x);
        let mapped = lan.fmap(|x| x);
        assert_eq!(mapped.lower::<OptToId>(), 7);
    }

    #[test]
    fn to_coyoneda_roundtrip() {
        let lan =
            Lan::<IdentityF, OptionF, String, i32>::new(Some(42), |x: i32| format!("val={x}"));
        let coy = lan.to_coyoneda();
        let result = coy.lower();
        assert_eq!(result, Some("val=42".to_string()));
    }

    #[test]
    fn to_coyoneda_with_fmap() {
        let lan = Lan::<IdentityF, OptionF, i32, i32>::new(Some(10), |x| x);
        let coy = lan.fmap(|x| x + 5).to_coyoneda();
        let result = coy.lower();
        assert_eq!(result, Some(15));
    }

    #[test]
    fn lower_with_extract_transform() {
        // Lan<OptionF, OptionF, String, i32>:
        // extract: Option<i32> -> String
        // source: Option<i32>
        // lower with id nat-trans: extract(source)
        struct OptionId;
        impl NaturalTransformation<OptionF, OptionF> for OptionId {
            fn transform<A>(fa: Option<A>) -> Option<A> {
                fa
            }
        }

        let lan =
            Lan::<OptionF, OptionF, String, i32>::new(Some(99), |opt| format!("got: {:?}", opt));
        let result = lan.lower::<OptionId>();
        assert_eq!(result, "got: Some(99)");
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::hkt::OptionF;
    use proptest::prelude::*;

    struct OptToId;
    impl NaturalTransformation<OptionF, IdentityF> for OptToId {
        fn transform<A>(fa: Option<A>) -> A {
            fa.unwrap()
        }
    }

    proptest! {
        // Functor identity: fmap(id, lan).lower() == lan.lower()
        #[test]
        fn functor_identity(x in any::<i32>()) {
            let result = Lan::<IdentityF, OptionF, i32, i32>::new(Some(x), |a| a)
                .fmap(|a| a)
                .lower::<OptToId>();
            prop_assert_eq!(result, x);
        }

        // Functor composition: fmap(g . f) == fmap(g) . fmap(f)
        #[test]
        fn functor_composition(x in any::<i32>()) {
            let f = |a: i32| a.wrapping_add(1);
            let g = |a: i32| a.wrapping_mul(2);

            let left = Lan::<IdentityF, OptionF, i32, i32>::new(Some(x), |a| a)
                .fmap(move |a| g(f(a)))
                .lower::<OptToId>();
            let right = Lan::<IdentityF, OptionF, i32, i32>::new(Some(x), |a| a)
                .fmap(f)
                .fmap(g)
                .lower::<OptToId>();
            prop_assert_eq!(left, right);
        }
    }
}
