#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(all(not(feature = "std"), feature = "alloc"), feature(alloc))]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(feature = "std")]
pub mod artifact;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod bundle;
#[cfg(feature = "std")]
pub mod command;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod export;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod lean;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod obligation;
#[cfg(feature = "std")]
pub mod report;
#[cfg(feature = "std")]
pub mod runner;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod signature;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod smt;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod trust;

#[cfg(feature = "std")]
pub use artifact::{
    ArtifactBatch, ArtifactLayout, ArtifactRecord, dry_run_bundle_artifacts, write_bundle_artifacts,
};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use bundle::ObligationBundle;
#[cfg(feature = "std")]
pub use command::{CommandKind, InvocationPlan, LeanConfig, SmtConfig};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use export::{export_lean_bundle, export_smt_batch, export_smt_bundle};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use lean::{Lean4, export_module as export_lean_module};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use obligation::{Declaration, Obligation, Origin, ProofDialect, Sort, Term, VerificationTier};
#[cfg(feature = "std")]
pub use report::{
    ModuleReport, ObligationReport, VerificationReport, dry_run_report, execute_report,
};
#[cfg(feature = "std")]
pub use runner::{
    DryRunner, ExecutionResult, ExecutionStatus, LocalProcessRunner, VerifierRunner,
    parse_smt_status,
};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use signature::{AlgebraicSignature, BinarySymbol, ConstantSymbol, UnarySymbol};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use smt::{SmtLib2, export_obligation as export_smt_obligation};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use trust::{
    Certificate, Certified, ExternalTrust, LeanCertificate, SmtCertificate, VerificationBackend,
};
