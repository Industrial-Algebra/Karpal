#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(any(feature = "std", feature = "alloc"))]
pub mod coyoneda;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod yoneda;

#[cfg(any(feature = "std", feature = "alloc"))]
pub use coyoneda::{Coyoneda, CoyonedaF};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use yoneda::{Yoneda, YonedaF};
