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

/// Type constructor for the Reader functor: `Of<T> = Box<dyn Fn(E) -> T>`.
///
/// Isomorphic to `TracedF<E>` in representation, but serves a different
/// semantic role: `ReaderF<E>` is the right adjoint of `EnvF<E>` in the
/// product/exponential adjunction (`EnvF<E> ⊣ ReaderF<E>`).
///
/// `ReaderF<E>` cannot implement the generic `Functor` trait because
/// `Box<dyn Fn>` requires `'static` bounds that the trait signature doesn't
/// allow. Following the Lan pattern, inherent methods with `'static` bounds
/// on the impl block provide the same functionality.
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct ReaderF<E>(PhantomData<E>);

#[cfg(any(feature = "std", feature = "alloc"))]
impl<E: 'static> HKT for ReaderF<E> {
    type Of<T> = Box<dyn Fn(E) -> T>;
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<E: Clone + 'static> ReaderF<E> {
    /// Functor `fmap` for Reader: post-compose a function.
    ///
    /// `fmap(f, reader) = |e| f(reader(e))`
    pub fn fmap<A: 'static, B: 'static>(
        fa: Box<dyn Fn(E) -> A>,
        f: impl Fn(A) -> B + 'static,
    ) -> Box<dyn Fn(E) -> B> {
        Box::new(move |e| f(fa(e)))
    }

    /// Applicative `pure` for Reader: ignore the environment, return a constant.
    ///
    /// `pure(a) = |_e| a`
    pub fn pure<A: Clone + 'static>(a: A) -> Box<dyn Fn(E) -> A> {
        Box::new(move |_| a.clone())
    }

    /// Monadic `chain` (bind) for Reader: Kleisli composition.
    ///
    /// `chain(reader, f) = |e| f(reader(e))(e)`
    pub fn chain<A: 'static, B: 'static>(
        fa: Box<dyn Fn(E) -> A>,
        f: impl Fn(A) -> Box<dyn Fn(E) -> B> + 'static,
    ) -> Box<dyn Fn(E) -> B> {
        Box::new(move |e: E| {
            let a = fa(e.clone());
            f(a)(e)
        })
    }

    /// Reader-specific: access the environment directly.
    ///
    /// `ask() = |e| e`
    pub fn ask() -> Box<dyn Fn(E) -> E> {
        Box::new(|e| e)
    }

    /// Reader-specific: modify the environment before running a reader.
    ///
    /// `local(f, reader) = |e| reader(f(e))`
    pub fn local<A: 'static>(
        f: impl Fn(E) -> E + 'static,
        reader: Box<dyn Fn(E) -> A>,
    ) -> Box<dyn Fn(E) -> A> {
        Box::new(move |e| reader(f(e)))
    }
}
