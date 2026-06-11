#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

pub mod bicategory;
pub mod enriched;
pub mod ffunctor;
pub mod two_category;

pub use bicategory::Bicategory;
pub use enriched::EnrichedCategory;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use enriched::SetCategory;
pub use ffunctor::{FFunctor, FMonad, IdentityFFunctor};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use two_category::Cat;
pub use two_category::TwoCategory;
