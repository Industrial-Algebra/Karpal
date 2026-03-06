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

/// Two-parameter type constructor (HKT for bifunctors / profunctors).
pub trait HKT2 {
    type P<A, B>;
}

/// Type constructor for `Result<A, B>` as a bifunctor (both parameters vary).
pub struct ResultBF;

impl HKT2 for ResultBF {
    type P<A, B> = Result<B, A>;
}

/// Type constructor for `(A, B)` as a bifunctor.
pub struct TupleF;

impl HKT2 for TupleF {
    type P<A, B> = (A, B);
}

/// Type constructor for the identity functor: `Of<T> = T`.
pub struct IdentityF;

impl HKT for IdentityF {
    type Of<T> = T;
}

/// A non-empty vector: guaranteed to have at least one element.
#[cfg(any(feature = "std", feature = "alloc"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmptyVec<T> {
    pub head: T,
    pub tail: Vec<T>,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T> NonEmptyVec<T> {
    pub fn new(head: T, tail: Vec<T>) -> Self {
        Self { head, tail }
    }

    pub fn singleton(value: T) -> Self {
        Self {
            head: value,
            tail: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        1 + self.tail.len()
    }

    pub fn is_empty(&self) -> bool {
        false // always at least one element
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        core::iter::once(&self.head).chain(self.tail.iter())
    }

    /// Collect all suffixes as a NonEmptyVec of NonEmptyVecs.
    pub fn tails(&self) -> NonEmptyVec<NonEmptyVec<T>>
    where
        T: Clone,
    {
        let mut result_tail = Vec::new();
        for i in 1..self.len() {
            result_tail.push(NonEmptyVec::new(
                self.tail[i - 1].clone(),
                self.tail[i..].to_vec(),
            ));
        }
        NonEmptyVec::new(self.clone(), result_tail)
    }
}

/// Type constructor for `NonEmptyVec<T>`.
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct NonEmptyVecF;

#[cfg(any(feature = "std", feature = "alloc"))]
impl HKT for NonEmptyVecF {
    type Of<T> = NonEmptyVec<T>;
}

/// Type constructor for the Env comonad: `Of<T> = (E, T)`.
pub struct EnvF<E>(PhantomData<E>);

impl<E> HKT for EnvF<E> {
    type Of<T> = (E, T);
}

/// Type constructor for the Store comonad: `Of<T> = (Box<dyn Fn(S) -> T>, S)`.
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct StoreF<S>(PhantomData<S>);

#[cfg(any(feature = "std", feature = "alloc"))]
impl<S: 'static> HKT for StoreF<S> {
    type Of<T> = (Box<dyn Fn(S) -> T>, S);
}

/// Type constructor for the Traced comonad: `Of<T> = Box<dyn Fn(M) -> T>`.
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct TracedF<M>(PhantomData<M>);

#[cfg(any(feature = "std", feature = "alloc"))]
impl<M: 'static> HKT for TracedF<M> {
    type Of<T> = Box<dyn Fn(M) -> T>;
}
