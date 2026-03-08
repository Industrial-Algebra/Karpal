/// A type with two operations (add, mul), where add forms a commutative monoid,
/// mul forms a monoid, mul distributes over add, and zero annihilates under mul.
pub trait Semiring: Sized + Clone + PartialEq {
    fn zero() -> Self;
    fn one() -> Self;
    fn add(self, other: Self) -> Self;
    fn mul(self, other: Self) -> Self;
}

macro_rules! impl_numeric_semiring {
    ($($t:ty => ($zero:expr, $one:expr)),*) => {
        $(
            impl Semiring for $t {
                fn zero() -> Self { $zero }
                fn one() -> Self { $one }
                fn add(self, other: Self) -> Self { self + other }
                fn mul(self, other: Self) -> Self { self * other }
            }
        )*
    };
}

impl_numeric_semiring!(
    i8 => (0, 1), i16 => (0, 1), i32 => (0, 1), i64 => (0, 1), i128 => (0, 1),
    u8 => (0, 1), u16 => (0, 1), u32 => (0, 1), u64 => (0, 1), u128 => (0, 1),
    f32 => (0.0, 1.0), f64 => (0.0, 1.0)
);

impl Semiring for bool {
    fn zero() -> Self {
        false
    }
    fn one() -> Self {
        true
    }
    fn add(self, other: Self) -> Self {
        self || other
    }
    fn mul(self, other: Self) -> Self {
        self && other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i32_semiring() {
        assert_eq!(i32::zero(), 0);
        assert_eq!(i32::one(), 1);
        assert_eq!(3i32.add(4), 7);
        assert_eq!(3i32.mul(4), 12);
    }

    #[test]
    fn bool_semiring() {
        assert_eq!(bool::zero(), false);
        assert_eq!(bool::one(), true);
        assert_eq!(false.add(true), true); // OR
        assert_eq!(true.mul(false), false); // AND
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Additive commutative monoid
        #[test]
        fn add_associativity(a in -30i16..30i16, b in -30i16..30i16, c in -30i16..30i16) {
            prop_assert_eq!(a.add(b).add(c), a.add(b.add(c)));
        }

        #[test]
        fn add_commutativity(a in -100i16..100i16, b in -100i16..100i16) {
            prop_assert_eq!(a.add(b), b.add(a));
        }

        #[test]
        fn add_identity(a in any::<i16>()) {
            prop_assert_eq!(i16::zero().add(a), a);
            prop_assert_eq!(a.add(i16::zero()), a);
        }

        // Multiplicative monoid
        #[test]
        fn mul_associativity(a in -10i16..10i16, b in -10i16..10i16, c in -10i16..10i16) {
            prop_assert_eq!(a.mul(b).mul(c), a.mul(b.mul(c)));
        }

        #[test]
        fn mul_identity(a in any::<i16>()) {
            prop_assert_eq!(i16::one().mul(a), a);
            prop_assert_eq!(a.mul(i16::one()), a);
        }

        // Distribution
        #[test]
        fn left_distribution(a in -10i16..10i16, b in -10i16..10i16, c in -10i16..10i16) {
            prop_assert_eq!(a.mul(b.add(c)), a.mul(b).add(a.mul(c)));
        }

        #[test]
        fn right_distribution(a in -10i16..10i16, b in -10i16..10i16, c in -10i16..10i16) {
            prop_assert_eq!(a.add(b).mul(c), a.mul(c).add(b.mul(c)));
        }

        // Annihilation
        #[test]
        fn zero_annihilation(a in any::<i16>()) {
            prop_assert_eq!(i16::zero().mul(a), i16::zero());
            prop_assert_eq!(a.mul(i16::zero()), i16::zero());
        }
    }
}
