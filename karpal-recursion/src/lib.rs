#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(any(feature = "std", feature = "alloc"))]
pub mod algebra;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod either;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod fix;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod nu;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod schemes;

#[cfg(any(feature = "std", feature = "alloc"))]
pub use either::Either;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use fix::{Fix, FixF, Mu};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use nu::Nu;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use schemes::{ana, apo, cata, chrono, futu, histo, hylo, para, zygo};
