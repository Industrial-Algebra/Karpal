#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(all(not(feature = "std"), feature = "alloc"), feature(alloc))]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

pub mod property;
pub mod proven;
pub mod rewrite;

#[cfg(any(feature = "std", feature = "alloc"))]
pub mod refinement;

#[cfg(any(feature = "std", feature = "alloc"))]
pub mod law_check;

pub use property::*;
pub use proven::Proven;
pub use rewrite::{Justifies, Rewrite};

#[cfg(any(feature = "std", feature = "alloc"))]
pub use refinement::{NonEmpty, Positive};
