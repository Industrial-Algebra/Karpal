#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use karpal_core::Semigroup;

/// A `Vec<T>` guaranteed to contain at least one element.
///
/// Unlike `NonEmptyVec<T>` in karpal-core (which has a structurally
/// different representation with separate `head` and `tail` fields),
/// `NonEmpty<Vec<T>>` wraps a standard `Vec<T>` with a refinement
/// invariant. Construction is only possible via `try_new` (which checks)
/// or `from_parts` (which requires at least one element).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonEmpty<C> {
    inner: C,
}

impl<T> NonEmpty<Vec<T>> {
    /// Attempt to construct a `NonEmpty<Vec<T>>` from a `Vec<T>`.
    /// Returns `None` if the vector is empty.
    pub fn try_new(v: Vec<T>) -> Option<Self> {
        if v.is_empty() {
            None
        } else {
            Some(NonEmpty { inner: v })
        }
    }

    /// Construct from a head element and remaining tail.
    pub fn from_parts(head: T, tail: Vec<T>) -> Self {
        let mut v = tail;
        v.insert(0, head);
        NonEmpty { inner: v }
    }

    /// Construct a single-element vector.
    pub fn singleton(value: T) -> Self {
        NonEmpty {
            inner: [value].into(),
        }
    }

    /// The first element (always exists).
    pub fn head(&self) -> &T {
        // Safety: invariant guarantees non-empty
        &self.inner[0]
    }

    /// Number of elements (always >= 1).
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Always returns `false` — a `NonEmpty` is never empty by construction.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Push a new element.
    pub fn push(&mut self, value: T) {
        self.inner.push(value);
    }

    /// Access the underlying slice.
    pub fn as_slice(&self) -> &[T] {
        &self.inner
    }

    /// Convert into the underlying `Vec<T>`, discarding the proof.
    pub fn into_vec(self) -> Vec<T> {
        self.inner
    }

    /// Iterate over references.
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.inner.iter()
    }

    /// Map a function over all elements, preserving non-emptiness.
    pub fn map<U>(self, f: impl FnMut(T) -> U) -> NonEmpty<Vec<U>> {
        NonEmpty {
            inner: self.inner.into_iter().map(f).collect(),
        }
    }
}

impl<T: Semigroup + Clone> Semigroup for NonEmpty<Vec<T>> {
    fn combine(mut self, other: Self) -> Self {
        self.inner.extend(other.inner);
        self
    }
}

/// A numeric value guaranteed to be strictly positive (> 0).
///
/// Useful for operations that require nonzero values, such as
/// `Field::reciprocal`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Positive<T> {
    value: T,
}

macro_rules! impl_positive_float {
    ($($t:ty),*) => {
        $(
            impl Positive<$t> {
                /// Attempt to construct a `Positive` from a value.
                /// Returns `None` if the value is not strictly positive.
                pub fn try_new(v: $t) -> Option<Self> {
                    if v > 0.0 && v.is_finite() {
                        Some(Positive { value: v })
                    } else {
                        None
                    }
                }

                /// Get the inner value.
                pub fn get(self) -> $t {
                    self.value
                }

                /// Safe reciprocal: always valid for positive values.
                pub fn reciprocal(self) -> Self {
                    Positive { value: 1.0 / self.value }
                }
            }
        )*
    };
}

impl_positive_float!(f32, f64);

macro_rules! impl_positive_int {
    ($($t:ty),*) => {
        $(
            impl Positive<$t> {
                /// Attempt to construct a `Positive` from a value.
                /// Returns `None` if the value is zero or negative.
                pub fn try_new(v: $t) -> Option<Self> {
                    if v > 0 {
                        Some(Positive { value: v })
                    } else {
                        None
                    }
                }

                /// Get the inner value.
                pub fn get(self) -> $t {
                    self.value
                }
            }
        )*
    };
}

impl_positive_int!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

macro_rules! impl_positive_unsigned {
    ($($t:ty),*) => {
        $(
            impl Positive<$t> {
                /// Construct from a nonzero unsigned value.
                /// Returns `None` if zero.
                pub fn from_nonzero(v: $t) -> Option<Self> {
                    if v > 0 {
                        Some(Positive { value: v })
                    } else {
                        None
                    }
                }
            }
        )*
    };
}

impl_positive_unsigned!(u8, u16, u32, u64, u128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_empty_try_new() {
        assert!(NonEmpty::try_new(Vec::<i32>::new()).is_none());
        let ne = NonEmpty::try_new(vec![1, 2, 3]).unwrap();
        assert_eq!(*ne.head(), 1);
        assert_eq!(ne.len(), 3);
    }

    #[test]
    fn non_empty_from_parts() {
        let ne = NonEmpty::from_parts(10, vec![20, 30]);
        assert_eq!(*ne.head(), 10);
        assert_eq!(ne.as_slice(), &[10, 20, 30]);
    }

    #[test]
    fn non_empty_singleton() {
        let ne = NonEmpty::singleton(42);
        assert_eq!(*ne.head(), 42);
        assert_eq!(ne.len(), 1);
    }

    #[test]
    fn non_empty_push() {
        let mut ne = NonEmpty::singleton(1);
        ne.push(2);
        assert_eq!(ne.len(), 2);
        assert_eq!(ne.as_slice(), &[1, 2]);
    }

    #[test]
    fn non_empty_map() {
        let ne = NonEmpty::from_parts(1, vec![2, 3]);
        let doubled = ne.map(|x| x * 2);
        assert_eq!(doubled.as_slice(), &[2, 4, 6]);
    }

    #[test]
    fn non_empty_into_vec() {
        let ne = NonEmpty::from_parts(1, vec![2]);
        assert_eq!(ne.into_vec(), vec![1, 2]);
    }

    #[test]
    fn non_empty_iter() {
        let ne = NonEmpty::from_parts(1, vec![2, 3]);
        let sum: i32 = ne.iter().sum();
        assert_eq!(sum, 6);
    }

    #[test]
    fn non_empty_semigroup() {
        let a = NonEmpty::from_parts(1, vec![2]);
        let b = NonEmpty::from_parts(3, vec![4]);
        let c = a.combine(b);
        assert_eq!(c.as_slice(), &[1, 2, 3, 4]);
    }

    #[test]
    fn positive_f64() {
        assert!(Positive::<f64>::try_new(0.0).is_none());
        assert!(Positive::<f64>::try_new(-1.0).is_none());
        assert!(Positive::<f64>::try_new(f64::NAN).is_none());
        assert!(Positive::<f64>::try_new(f64::INFINITY).is_none());

        let p = Positive::<f64>::try_new(4.0).unwrap();
        assert_eq!(p.get(), 4.0);

        let r = p.reciprocal();
        assert!((r.get() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn positive_f32() {
        let p = Positive::<f32>::try_new(2.0).unwrap();
        assert!((p.reciprocal().get() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn positive_i32() {
        assert!(Positive::<i32>::try_new(0).is_none());
        assert!(Positive::<i32>::try_new(-5).is_none());
        let p = Positive::<i32>::try_new(7).unwrap();
        assert_eq!(p.get(), 7);
    }

    #[test]
    fn positive_u32() {
        assert!(Positive::<u32>::try_new(0).is_none());
        let p = Positive::<u32>::try_new(10).unwrap();
        assert_eq!(p.get(), 10);
    }

    #[test]
    fn positive_u32_from_nonzero() {
        assert!(Positive::<u32>::from_nonzero(0).is_none());
        let p = Positive::<u32>::from_nonzero(5).unwrap();
        assert_eq!(p.get(), 5);
    }
}
