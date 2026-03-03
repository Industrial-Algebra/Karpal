use core::marker::PhantomData;

/// Higher-Kinded Type encoding via GATs.
///
/// A type implementing `HKT` acts as a type-level function:
/// given a type `T`, it produces `Self::Of<T>`.
pub trait HKT {
    type Of<T>;
}

/// Type constructor for `Option<T>`.
pub struct OptionF;

impl HKT for OptionF {
    type Of<T> = Option<T>;
}

/// Type constructor for `Result<T, E>` (fixed error type `E`).
pub struct ResultF<E> {
    _marker: PhantomData<E>,
}

impl<E> HKT for ResultF<E> {
    type Of<T> = Result<T, E>;
}

/// Type constructor for `Vec<T>`.
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct VecF;

#[cfg(any(feature = "std", feature = "alloc"))]
impl HKT for VecF {
    #[cfg(feature = "std")]
    type Of<T> = Vec<T>;

    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    type Of<T> = alloc::vec::Vec<T>;
}
