#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod choice;
#[cfg(feature = "alloc")]
pub mod fn_profunctor;
#[cfg(feature = "alloc")]
pub mod forget;
pub mod profunctor;
pub mod strong;
pub mod tagged;
#[cfg(feature = "alloc")]
pub mod traversing;

pub use choice::Choice;
#[cfg(feature = "alloc")]
pub use fn_profunctor::FnP;
#[cfg(feature = "alloc")]
pub use forget::ForgetF;
pub use profunctor::{HKT2, Profunctor};
pub use strong::Strong;
pub use tagged::TaggedF;
#[cfg(feature = "alloc")]
pub use traversing::Traversing;
