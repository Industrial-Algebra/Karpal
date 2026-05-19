#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "std", feature = "alloc"))]
extern crate alloc;

pub mod braiding;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod diagram;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod render;
pub mod symmetry;
pub mod tensor;

pub use braiding::Braiding;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use diagram::{Diagram, DiagramKind};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use render::{SvgRenderer, TextRenderer};
pub use symmetry::Symmetry;
pub use tensor::Tensor;
