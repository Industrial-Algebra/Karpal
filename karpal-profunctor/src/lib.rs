#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod choice;
#[cfg(feature = "alloc")]
pub mod fn_profunctor;
pub mod profunctor;
pub mod strong;

pub use choice::Choice;
#[cfg(feature = "alloc")]
pub use fn_profunctor::FnP;
pub use profunctor::{HKT2, Profunctor};
pub use strong::Strong;
