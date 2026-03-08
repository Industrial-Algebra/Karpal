//! Newtype wrappers for selecting alternative `Semigroup`/`Monoid` instances.
//!
//! The default `Semigroup` for numeric types uses addition. These newtypes
//! let you choose a different combining strategy.

use crate::monoid::Monoid;
use crate::semigroup::Semigroup;

/// Newtype selecting additive `Semigroup` (uses `+`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sum<T>(pub T);

/// Newtype selecting multiplicative `Semigroup` (uses `*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Product<T>(pub T);

/// Newtype selecting minimum `Semigroup` (uses `core::cmp::min`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Min<T>(pub T);

/// Newtype selecting maximum `Semigroup` (uses `core::cmp::max`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Max<T>(pub T);

/// Newtype selecting first-wins `Semigroup`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct First<T>(pub T);

/// Newtype selecting last-wins `Semigroup`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Last<T>(pub T);

// --- Sum ---

impl<T: core::ops::Add<Output = T>> Semigroup for Sum<T> {
    fn combine(self, other: Self) -> Self {
        Sum(self.0 + other.0)
    }
}

macro_rules! impl_sum_monoid {
    ($($t:ty => $zero:expr),*) => {
        $(
            impl Monoid for Sum<$t> {
                fn empty() -> Self { Sum($zero) }
            }
        )*
    };
}

impl_sum_monoid!(
    i8 => 0, i16 => 0, i32 => 0, i64 => 0, i128 => 0,
    u8 => 0, u16 => 0, u32 => 0, u64 => 0, u128 => 0,
    f32 => 0.0, f64 => 0.0
);

// --- Product ---

impl<T: core::ops::Mul<Output = T>> Semigroup for Product<T> {
    fn combine(self, other: Self) -> Self {
        Product(self.0 * other.0)
    }
}

macro_rules! impl_product_monoid {
    ($($t:ty => $one:expr),*) => {
        $(
            impl Monoid for Product<$t> {
                fn empty() -> Self { Product($one) }
            }
        )*
    };
}

impl_product_monoid!(
    i8 => 1, i16 => 1, i32 => 1, i64 => 1, i128 => 1,
    u8 => 1, u16 => 1, u32 => 1, u64 => 1, u128 => 1,
    f32 => 1.0, f64 => 1.0
);

// --- Min ---

impl<T: Ord> Semigroup for Min<T> {
    fn combine(self, other: Self) -> Self {
        Min(core::cmp::min(self.0, other.0))
    }
}

macro_rules! impl_min_monoid {
    ($($t:ty),*) => {
        $(
            impl Monoid for Min<$t> {
                fn empty() -> Self { Min(<$t>::MAX) }
            }
        )*
    };
}

impl_min_monoid!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

// --- Max ---

impl<T: Ord> Semigroup for Max<T> {
    fn combine(self, other: Self) -> Self {
        Max(core::cmp::max(self.0, other.0))
    }
}

macro_rules! impl_max_monoid {
    ($($t:ty),*) => {
        $(
            impl Monoid for Max<$t> {
                fn empty() -> Self { Max(<$t>::MIN) }
            }
        )*
    };
}

impl_max_monoid!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

// --- First (Option only — picks first Some) ---

impl<T> Semigroup for First<Option<T>> {
    fn combine(self, other: Self) -> Self {
        match self.0 {
            Some(_) => self,
            None => other,
        }
    }
}

impl<T> Monoid for First<Option<T>> {
    fn empty() -> Self {
        First(None)
    }
}

// --- Last (Option only — picks last Some) ---

impl<T> Semigroup for Last<Option<T>> {
    fn combine(self, other: Self) -> Self {
        match other.0 {
            Some(_) => other,
            None => self,
        }
    }
}

impl<T> Monoid for Last<Option<T>> {
    fn empty() -> Self {
        Last(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum_combine() {
        assert_eq!(Sum(3i32).combine(Sum(4)), Sum(7));
    }

    #[test]
    fn sum_monoid_identity() {
        assert_eq!(Sum::<i32>::empty().combine(Sum(5)), Sum(5));
        assert_eq!(Sum(5i32).combine(Sum::empty()), Sum(5));
    }

    #[test]
    fn product_combine() {
        assert_eq!(Product(3i32).combine(Product(4)), Product(12));
    }

    #[test]
    fn product_monoid_identity() {
        assert_eq!(Product::<i32>::empty().combine(Product(5)), Product(5));
        assert_eq!(Product(5i32).combine(Product::empty()), Product(5));
    }

    #[test]
    fn min_combine() {
        assert_eq!(Min(3i32).combine(Min(7)), Min(3));
        assert_eq!(Min(10u8).combine(Min(2)), Min(2));
    }

    #[test]
    fn min_monoid_identity() {
        assert_eq!(Min::<i32>::empty().combine(Min(5)), Min(5));
        assert_eq!(Min(5i32).combine(Min::empty()), Min(5));
    }

    #[test]
    fn max_combine() {
        assert_eq!(Max(3i32).combine(Max(7)), Max(7));
        assert_eq!(Max(10u8).combine(Max(2)), Max(10));
    }

    #[test]
    fn max_monoid_identity() {
        assert_eq!(Max::<i32>::empty().combine(Max(5)), Max(5));
        assert_eq!(Max(5i32).combine(Max::empty()), Max(5));
    }

    #[test]
    fn first_option_combine() {
        assert_eq!(First(Some(1i32)).combine(First(Some(2))), First(Some(1)));
        assert_eq!(First(None::<i32>).combine(First(Some(2))), First(Some(2)));
        assert_eq!(First(Some(1i32)).combine(First(None)), First(Some(1)));
    }

    #[test]
    fn first_option_monoid() {
        assert_eq!(
            First::<Option<i32>>::empty().combine(First(Some(5))),
            First(Some(5))
        );
        assert_eq!(First(Some(5i32)).combine(First::empty()), First(Some(5)));
    }

    #[test]
    fn last_option_combine() {
        assert_eq!(Last(Some(1i32)).combine(Last(Some(2))), Last(Some(2)));
        assert_eq!(Last(None::<i32>).combine(Last(Some(2))), Last(Some(2)));
        assert_eq!(Last(Some(1i32)).combine(Last(None)), Last(Some(1)));
    }

    #[test]
    fn last_option_monoid() {
        assert_eq!(
            Last::<Option<i32>>::empty().combine(Last(Some(5))),
            Last(Some(5))
        );
        assert_eq!(Last(Some(5i32)).combine(Last::empty()), Last(Some(5)));
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn sum_associativity(a in -100i16..100i16, b in -100i16..100i16, c in -100i16..100i16) {
            let left = Sum(a).combine(Sum(b)).combine(Sum(c));
            let right = Sum(a).combine(Sum(b).combine(Sum(c)));
            prop_assert_eq!(left, right);
        }

        #[test]
        fn product_associativity(a in -10i16..10i16, b in -10i16..10i16, c in -10i16..10i16) {
            let left = Product(a).combine(Product(b)).combine(Product(c));
            let right = Product(a).combine(Product(b).combine(Product(c)));
            prop_assert_eq!(left, right);
        }

        #[test]
        fn min_associativity(a in any::<i32>(), b in any::<i32>(), c in any::<i32>()) {
            let left = Min(a).combine(Min(b)).combine(Min(c));
            let right = Min(a).combine(Min(b).combine(Min(c)));
            prop_assert_eq!(left, right);
        }

        #[test]
        fn max_associativity(a in any::<i32>(), b in any::<i32>(), c in any::<i32>()) {
            let left = Max(a).combine(Max(b)).combine(Max(c));
            let right = Max(a).combine(Max(b).combine(Max(c)));
            prop_assert_eq!(left, right);
        }

        #[test]
        fn first_option_associativity(
            a in proptest::option::of(any::<i32>()),
            b in proptest::option::of(any::<i32>()),
            c in proptest::option::of(any::<i32>())
        ) {
            let left = First(a.clone()).combine(First(b.clone())).combine(First(c.clone()));
            let right = First(a).combine(First(b).combine(First(c)));
            prop_assert_eq!(left, right);
        }

        #[test]
        fn last_option_associativity(
            a in proptest::option::of(any::<i32>()),
            b in proptest::option::of(any::<i32>()),
            c in proptest::option::of(any::<i32>())
        ) {
            let left = Last(a.clone()).combine(Last(b.clone())).combine(Last(c.clone()));
            let right = Last(a).combine(Last(b).combine(Last(c)));
            prop_assert_eq!(left, right);
        }

        // Monoid identity laws
        #[test]
        fn sum_left_identity(a in -100i16..100i16) {
            prop_assert_eq!(Sum::<i16>::empty().combine(Sum(a)), Sum(a));
        }

        #[test]
        fn sum_right_identity(a in -100i16..100i16) {
            prop_assert_eq!(Sum(a).combine(Sum::empty()), Sum(a));
        }

        #[test]
        fn product_left_identity(a in -100i16..100i16) {
            prop_assert_eq!(Product::<i16>::empty().combine(Product(a)), Product(a));
        }

        #[test]
        fn product_right_identity(a in -100i16..100i16) {
            prop_assert_eq!(Product(a).combine(Product::empty()), Product(a));
        }

        #[test]
        fn min_left_identity(a in any::<i32>()) {
            prop_assert_eq!(Min::<i32>::empty().combine(Min(a)), Min(a));
        }

        #[test]
        fn min_right_identity(a in any::<i32>()) {
            prop_assert_eq!(Min(a).combine(Min::empty()), Min(a));
        }

        #[test]
        fn max_left_identity(a in any::<i32>()) {
            prop_assert_eq!(Max::<i32>::empty().combine(Max(a)), Max(a));
        }

        #[test]
        fn max_right_identity(a in any::<i32>()) {
            prop_assert_eq!(Max(a).combine(Max::empty()), Max(a));
        }
    }
}
