/// A type with an associative binary operation.
pub trait Semigroup {
    fn combine(self, other: Self) -> Self;
}

macro_rules! impl_additive_semigroup {
    ($($t:ty),*) => {
        $(
            impl Semigroup for $t {
                fn combine(self, other: Self) -> Self {
                    self + other
                }
            }
        )*
    };
}

impl_additive_semigroup!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64);

#[cfg(any(feature = "std", feature = "alloc"))]
impl Semigroup for String {
    fn combine(mut self, other: Self) -> Self {
        self.push_str(&other);
        self
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T> Semigroup for Vec<T> {
    fn combine(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}

impl<T: Semigroup> Semigroup for Option<T> {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Some(a), Some(b)) => Some(a.combine(b)),
            (a @ Some(_), None) => a,
            (None, b) => b,
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T> Semigroup for crate::hkt::NonEmptyVec<T> {
    fn combine(mut self, other: Self) -> Self {
        self.tail.push(other.head);
        self.tail.extend(other.tail);
        self
    }
}

impl<A: Semigroup, B: Semigroup> Semigroup for (A, B) {
    fn combine(self, other: Self) -> Self {
        (self.0.combine(other.0), self.1.combine(other.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i32_combine() {
        assert_eq!(3i32.combine(4), 7);
    }

    #[test]
    fn string_combine() {
        assert_eq!(
            "hello ".to_string().combine("world".to_string()),
            "hello world"
        );
    }

    #[test]
    fn vec_combine() {
        assert_eq!(vec![1, 2].combine(vec![3, 4]), vec![1, 2, 3, 4]);
    }

    #[test]
    fn option_combine() {
        assert_eq!(Some(3i32).combine(Some(4)), Some(7));
        assert_eq!(Some(3i32).combine(None), Some(3));
        assert_eq!(None::<i32>.combine(Some(4)), Some(4));
        assert_eq!(None::<i32>.combine(None), None);
    }
}

#[cfg(test)]
mod law_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn i32_associativity(a in any::<i32>(), b in any::<i32>(), c in any::<i32>()) {
            // Use wrapping to avoid overflow panics
            let left = (a.wrapping_add(b)).wrapping_add(c);
            let right = a.wrapping_add(b.wrapping_add(c));
            prop_assert_eq!(left, right);
        }

        #[test]
        fn string_associativity(
            a in "[a-z]{0,10}",
            b in "[a-z]{0,10}",
            c in "[a-z]{0,10}"
        ) {
            let left = a.clone().combine(b.clone()).combine(c.clone());
            let right = a.combine(b.combine(c));
            prop_assert_eq!(left, right);
        }

        #[test]
        fn vec_associativity(
            a in prop::collection::vec(any::<i32>(), 0..5),
            b in prop::collection::vec(any::<i32>(), 0..5),
            c in prop::collection::vec(any::<i32>(), 0..5)
        ) {
            let left = a.clone().combine(b.clone()).combine(c.clone());
            let right = a.combine(b.combine(c));
            prop_assert_eq!(left, right);
        }

        #[test]
        fn option_associativity(
            a in proptest::option::of(0u16..100),
            b in proptest::option::of(0u16..100),
            c in proptest::option::of(0u16..100)
        ) {
            let left = a.clone().combine(b.clone()).combine(c.clone());
            let right = a.combine(b.combine(c));
            prop_assert_eq!(left, right);
        }
    }
}
