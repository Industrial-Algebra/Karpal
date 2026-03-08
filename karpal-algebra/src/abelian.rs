use crate::group::Group;

/// A `Group` whose operation is commutative.
///
/// Law: `a.combine(b) == b.combine(a)`
///
/// This is a marker trait — commutativity is verified by property tests,
/// not enforced at the type level.
pub trait AbelianGroup: Group {}

macro_rules! impl_abelian {
    ($($t:ty),*) => {
        $( impl AbelianGroup for $t {} )*
    };
}

impl_abelian!(i8, i16, i32, i64, i128, f32, f64);

impl<A: AbelianGroup, B: AbelianGroup> AbelianGroup for (A, B) {}

#[cfg(test)]
mod law_tests {
    use karpal_core::Semigroup;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn i16_commutativity(a in -100i16..100i16, b in -100i16..100i16) {
            prop_assert_eq!(a.combine(b), b.combine(a));
        }

        #[test]
        fn f64_commutativity(a in -100.0f64..100.0, b in -100.0f64..100.0) {
            let left = a + b;
            let right = b + a;
            prop_assert!((left - right).abs() < 1e-10);
        }
    }
}
