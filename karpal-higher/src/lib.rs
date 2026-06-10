#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

pub mod two_category;

pub use two_category::TwoCategory;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use two_category::Cat;
