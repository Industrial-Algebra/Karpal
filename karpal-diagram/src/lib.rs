#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "std", feature = "alloc"))]
extern crate alloc;

pub mod braiding;
pub mod coherence;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod diagram;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod render;
pub mod symmetry;
pub mod tensor;
pub mod trace;

pub use braiding::Braiding;
pub use coherence::{
    ByNormalization, ByYanking, HexagonIdentity, PentagonIdentity, TriangleIdentity,
};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use coherence::{
    CoherenceCertificate, coherence_certificates, equivalent_proved, prove_yanking, verify_hexagon,
    verify_pentagon, verify_triangle,
};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use diagram::{Diagram, DiagramKind, NormalizationRule, NormalizationTrace};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use render::{SvgRenderer, TextRenderer};
pub use symmetry::Symmetry;
pub use tensor::Tensor;
pub use trace::Trace;
