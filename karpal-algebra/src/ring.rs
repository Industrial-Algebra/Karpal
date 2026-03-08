use crate::semiring::Semiring;

/// A `Semiring` with additive inverses.
pub trait Ring: Semiring {
    fn negate(self) -> Self;

    fn sub(self, other: Self) -> Self
    where
        Self: Sized,
    {
        self.add(other.negate())
    }
}

macro_rules! impl_signed_ring {
    ($($t:ty),*) => {
        $(
            impl Ring for $t {
                fn negate(self) -> Self { -self }
            }
        )*
    };
}

impl_signed_ring!(i8, i16, i32, i64, i128, f32, f64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i32_negate() {
        assert_eq!(5i32.negate(), -5);
    }

    #[test]
    fn i32_sub() {
        assert_eq!(10i32.sub(3), 7);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn additive_inverse(a in -100i16..100i16) {
            prop_assert_eq!(a.add(a.negate()), i16::zero());
        }

        #[test]
        fn sub_is_add_negate(a in -50i16..50i16, b in -50i16..50i16) {
            prop_assert_eq!(a.sub(b), a.add(b.negate()));
        }
    }
}
