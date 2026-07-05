// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

pub mod abelian;
pub mod bounded_lattice;
pub mod field;
pub mod group;
pub mod heyting;
pub mod lattice;
pub mod module;
pub mod ring;
pub mod semiring;
pub mod vector_space;

pub use abelian::AbelianGroup;
pub use bounded_lattice::BoundedLattice;
pub use field::Field;
pub use group::Group;
pub use heyting::HeytingAlgebra;
pub use lattice::Lattice;
pub use module::Module;
pub use ring::Ring;
pub use semiring::Semiring;
pub use vector_space::VectorSpace;
