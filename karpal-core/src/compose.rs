use core::marker::PhantomData;

use crate::functor::Functor;
use crate::hkt::HKT;

/// Marker type for the composition of two type constructors: `(F . G)(A) = F(G(A))`.
///
/// Given `F: HKT` and `G: HKT`, `ComposeF<F, G>` is itself an HKT where
/// `Of<A> = F::Of<G::Of<A>>`.
pub struct ComposeF<F, G>(PhantomData<(F, G)>);

impl<F: HKT, G: HKT> HKT for ComposeF<F, G> {
    type Of<T> = F::Of<G::Of<T>>;
}

/// Functors compose: if F and G are both Functors, so is F . G.
impl<F: Functor, G: Functor> Functor for ComposeF<F, G> {
    fn fmap<A, B>(fga: F::Of<G::Of<A>>, f: impl Fn(A) -> B) -> F::Of<G::Of<B>> {
        F::fmap(fga, |ga| G::fmap(ga, &f))
    }
}

// Note: Apply/Applicative for ComposeF requires distributing function application
// through two layers, which doesn't compose cleanly with Karpal's `fn(A) -> B`
// representation (function pointers rather than closures). Functor composition is
// the primary use case for adjunction-derived monads/comonads.

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(any(feature = "std", feature = "alloc"))]
    use crate::hkt::VecF;
    use crate::hkt::{IdentityF, OptionF};

    #[test]
    fn compose_option_option_fmap() {
        // ComposeF<OptionF, OptionF>::Of<i32> = Option<Option<i32>>
        let val: Option<Option<i32>> = Some(Some(42));
        let result = ComposeF::<OptionF, OptionF>::fmap(val, |x| x + 1);
        assert_eq!(result, Some(Some(43)));
    }

    #[test]
    fn compose_option_option_fmap_outer_none() {
        let val: Option<Option<i32>> = None;
        let result = ComposeF::<OptionF, OptionF>::fmap(val, |x| x + 1);
        assert_eq!(result, None);
    }

    #[test]
    fn compose_option_option_fmap_inner_none() {
        let val: Option<Option<i32>> = Some(None);
        let result = ComposeF::<OptionF, OptionF>::fmap(val, |x| x + 1);
        assert_eq!(result, Some(None));
    }

    #[test]
    fn compose_identity_option() {
        // ComposeF<IdentityF, OptionF>::Of<i32> = Option<i32>
        let val: Option<i32> = Some(10);
        let result = ComposeF::<IdentityF, OptionF>::fmap(val, |x| x * 2);
        assert_eq!(result, Some(20));
    }

    #[test]
    fn compose_option_identity() {
        // ComposeF<OptionF, IdentityF>::Of<i32> = Option<i32>
        let val: Option<i32> = Some(10);
        let result = ComposeF::<OptionF, IdentityF>::fmap(val, |x| x * 2);
        assert_eq!(result, Some(20));
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn compose_vec_option_fmap() {
        // ComposeF<VecF, OptionF>::Of<i32> = Vec<Option<i32>>
        let val: Vec<Option<i32>> = vec![Some(1), None, Some(3)];
        let result = ComposeF::<VecF, OptionF>::fmap(val, |x| x * 10);
        assert_eq!(result, vec![Some(10), None, Some(30)]);
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn compose_option_vec_fmap() {
        // ComposeF<OptionF, VecF>::Of<i32> = Option<Vec<i32>>
        let val: Option<Vec<i32>> = Some(vec![1, 2, 3]);
        let result = ComposeF::<OptionF, VecF>::fmap(val, |x| x + 1);
        assert_eq!(result, Some(vec![2, 3, 4]));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use crate::hkt::OptionF;
    use proptest::prelude::*;

    proptest! {
        // Identity law: fmap(id, fga) == fga
        #[test]
        fn compose_option_option_identity(x in any::<Option<Option<i32>>>()) {
            let result = ComposeF::<OptionF, OptionF>::fmap(x.clone(), |a| a);
            prop_assert_eq!(result, x);
        }

        // Composition law: fmap(g . f, fga) == fmap(g, fmap(f, fga))
        #[test]
        fn compose_option_option_composition(x in any::<Option<Option<i32>>>()) {
            let f = |a: i32| a.wrapping_add(1);
            let g = |a: i32| a.wrapping_mul(2);
            let left = ComposeF::<OptionF, OptionF>::fmap(x.clone(), |a| g(f(a)));
            let right = ComposeF::<OptionF, OptionF>::fmap(
                ComposeF::<OptionF, OptionF>::fmap(x, f),
                g,
            );
            prop_assert_eq!(left, right);
        }
    }
}
