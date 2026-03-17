use crate::{
    ArtifactBatch, Certificate, DryRunner, ExecutionResult, ExecutionStatus, Obligation,
    ObligationBundle, VerifierRunner,
};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

/// Per-obligation verification report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationReport {
    pub obligation_name: String,
    pub summary: String,
    pub artifact_path: Option<String>,
    pub result: Option<ExecutionResult>,
    pub certificate: Option<Certificate>,
}

impl ObligationReport {
    pub fn status(&self) -> Option<ExecutionStatus> {
        self.result.as_ref().map(|result| result.status)
    }

    pub fn succeeded(&self) -> bool {
        self.result
            .as_ref()
            .map(|result| result.is_success())
            .unwrap_or(false)
    }
}

/// Report for the generated Lean module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleReport {
    pub module_name: String,
    pub artifact_path: Option<String>,
    pub result: Option<ExecutionResult>,
}

impl ModuleReport {
    pub fn status(&self) -> Option<ExecutionStatus> {
        self.result.as_ref().map(|result| result.status)
    }
}

/// Aggregate verification report for a bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationReport {
    pub bundle_name: String,
    pub root: String,
    pub obligations: Vec<ObligationReport>,
    pub lean_module: Option<ModuleReport>,
}

impl VerificationReport {
    pub fn obligation_count(&self) -> usize {
        self.obligations.len()
    }

    pub fn success_count(&self) -> usize {
        self.obligations
            .iter()
            .filter(|report| report.succeeded())
            .count()
    }

    pub fn failure_count(&self) -> usize {
        self.obligation_count().saturating_sub(self.success_count())
    }

    pub fn is_success(&self) -> bool {
        self.failure_count() == 0
    }
}

/// Build a dry-run report without spawning external tools.
pub fn dry_run_report(bundle: &ObligationBundle, artifacts: &ArtifactBatch) -> VerificationReport {
    execute_report(bundle, artifacts, &DryRunner)
}

/// Execute all plans in an artifact batch and attach results back to obligations.
pub fn execute_report(
    bundle: &ObligationBundle,
    artifacts: &ArtifactBatch,
    runner: &impl VerifierRunner,
) -> VerificationReport {
    let mut obligations = Vec::new();

    for obligation in bundle.obligations() {
        let artifact_path = artifacts
            .records
            .iter()
            .find(|record| record.name == obligation.name)
            .map(|record| record.path.clone());
        let result = artifacts
            .plans
            .iter()
            .find(|plan| {
                plan.kind == crate::CommandKind::Smt
                    && plan
                        .input_files
                        .iter()
                        .any(|f| artifact_path.as_ref() == Some(f))
            })
            .map(|plan| runner.run(plan));
        let certificate = result.as_ref().and_then(|result| {
            certificate_for_obligation(result, obligation, artifact_path.clone())
        });

        obligations.push(ObligationReport {
            obligation_name: obligation.name.clone(),
            summary: obligation.summary(),
            artifact_path,
            result,
            certificate,
        });
    }

    let lean_module = artifacts
        .records
        .iter()
        .find(|record| record.path.ends_with(".lean"))
        .map(|record| {
            let result = artifacts
                .plans
                .iter()
                .find(|plan| {
                    plan.kind == crate::CommandKind::Lean
                        && plan.input_files.iter().any(|f| f == &record.path)
                })
                .map(|plan| runner.run(plan));
            ModuleReport {
                module_name: record.name.clone(),
                artifact_path: Some(record.path.clone()),
                result,
            }
        });

    VerificationReport {
        bundle_name: bundle.name.clone(),
        root: artifacts.root.clone(),
        obligations,
        lean_module,
    }
}

fn certificate_for_obligation(
    result: &ExecutionResult,
    obligation: &Obligation,
    artifact_path: Option<String>,
) -> Option<Certificate> {
    if !result.is_success() {
        return None;
    }

    let mut cert = match result.plan.kind {
        crate::CommandKind::Smt => Certificate::from_obligation::<crate::SmtCertificate>(
            obligation,
            format!("{}:unsat", result.plan.executable),
        ),
        crate::CommandKind::Lean => Certificate::from_obligation::<crate::LeanCertificate>(
            obligation,
            format!("{}:ok", result.plan.executable),
        ),
    };

    if let Some(path) = artifact_path {
        cert = cert.with_artifact_path(path);
    }

    Some(cert)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AlgebraicSignature, ArtifactLayout, LeanConfig, Origin, SmtConfig, Sort,
        dry_run_bundle_artifacts,
    };

    #[test]
    fn dry_run_report_attaches_results_to_all_obligations() {
        let sig = AlgebraicSignature::monoid(Sort::Int, "combine", "e");
        let bundle = ObligationBundle::monoid(
            "sum_monoid",
            Origin::new("karpal-core", "Monoid for Sum<i32>"),
            &sig,
        );
        let layout = ArtifactLayout::new("target/karpal-verify-report-test");
        let artifacts = dry_run_bundle_artifacts(
            &bundle,
            &layout,
            "KarpalVerify",
            &SmtConfig::default(),
            &LeanConfig::default(),
        );

        let report = dry_run_report(&bundle, &artifacts);
        assert_eq!(report.obligation_count(), 3);
        assert!(
            report
                .obligations
                .iter()
                .all(|entry| entry.status() == Some(ExecutionStatus::DryRun))
        );
        assert_eq!(
            report.lean_module.as_ref().unwrap().status(),
            Some(ExecutionStatus::DryRun)
        );
    }

    #[test]
    fn successful_execution_yields_certificates() {
        let sig = AlgebraicSignature::semigroup(Sort::Int, "combine");
        let bundle = ObligationBundle::semigroup(
            "sum_semigroup",
            Origin::new("karpal-core", "Semigroup for Sum<i32>"),
            &sig,
        );
        let layout = ArtifactLayout::new("target/karpal-verify-report-test-2");
        let artifacts = dry_run_bundle_artifacts(
            &bundle,
            &layout,
            "KarpalVerify",
            &SmtConfig::default(),
            &LeanConfig::default(),
        );

        struct SuccessRunner;
        impl VerifierRunner for SuccessRunner {
            fn run(&self, plan: &crate::InvocationPlan) -> ExecutionResult {
                ExecutionResult {
                    plan: plan.clone(),
                    status: match plan.kind {
                        crate::CommandKind::Smt => ExecutionStatus::Unsat,
                        crate::CommandKind::Lean => ExecutionStatus::Success,
                    },
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: Some(0),
                }
            }
        }

        let report = execute_report(&bundle, &artifacts, &SuccessRunner);
        assert_eq!(report.success_count(), 1);
        assert!(report.obligations[0].certificate.is_some());
    }
}
