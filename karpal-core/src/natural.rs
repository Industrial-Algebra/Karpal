use crate::hkt::HKT;

/// Natural transformation: a mapping between two functors that preserves structure.
///
/// Laws:
/// - Naturality: `fmap_G(f, transform(fa)) == transform(fmap_F(f, fa))`
pub trait NaturalTransformation<F: HKT, G: HKT> {
    fn transform<A>(fa: F::Of<A>) -> G::Of<A>;
}

/// Converts Option to Vec (None → [], Some(a) → [a]).
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct OptionToVec;

#[cfg(any(feature = "std", feature = "alloc"))]
impl NaturalTransformation<crate::hkt::OptionF, crate::hkt::VecF> for OptionToVec {
    fn transform<A>(fa: Option<A>) -> Vec<A> {
        match fa {
            Some(a) => vec![a],
            None => vec![],
        }
    }
}

/// Converts Vec to Option by taking the first element.
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct VecHeadToOption;

#[cfg(any(feature = "std", feature = "alloc"))]
impl NaturalTransformation<crate::hkt::VecF, crate::hkt::OptionF> for VecHeadToOption {
    fn transform<A>(fa: Vec<A>) -> Option<A> {
        fa.into_iter().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_to_vec_some() {
        assert_eq!(OptionToVec::transform(Some(42)), vec![42]);
    }

    #[test]
    fn option_to_vec_none() {
        assert_eq!(OptionToVec::transform(None::<i32>), Vec::<i32>::new());
    }

    #[test]
    fn vec_head_to_option_non_empty() {
        assert_eq!(VecHeadToOption::transform(vec![1, 2, 3]), Some(1));
    }

    #[test]
    fn vec_head_to_option_empty() {
        assert_eq!(VecHeadToOption::transform(Vec::<i32>::new()), None);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use crate::functor::Functor;
    use crate::hkt::{OptionF, VecF};
    use proptest::prelude::*;

    proptest! {
        // Naturality: fmap_G(f, transform(fa)) == transform(fmap_F(f, fa))
        #[test]
        fn option_to_vec_naturality(x in any::<Option<i32>>()) {
            let f = |a: i32| a.wrapping_add(1);
            let left = VecF::fmap(OptionToVec::transform(x), f);
            let right = OptionToVec::transform(OptionF::fmap(x, f));
            prop_assert_eq!(left, right);
        }

        #[test]
        fn vec_head_to_option_naturality(x in prop::collection::vec(any::<i32>(), 0..10)) {
            let f = |a: i32| a.wrapping_add(1);
            let left = OptionF::fmap(VecHeadToOption::transform(x.clone()), f);
            let right = VecHeadToOption::transform(VecF::fmap(x, f));
            prop_assert_eq!(left, right);
        }
    }
}
