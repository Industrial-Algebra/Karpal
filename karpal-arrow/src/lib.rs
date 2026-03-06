#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

pub mod arrow;
pub mod arrow_apply;
pub mod arrow_choice;
pub mod arrow_loop;
pub mod arrow_plus;
pub mod arrow_zero;
pub mod category;
pub mod semigroupoid;

#[cfg(any(feature = "std", feature = "alloc"))]
pub mod cokleisli;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod fn_arrow;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod kleisli;

pub use arrow::Arrow;
pub use arrow_apply::ArrowApply;
pub use arrow_choice::ArrowChoice;
pub use arrow_loop::ArrowLoop;
pub use arrow_plus::ArrowPlus;
pub use arrow_zero::ArrowZero;
pub use category::Category;
pub use semigroupoid::Semigroupoid;

#[cfg(any(feature = "std", feature = "alloc"))]
pub use cokleisli::CokleisliF;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use fn_arrow::FnA;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use kleisli::KleisliF;
