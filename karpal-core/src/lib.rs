#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(all(not(feature = "std"), feature = "alloc"), feature(alloc))]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

pub mod functor;
pub mod hkt;
pub mod monoid;
pub mod semigroup;

pub use functor::Functor;
pub use hkt::HKT;
pub use monoid::Monoid;
pub use semigroup::Semigroup;
