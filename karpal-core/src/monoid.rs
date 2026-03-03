use crate::semigroup::Semigroup;

/// A `Semigroup` with an identity element.
pub trait Monoid: Semigroup {
    fn empty() -> Self;
}

macro_rules! impl_additive_monoid {
    ($($t:ty => $zero:expr),*) => {
        $(
            impl Monoid for $t {
                fn empty() -> Self {
                    $zero
                }
            }
        )*
    };
}

impl_additive_monoid!(
    i8 => 0, i16 => 0, i32 => 0, i64 => 0, i128 => 0,
    u8 => 0, u16 => 0, u32 => 0, u64 => 0, u128 => 0,
    f32 => 0.0, f64 => 0.0
);

#[cfg(any(feature = "std", feature = "alloc"))]
impl Monoid for String {
    fn empty() -> Self {
        String::new()
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T> Monoid for Vec<T> {
    fn empty() -> Self {
        Vec::new()
    }
}

impl<T: Semigroup> Monoid for Option<T> {
    fn empty() -> Self {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i32_empty() {
        assert_eq!(i32::empty(), 0);
    }

    #[test]
    fn string_empty() {
        assert_eq!(String::empty(), "");
    }

    #[test]
    fn vec_empty() {
        assert_eq!(Vec::<i32>::empty(), Vec::<i32>::new());
    }

    #[test]
    fn option_empty() {
        assert_eq!(Option::<i32>::empty(), None);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn i32_left_identity(a in any::<i32>()) {
            prop_assert_eq!(i32::empty().combine(a), a);
        }

        #[test]
        fn i32_right_identity(a in any::<i32>()) {
            prop_assert_eq!(a.combine(i32::empty()), a);
        }

        #[test]
        fn string_left_identity(a in "[a-z]{0,10}") {
            prop_assert_eq!(String::empty().combine(a.clone()), a);
        }

        #[test]
        fn string_right_identity(a in "[a-z]{0,10}") {
            prop_assert_eq!(a.clone().combine(String::empty()), a);
        }

        #[test]
        fn vec_left_identity(a in prop::collection::vec(any::<i32>(), 0..10)) {
            prop_assert_eq!(Vec::<i32>::empty().combine(a.clone()), a);
        }

        #[test]
        fn vec_right_identity(a in prop::collection::vec(any::<i32>(), 0..10)) {
            prop_assert_eq!(a.clone().combine(Vec::<i32>::empty()), a);
        }

        #[test]
        fn option_left_identity(a in any::<Option<u16>>()) {
            prop_assert_eq!(Option::<u16>::empty().combine(a.clone()), a);
        }

        #[test]
        fn option_right_identity(a in any::<Option<u16>>()) {
            prop_assert_eq!(a.clone().combine(Option::<u16>::empty()), a);
        }
    }
}
