// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use amari_enumerative::SchubertClass;

/// A Schubert class in a Grassmannian, used as a type-level marker.
///
/// Wraps `amari_enumerative::SchubertClass` with additional metadata
/// and validation. The partition must fit in the `k × (n-k)` box.
#[derive(Debug, Clone)]
pub struct SchubertType {
    inner: SchubertClass,
}

impl SchubertType {
    /// Create a new Schubert type from a partition and Grassmannian dimension.
    ///
    /// Returns `Err` if any partition entry exceeds `n - k`.
    pub fn new(partition: Vec<usize>, grassmannian_dim: (usize, usize)) -> Result<Self, String> {
        SchubertClass::new(partition, grassmannian_dim)
            .map(|sc| Self { inner: sc })
            .map_err(|e| e.to_string())
    }

    /// The partition indexing this Schubert class.
    pub fn partition(&self) -> &[usize] {
        &self.inner.partition
    }

    /// The underlying Grassmannian `(k, n)`.
    pub fn grassmannian_dim(&self) -> (usize, usize) {
        self.inner.grassmannian_dim
    }

    /// Codimension of this Schubert class (sum of partition entries).
    pub fn codimension(&self) -> usize {
        self.inner.codimension()
    }

    /// Dimension of this Schubert class in the Grassmannian.
    pub fn dimension(&self) -> usize {
        self.inner.dimension()
    }

    /// True for the point class (empty partition, codim = 0).
    pub fn is_point_class(&self) -> bool {
        self.inner.partition.is_empty()
    }

    /// Access the underlying `SchubertClass`.
    pub(crate) fn as_inner(&self) -> &SchubertClass {
        &self.inner
    }

    /// Construct from an existing `SchubertClass`.
    #[allow(dead_code)]
    pub(crate) fn from_inner(inner: SchubertClass) -> Self {
        Self { inner }
    }
}
