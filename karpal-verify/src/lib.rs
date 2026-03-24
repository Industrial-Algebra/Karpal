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
#[cfg(feature = "std")]
pub mod session;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod signature;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod smt;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod trust;

#[cfg(feature = "std")]
pub use artifact::{
    ArtifactBatch, ArtifactLayout, ArtifactRecord, LEAN_MANIFEST_SCHEMA_VERSION, LeanManifest,
    LeanManifestAlias, LeanManifestPrelude, LeanManifestProject, LeanManifestReportFiles,
    LeanManifestTheorem, dry_run_bundle_artifacts, write_bundle_artifacts,
};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use bundle::ObligationBundle;
#[cfg(feature = "std")]
pub use command::{CommandKind, InvocationPlan, LeanConfig, LeanDriver, SmtConfig};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use export::{
    export_lean_bundle, export_lean_bundle_structured, export_lean_bundle_structured_with_prelude,
    export_lean_bundle_with_prelude, export_smt_batch, export_smt_bundle,
};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use lean::{
    Lean4, LeanAlias, LeanExport, LeanImport, LeanPrelude, LeanProject, LeanTheorem, export,
    export_module as export_lean_module, export_module_with_prelude, export_with_prelude,
};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use obligation::{Declaration, Obligation, Origin, ProofDialect, Sort, Term, VerificationTier};
#[cfg(feature = "std")]
pub use report::{
    ModuleReport, ObligationReport, VERIFICATION_REPORT_SCHEMA_VERSION, VerificationReport,
    dry_run_report, execute_report,
};
#[cfg(feature = "std")]
pub use runner::{
    DryRunner, ExecutionResult, ExecutionStatus, LeanDiagnostic, LeanOutput, LocalProcessRunner,
    SmtOutput, VerificationPolicy, VerifierRunner, parse_lean_output, parse_smt_output,
    parse_smt_status,
};
#[cfg(feature = "std")]
pub use session::{
    DEFAULT_REPORT_STEM, ReportFiles, VERIFICATION_SIDECAR_SCHEMA_VERSION, VerificationOutput,
    VerificationSession, verify_bundle, verify_bundle_with_ci_outputs,
};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use signature::{AlgebraicSignature, BinarySymbol, ConstantSymbol, UnarySymbol};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use smt::{SmtLib2, export_obligation as export_smt_obligation};
#[cfg(any(feature = "std", feature = "alloc"))]
pub use trust::{
    Certificate, Certified, ExternalTrust, LeanCertificate, SmtCertificate, VerificationBackend,
};
