#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(all(not(feature = "std"), feature = "alloc"), feature(alloc))]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

pub mod adjunction;
pub mod alt;
pub mod alternative;
pub mod applicative;
pub mod apply;
pub mod bifunctor;
pub mod chain;
pub mod coend;
pub mod comonad;
pub mod comonad_env;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod comonad_store;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod comonad_traced;
pub mod compose;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod conclude;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod contravariant;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod decide;
pub mod dinatural;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod divide;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod divisible;
pub mod end;
pub mod extend;
pub mod foldable;
pub mod functor;
pub mod functor_filter;
pub mod hkt;
pub mod invariant;
#[macro_use]
pub mod macros;
pub mod monad;
pub mod monoid;
pub mod natural;
pub mod newtype;
pub mod plus;
pub mod selective;
pub mod semigroup;
pub mod traversable;

pub use adjunction::Adjunction;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use adjunction::ContAdj;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use adjunction::ContF;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use adjunction::ContravariantAdjunction;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use adjunction::CurryAdj;
pub use adjunction::IdentityAdj;
pub use adjunction::ProfunctorAdjunction;
pub use adjunction::ProfunctorFunctor;
pub use adjunction::ProfunctorIdentityAdj;
pub use adjunction::ProfunctorIdentityF;
pub use alt::Alt;
pub use alternative::Alternative;
pub use applicative::Applicative;
pub use apply::Apply;
pub use bifunctor::Bifunctor;
pub use chain::Chain;
pub use coend::Coend;
pub use comonad::Comonad;
pub use comonad_env::ComonadEnv;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use comonad_store::ComonadStore;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use comonad_traced::ComonadTraced;
pub use compose::ComposeF;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use conclude::Conclude;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use contravariant::{Contravariant, PredicateF};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use decide::Decide;
pub use dinatural::DinaturalTransformation;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use divide::Divide;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use divisible::Divisible;
pub use end::End;
pub use extend::Extend;
pub use foldable::Foldable;
pub use functor::Functor;
pub use functor_filter::FunctorFilter;
pub use hkt::{HKT, HKT2};
pub use invariant::Invariant;
pub use monad::Monad;
pub use monoid::Monoid;
pub use natural::NaturalTransformation;
pub use newtype::{First, Last, Max, Min, Product, Sum};
pub use plus::Plus;
pub use selective::Selective;
pub use semigroup::Semigroup;
pub use traversable::Traversable;
