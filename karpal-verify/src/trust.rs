use core::marker::PhantomData;

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

use crate::Obligation;

use karpal_proof::Proven;

/// Marker trait for external verification backends.
pub trait VerificationBackend {
    const NAME: &'static str;
}

/// SMT-LIB2 / solver-backed verification.
pub struct SmtCertificate;
impl VerificationBackend for SmtCertificate {
    const NAME: &'static str = "smtlib2";
}

/// Lean 4 proof assistant verification.
pub struct LeanCertificate;
impl VerificationBackend for LeanCertificate {
    const NAME: &'static str = "lean4";
}

/// Metadata describing imported external evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Certificate {
    pub backend: &'static str,
    pub backend_version: Option<String>,
    pub obligation: String,
    pub obligation_digest: Option<String>,
    pub witness_ref: String,
    pub artifact_path: Option<String>,
    pub notes: Option<String>,
}

impl Certificate {
    pub fn new(
        backend: &'static str,
        obligation: impl Into<String>,
        witness_ref: impl Into<String>,
    ) -> Self {
        Self {
            backend,
            backend_version: None,
            obligation: obligation.into(),
            obligation_digest: None,
            witness_ref: witness_ref.into(),
            artifact_path: None,
            notes: None,
        }
    }

    pub fn from_obligation<B: VerificationBackend>(
        obligation: &Obligation,
        witness_ref: impl Into<String>,
    ) -> Self {
        Self::new(B::NAME, obligation.summary(), witness_ref)
            .with_obligation_digest(obligation_digest(obligation))
    }

    pub fn with_backend_version(mut self, version: impl Into<String>) -> Self {
        self.backend_version = Some(version.into());
        self
    }

    pub fn with_obligation_digest(mut self, digest: impl Into<String>) -> Self {
        self.obligation_digest = Some(digest.into());
        self
    }

    pub fn with_artifact_path(mut self, path: impl Into<String>) -> Self {
        self.artifact_path = Some(path.into());
        self
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

fn obligation_digest(obligation: &Obligation) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in obligation.summary().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

/// Explicit trust boundary for externally-certified values.
///
/// This type does not silently become `Proven<P, T>`. Converting from
/// external evidence into a Karpal proof witness is `unsafe`, making the
/// trust boundary visible in code review.
pub struct Certified<B, P, T> {
    value: T,
    certificate: Certificate,
    _phantom: PhantomData<(B, P)>,
}

/// Alias emphasizing that the certificate crosses an external trust boundary.
pub type ExternalTrust<B, P, T> = Certified<B, P, T>;

impl<B, P, T> Certified<B, P, T> {
    /// Import an externally checked value.
    ///
    /// # Safety
    ///
    /// The caller must ensure the certificate genuinely establishes property
    /// `P` for `value`, according to backend `B`.
    pub unsafe fn assume(value: T, certificate: Certificate) -> Self {
        Self {
            value,
            certificate,
            _phantom: PhantomData,
        }
    }

    pub fn certificate(&self) -> &Certificate {
        &self.certificate
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn into_inner(self) -> T {
        self.value
    }

    /// Convert external evidence into `Proven<P, T>`.
    ///
    /// # Safety
    ///
    /// This is the explicit trust boundary: the caller accepts the imported
    /// certificate as sound.
    pub unsafe fn into_proven(self) -> Proven<P, T> {
        unsafe { Proven::axiom(self.value) }
    }
}

impl<B: VerificationBackend, P, T: core::fmt::Debug> core::fmt::Debug for Certified<B, P, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Certified")
            .field("backend", &B::NAME)
            .field("value", &self.value)
            .field("certificate", &self.certificate)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Origin, Sort};
    use karpal_proof::{IsAssociative, Proven};

    #[test]
    fn certificate_stores_metadata() {
        let cert = Certificate::new("lean4", "sum_assoc", "theorem Sum.assoc")
            .with_backend_version("4.15.0")
            .with_artifact_path(".artifacts/SumAssoc.lean")
            .with_notes("ported from generated module");
        assert_eq!(cert.backend, "lean4");
        assert_eq!(cert.backend_version.as_deref(), Some("4.15.0"));
        assert_eq!(
            cert.artifact_path.as_deref(),
            Some(".artifacts/SumAssoc.lean")
        );
        assert!(cert.notes.as_deref().unwrap().contains("ported"));
    }

    #[test]
    fn certified_wraps_value() {
        let cert = Certificate::new("smtlib2", "sum_assoc", "z3:model:1");
        let verified = unsafe { Certified::<SmtCertificate, IsAssociative, i32>::assume(7, cert) };
        assert_eq!(*verified.value(), 7);
        assert_eq!(verified.certificate().backend, "smtlib2");
    }

    #[test]
    fn trust_boundary_can_be_crossed_explicitly() {
        let cert = Certificate::new("lean4", "sum_assoc", "Sum.assoc");
        let verified =
            unsafe { Certified::<LeanCertificate, IsAssociative, i32>::assume(11, cert) };
        let proven: Proven<IsAssociative, i32> = unsafe { verified.into_proven() };
        assert_eq!(*proven.value(), 11);
    }

    #[test]
    fn certificate_can_be_derived_from_obligation() {
        let obligation = Obligation::associativity(
            "sum_assoc",
            Origin::new("karpal-core", "Semigroup for i32"),
            Sort::Int,
            "combine",
        );
        let cert = Certificate::from_obligation::<SmtCertificate>(&obligation, "z3:proof:assoc")
            .with_backend_version("4.13.0");
        assert_eq!(cert.backend, "smtlib2");
        assert!(cert.obligation.contains("associativity"));
        assert!(
            cert.obligation_digest
                .as_deref()
                .unwrap()
                .starts_with("fnv1a64:")
        );
    }
}
