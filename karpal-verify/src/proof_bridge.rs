#[cfg(not(feature = "std"))]
use alloc::{format, string::String};
#[cfg(feature = "std")]
use std::{format, string::String};

use crate::{Certificate, Obligation, VerificationBackend};

/// Runtime/test evidence emitted by `karpal-proof` or `karpal-proof-derive` checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofEvidence {
    pub test_path: String,
    pub case_count: usize,
    pub seed: Option<String>,
    pub notes: Option<String>,
}

impl ProofEvidence {
    pub fn passed_tests(test_path: impl Into<String>, case_count: usize) -> Self {
        Self {
            test_path: test_path.into(),
            case_count,
            seed: None,
            notes: None,
        }
    }

    pub fn with_seed(mut self, seed: impl Into<String>) -> Self {
        self.seed = Some(seed.into());
        self
    }

    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    fn witness_ref(&self) -> String {
        match &self.seed {
            Some(seed) => format!("{}#seed={seed}", self.test_path),
            None => self.test_path.clone(),
        }
    }

    fn certificate_notes(&self) -> String {
        let mut notes = format!(
            "karpal-proof evidence from {} passing case(s)",
            self.case_count
        );
        if let Some(seed) = &self.seed {
            notes.push_str(&format!("; seed {seed}"));
        }
        if let Some(extra) = &self.notes {
            notes.push_str("; ");
            notes.push_str(extra);
        }
        notes
    }
}

/// Helper for turning proof/test evidence into certificate metadata.
pub struct ProofBridge;

impl ProofBridge {
    pub fn certificate<B: VerificationBackend>(
        obligation: &Obligation,
        evidence: ProofEvidence,
    ) -> Certificate {
        Certificate::from_obligation::<B>(obligation, evidence.witness_ref())
            .with_notes(evidence.certificate_notes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Obligation, Origin, ProofTestCertificate, Sort};

    #[test]
    fn proof_bridge_builds_certificate_from_passed_law_check() {
        let obligation = Obligation::associativity(
            "sum_assoc",
            Origin::new("karpal-core", "Sum<i32>"),
            Sort::Int,
            "combine",
        );
        let evidence = ProofEvidence::passed_tests("verify_semigroup_sum::associativity", 256);

        let cert = ProofBridge::certificate::<ProofTestCertificate>(&obligation, evidence);

        assert_eq!(cert.backend, "karpal-proof");
        assert!(
            cert.witness_ref
                .contains("verify_semigroup_sum::associativity")
        );
        assert!(cert.notes.as_deref().unwrap().contains("256"));
    }

    #[test]
    fn proof_evidence_can_record_seed_and_notes() {
        let evidence = ProofEvidence::passed_tests("verify_group::left_inverse", 128)
            .with_seed("0xdeadbeef")
            .with_notes("regression suite");

        assert_eq!(evidence.test_path, "verify_group::left_inverse");
        assert_eq!(evidence.case_count, 128);
        assert_eq!(evidence.seed.as_deref(), Some("0xdeadbeef"));
        assert_eq!(evidence.notes.as_deref(), Some("regression suite"));
    }
}
