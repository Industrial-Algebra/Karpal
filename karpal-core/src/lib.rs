#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(all(not(feature = "std"), feature = "alloc"), feature(alloc))]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

pub mod alt;
pub mod alternative;
pub mod applicative;
pub mod apply;
pub mod bifunctor;
pub mod chain;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod contravariant;
pub mod foldable;
pub mod functor;
pub mod functor_filter;
pub mod hkt;
#[macro_use]
pub mod macros;
pub mod monad;
pub mod monoid;
pub mod natural;
pub mod plus;
pub mod selective;
pub mod semigroup;
pub mod traversable;

pub use alt::Alt;
pub use alternative::Alternative;
pub use applicative::Applicative;
pub use apply::Apply;
pub use bifunctor::Bifunctor;
pub use chain::Chain;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use contravariant::{Contravariant, PredicateF};
pub use foldable::Foldable;
pub use functor::Functor;
pub use functor_filter::FunctorFilter;
pub use hkt::{HKT, HKT2};
pub use monad::Monad;
pub use monoid::Monoid;
pub use natural::NaturalTransformation;
pub use plus::Plus;
pub use selective::Selective;
pub use semigroup::Semigroup;
pub use traversable::Traversable;
