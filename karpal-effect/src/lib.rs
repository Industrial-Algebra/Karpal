#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(all(not(feature = "std"), feature = "alloc"), feature(alloc))]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(any(feature = "std", feature = "alloc"))]
pub mod classes;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod except_t;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod reader_t;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod state_t;
pub mod trans;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod writer_t;

pub use trans::MonadTrans;

#[cfg(any(feature = "std", feature = "alloc"))]
pub use classes::{ApplicativeSt, ChainSt, FunctorSt};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use except_t::ExceptTF;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use reader_t::ReaderTF;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use state_t::StateTF;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use writer_t::WriterTF;
