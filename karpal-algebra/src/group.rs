use karpal_core::Monoid;

/// A `Monoid` where every element has an inverse.
///
/// Law: `a.combine(a.invert()) == empty()`
/// Law: `a.invert().combine(a) == empty()`
pub trait Group: Monoid {
    fn invert(self) -> Self;

    fn combine_inverse(self, other: Self) -> Self
    where
        Self: Sized,
    {
        self.combine(other.invert())
    }
}

macro_rules! impl_signed_group {
    ($($t:ty),*) => {
        $(
            impl Group for $t {
                fn invert(self) -> Self {
                    -self
                }
            }
        )*
    };
}

impl_signed_group!(i8, i16, i32, i64, i128, f32, f64);

impl<A: Group, B: Group> Group for (A, B) {
    fn invert(self) -> Self {
        (self.0.invert(), self.1.invert())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i32_invert() {
        assert_eq!(5i32.invert(), -5);
        assert_eq!((-3i32).invert(), 3);
    }

    #[test]
    fn i32_combine_inverse() {
        assert_eq!(10i32.combine_inverse(3), 7);
    }

    #[test]
    fn f64_invert() {
        assert!((1.5f64.invert() - (-1.5)).abs() < 1e-10);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use karpal_core::Semigroup;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn left_inverse(a in -100i16..100i16) {
            prop_assert_eq!(a.invert().combine(a), i16::empty());
        }

        #[test]
        fn right_inverse(a in -100i16..100i16) {
            prop_assert_eq!(a.combine(a.invert()), i16::empty());
        }

        #[test]
        fn combine_inverse_is_combine_invert(a in -50i16..50i16, b in -50i16..50i16) {
            prop_assert_eq!(a.combine_inverse(b), a.combine(b.invert()));
        }
    }
}
