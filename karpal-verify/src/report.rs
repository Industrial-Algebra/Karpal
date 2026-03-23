use crate::{
    ArtifactBatch, Certificate, DryRunner, ExecutionResult, ExecutionStatus, Obligation,
    ObligationBundle, VerifierRunner,
};

#[cfg(feature = "std")]
use std::fmt::Write as _;

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
    pub lean_theorem_ref: Option<String>,
    pub lean_diagnostics: Vec<String>,
    pub result: Option<ExecutionResult>,
    pub certificate: Option<Certificate>,
    pub lean_certificate: Option<Certificate>,
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
    pub theorem_refs: Vec<String>,
    pub prelude_imports: Vec<String>,
    pub prelude_aliases: Vec<String>,
    pub diagnostics: Vec<String>,
    pub theorem_failures: Vec<String>,
    pub result: Option<ExecutionResult>,
    pub certificate: Option<Certificate>,
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

    #[cfg(feature = "std")]
    pub fn to_json(&self) -> String {
        fn esc(s: &str) -> String {
            s.replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
        }

        fn render_certificate_json(certificate: &Certificate) -> String {
            format!(
                "{{\"backend\":\"{}\",\"witness_ref\":\"{}\"}}",
                esc(certificate.backend),
                esc(&certificate.witness_ref)
            )
        }

        let mut out = String::new();
        let _ = write!(
            out,
            "{{\"bundle_name\":\"{}\",\"root\":\"{}\",\"success_count\":{},\"failure_count\":{},\"obligations\":[",
            esc(&self.bundle_name),
            esc(&self.root),
            self.success_count(),
            self.failure_count()
        );

        for (idx, obligation) in self.obligations.iter().enumerate() {
            if idx > 0 {
                out.push(',');
            }
            let _ = write!(
                out,
                "{{\"name\":\"{}\",\"summary\":\"{}\",\"status\":\"{}\",\"artifact_path\":{},\"lean_theorem_ref\":{},\"lean_diagnostic_count\":{},\"certificate\":{},\"lean_certificate\":{}}}",
                esc(&obligation.obligation_name),
                esc(&obligation.summary),
                obligation
                    .status()
                    .map(|s| format!("{s:?}"))
                    .unwrap_or_else(|| "None".into()),
                obligation
                    .artifact_path
                    .as_ref()
                    .map(|p| format!("\"{}\"", esc(p)))
                    .unwrap_or_else(|| "null".into()),
                obligation
                    .lean_theorem_ref
                    .as_ref()
                    .map(|r| format!("\"{}\"", esc(r)))
                    .unwrap_or_else(|| "null".into()),
                obligation.lean_diagnostics.len(),
                obligation
                    .certificate
                    .as_ref()
                    .map(render_certificate_json)
                    .unwrap_or_else(|| "null".into()),
                obligation
                    .lean_certificate
                    .as_ref()
                    .map(render_certificate_json)
                    .unwrap_or_else(|| "null".into())
            );
        }
        out.push(']');
        if let Some(module) = &self.lean_module {
            let _ = write!(
                out,
                ",\"lean_module\":{{\"module_name\":\"{}\",\"status\":\"{}\",\"theorem_count\":{},\"import_count\":{},\"alias_count\":{},\"diagnostic_count\":{},\"theorem_failure_count\":{},\"certificate\":{}}}",
                esc(&module.module_name),
                module
                    .status()
                    .map(|s| format!("{s:?}"))
                    .unwrap_or_else(|| "None".into()),
                module.theorem_refs.len(),
                module.prelude_imports.len(),
                module.prelude_aliases.len(),
                module.diagnostics.len(),
                module.theorem_failures.len(),
                module
                    .certificate
                    .as_ref()
                    .map(render_certificate_json)
                    .unwrap_or_else(|| "null".into())
            );
        }
        out.push('}');
        out
    }

    #[cfg(feature = "std")]
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "# Verification Report");
        let _ = writeln!(out);
        let _ = writeln!(out, "- Bundle: `{}`", self.bundle_name);
        let _ = writeln!(out, "- Root: `{}`", self.root);
        let _ = writeln!(out, "- Successes: {}", self.success_count());
        let _ = writeln!(out, "- Failures: {}", self.failure_count());
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "| Obligation | Status | Artifact | Lean theorem | Lean diagnostics | SMT certificate | Lean certificate |"
        );
        let _ = writeln!(out, "|---|---|---|---|---|---|---|");
        for obligation in &self.obligations {
            let _ = writeln!(
                out,
                "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |",
                obligation.obligation_name,
                obligation
                    .status()
                    .map(|s| format!("{s:?}"))
                    .unwrap_or_else(|| "None".into()),
                obligation.artifact_path.as_deref().unwrap_or("-"),
                obligation.lean_theorem_ref.as_deref().unwrap_or("-"),
                if obligation.lean_diagnostics.is_empty() {
                    "-".into()
                } else {
                    obligation.lean_diagnostics.join("; ")
                },
                obligation
                    .certificate
                    .as_ref()
                    .map(|c| c.backend)
                    .unwrap_or("-"),
                obligation
                    .lean_certificate
                    .as_ref()
                    .map(|c| c.backend)
                    .unwrap_or("-")
            );
        }
        if let Some(module) = &self.lean_module {
            let _ = writeln!(out);
            let _ = writeln!(out, "Lean module: `{}`", module.module_name);
            let _ = writeln!(out, "Lean theorems: {}", module.theorem_refs.len());
            let _ = writeln!(out, "Lean imports: {}", module.prelude_imports.len());
            let _ = writeln!(out, "Lean aliases: {}", module.prelude_aliases.len());
            let _ = writeln!(out, "Lean diagnostics: {}", module.diagnostics.len());
            let _ = writeln!(
                out,
                "Lean theorem failures: {}",
                module.theorem_failures.len()
            );
            let _ = writeln!(
                out,
                "Lean certificate: `{}`",
                module
                    .certificate
                    .as_ref()
                    .map(|c| c.witness_ref.as_str())
                    .unwrap_or("-")
            );
        }
        out
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
    let lean_record = artifacts
        .records
        .iter()
        .find(|record| record.path.ends_with(".lean"));
    let lean_result = lean_record.and_then(|record| {
        artifacts
            .plans
            .iter()
            .find(|plan| {
                plan.kind == crate::CommandKind::Lean
                    && plan.input_files.iter().any(|f| f == &record.path)
            })
            .map(|plan| runner.run(plan))
    });

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
        let lean_theorem = artifacts
            .lean_export
            .as_ref()
            .and_then(|export| export.theorem_for_obligation(&obligation.name).cloned());
        let lean_theorem_ref = artifacts.lean_export.as_ref().and_then(|export| {
            lean_theorem
                .as_ref()
                .map(|theorem| theorem.witness_ref(&export.module_name))
        });
        let lean_diagnostics = lean_result
            .as_ref()
            .and_then(|result| result.lean_output.as_ref())
            .and_then(|output| {
                lean_theorem.as_ref().map(|theorem| {
                    output
                        .theorem_diagnostics(theorem)
                        .into_iter()
                        .map(|diagnostic| diagnostic.message.clone())
                        .collect::<Vec<_>>()
                })
            })
            .unwrap_or_default();
        let lean_certificate = lean_result.as_ref().and_then(|result| {
            lean_theorem_ref.as_ref().and_then(|witness_ref| {
                certificate_for_witness::<crate::LeanCertificate>(
                    result,
                    obligation,
                    witness_ref.clone(),
                    lean_record.map(|record| record.path.clone()),
                )
            })
        });

        obligations.push(ObligationReport {
            obligation_name: obligation.name.clone(),
            summary: obligation.summary(),
            artifact_path,
            lean_theorem_ref,
            lean_diagnostics,
            result,
            certificate,
            lean_certificate,
        });
    }

    let lean_module = lean_record.map(|record| {
        let theorem_refs = artifacts
            .lean_export
            .as_ref()
            .map(|export| {
                export
                    .theorems
                    .iter()
                    .map(|theorem| theorem.witness_ref(&export.module_name))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let prelude_imports = artifacts
            .lean_export
            .as_ref()
            .map(|export| {
                export
                    .prelude
                    .imports
                    .iter()
                    .map(|import| import.module.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let prelude_aliases = artifacts
            .lean_export
            .as_ref()
            .map(|export| {
                export
                    .prelude
                    .aliases
                    .iter()
                    .map(|alias| format!("{} := {}", alias.alias, alias.target))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let diagnostics = lean_result
            .as_ref()
            .and_then(|result| result.lean_output.as_ref())
            .map(|output| {
                output
                    .diagnostics
                    .iter()
                    .map(|diagnostic| diagnostic.message.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let theorem_failures = match (
            artifacts.lean_export.as_ref(),
            lean_result
                .as_ref()
                .and_then(|result| result.lean_output.as_ref()),
        ) {
            (Some(export), Some(output)) => export
                .theorems
                .iter()
                .filter(|theorem| output.has_theorem_failure(theorem))
                .map(|theorem| theorem.witness_ref(&export.module_name))
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        };
        let certificate = lean_result.as_ref().and_then(|result| {
            certificate_for_module(
                result,
                &record.name,
                Some(record.path.clone()),
                theorem_refs.len(),
            )
        });

        ModuleReport {
            module_name: record.name.clone(),
            artifact_path: Some(record.path.clone()),
            theorem_refs,
            prelude_imports,
            prelude_aliases,
            diagnostics,
            theorem_failures,
            result: lean_result.clone(),
            certificate,
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
    let policy = result.verification_policy();
    let witness_ref = format!("{}:{}", result.plan.executable, policy.witness_suffix);
    match result.plan.kind {
        crate::CommandKind::Smt => certificate_for_witness::<crate::SmtCertificate>(
            result,
            obligation,
            witness_ref,
            artifact_path,
        ),
        crate::CommandKind::Lean => certificate_for_witness::<crate::LeanCertificate>(
            result,
            obligation,
            witness_ref,
            artifact_path,
        ),
    }
}

fn certificate_for_witness<B: crate::VerificationBackend>(
    result: &ExecutionResult,
    obligation: &Obligation,
    witness_ref: String,
    artifact_path: Option<String>,
) -> Option<Certificate> {
    if !result.is_success() {
        return None;
    }

    let mut cert = Certificate::from_obligation::<B>(obligation, witness_ref);

    if let Some(version) = &result.backend_version {
        cert = cert.with_backend_version(version.clone());
    }

    if let Some(path) = artifact_path {
        cert = cert.with_artifact_path(path);
    }

    Some(cert)
}

fn certificate_for_module(
    result: &ExecutionResult,
    module_name: &str,
    artifact_path: Option<String>,
    theorem_count: usize,
) -> Option<Certificate> {
    if !result.is_success() {
        return None;
    }

    let mut cert = Certificate::new(
        <crate::LeanCertificate as crate::VerificationBackend>::NAME,
        format!("Lean module {module_name}"),
        crate::LeanCertificate::module_ref(module_name),
    )
    .with_notes(format!(
        "verified module containing {theorem_count} theorem(s)"
    ));

    if let Some(version) = &result.backend_version {
        cert = cert.with_backend_version(version.clone());
    }

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
        assert_eq!(report.lean_module.as_ref().unwrap().theorem_refs.len(), 3);
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
                    backend_version: Some("tool 1.0".into()),
                    smt_output: Some(crate::SmtOutput {
                        status: Some(ExecutionStatus::Unsat),
                        model: None,
                        reason_unknown: None,
                    }),
                    lean_output: (plan.kind == crate::CommandKind::Lean).then(|| {
                        crate::LeanOutput {
                            diagnostics: vec![crate::LeanDiagnostic {
                                file: Some("lean/KarpalVerify.lean".into()),
                                line: Some(7),
                                column: Some(2),
                                severity: "error".into(),
                                message: "unsolved goals in theorem associativity".into(),
                                theorem_hits: vec!["associativity".into()],
                            }],
                            theorem_hits: vec!["associativity".into()],
                        }
                    }),
                }
            }
        }

        let report = execute_report(&bundle, &artifacts, &SuccessRunner);
        assert_eq!(report.success_count(), 1);
        assert!(report.obligations[0].certificate.is_some());
        assert_eq!(
            report.obligations[0]
                .certificate
                .as_ref()
                .and_then(|c| c.backend_version.as_deref()),
            Some("tool 1.0")
        );
        assert_eq!(
            report
                .lean_module
                .as_ref()
                .map(|module| module.theorem_refs.len()),
            Some(1)
        );
        assert_eq!(
            report.obligations[0].lean_theorem_ref.as_deref(),
            Some("KarpalVerify.associativity")
        );
        assert_eq!(
            report.obligations[0].lean_diagnostics,
            vec!["unsolved goals in theorem associativity".to_string()]
        );
        assert_eq!(
            report.obligations[0]
                .lean_certificate
                .as_ref()
                .map(|certificate| certificate.backend),
            Some("lean4")
        );
        assert_eq!(
            report
                .lean_module
                .as_ref()
                .and_then(|module| module.certificate.as_ref())
                .map(|certificate| certificate.witness_ref.as_str()),
            Some("KarpalVerify")
        );
        assert_eq!(
            report
                .lean_module
                .as_ref()
                .map(|module| module.theorem_failures.clone()),
            Some(vec!["KarpalVerify.associativity".to_string()])
        );
    }

    #[test]
    fn report_serialization_includes_summary_data() {
        let report = VerificationReport {
            bundle_name: "demo".into(),
            root: "target/demo".into(),
            obligations: vec![ObligationReport {
                obligation_name: "assoc".into(),
                summary: "demo::assoc [associativity]".into(),
                artifact_path: Some("target/demo/smt/assoc.smt2".into()),
                lean_theorem_ref: None,
                lean_diagnostics: Vec::new(),
                result: None,
                certificate: None,
                lean_certificate: None,
            }],
            lean_module: None,
        };

        let json = report.to_json();
        let markdown = report.to_markdown();
        assert!(json.contains("\"bundle_name\":\"demo\""));
        assert!(markdown.contains("# Verification Report"));
        assert!(markdown.contains("`assoc`"));
    }

    #[test]
    fn theorem_mapping_uses_exported_theorem_identity_not_obligation_name() {
        let sig = AlgebraicSignature::semiring(Sort::Int, "add", "mul", "zero", "one");
        let bundle = ObligationBundle::semiring(
            "sum_semiring",
            Origin::new("karpal-core", "Semiring for Sum<i32>"),
            &sig,
        );
        let layout = ArtifactLayout::new("target/karpal-verify-report-test-3");
        let artifacts = dry_run_bundle_artifacts(
            &bundle,
            &layout,
            "KarpalVerify",
            &SmtConfig::default(),
            &LeanConfig::default(),
        );

        struct TheoremNameRunner;
        impl VerifierRunner for TheoremNameRunner {
            fn run(&self, plan: &crate::InvocationPlan) -> ExecutionResult {
                ExecutionResult {
                    plan: plan.clone(),
                    status: match plan.kind {
                        crate::CommandKind::Smt => ExecutionStatus::Unsat,
                        crate::CommandKind::Lean => ExecutionStatus::Failure,
                    },
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: Some(1),
                    backend_version: Some("tool 1.0".into()),
                    smt_output: Some(crate::SmtOutput {
                        status: Some(ExecutionStatus::Unsat),
                        model: None,
                        reason_unknown: None,
                    }),
                    lean_output: (plan.kind == crate::CommandKind::Lean).then(|| {
                        crate::LeanOutput {
                            diagnostics: vec![crate::LeanDiagnostic {
                                file: Some("lean/KarpalVerify.lean".into()),
                                line: Some(9),
                                column: Some(2),
                                severity: "error".into(),
                                message: "unsolved goals in theorem left_distributivity".into(),
                                theorem_hits: vec!["left_distributivity".into()],
                            }],
                            theorem_hits: vec!["left_distributivity".into()],
                        }
                    }),
                }
            }
        }

        let report = execute_report(&bundle, &artifacts, &TheoremNameRunner);
        let left_distributivity = report
            .obligations
            .iter()
            .find(|entry| entry.obligation_name == "left_distributivity")
            .expect("left distributivity obligation should be present");
        assert_eq!(
            left_distributivity.lean_diagnostics,
            vec!["unsolved goals in theorem left_distributivity".to_string()]
        );
        assert_eq!(
            report
                .lean_module
                .as_ref()
                .map(|module| module.theorem_failures.clone()),
            Some(vec!["KarpalVerify.left_distributivity".to_string()])
        );
    }

    #[test]
    fn theorem_mapping_can_fall_back_to_exported_line_spans() {
        let sig = AlgebraicSignature::semiring(Sort::Int, "add", "mul", "zero", "one");
        let bundle = ObligationBundle::semiring(
            "sum_semiring",
            Origin::new("karpal-core", "Semiring for Sum<i32>"),
            &sig,
        );
        let layout = ArtifactLayout::new("target/karpal-verify-report-test-4");
        let artifacts = dry_run_bundle_artifacts(
            &bundle,
            &layout,
            "KarpalVerify",
            &SmtConfig::default(),
            &LeanConfig::default(),
        );

        let failure_line = artifacts
            .lean_export
            .as_ref()
            .and_then(|export| export.theorem_for_obligation("left_distributivity"))
            .map(|theorem| theorem.declaration_start_line)
            .expect("left distributivity theorem span should be present");

        struct LocationRunner {
            failure_line: usize,
        }
        impl VerifierRunner for LocationRunner {
            fn run(&self, plan: &crate::InvocationPlan) -> ExecutionResult {
                ExecutionResult {
                    plan: plan.clone(),
                    status: match plan.kind {
                        crate::CommandKind::Smt => ExecutionStatus::Unsat,
                        crate::CommandKind::Lean => ExecutionStatus::Failure,
                    },
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: Some(1),
                    backend_version: Some("tool 1.0".into()),
                    smt_output: Some(crate::SmtOutput {
                        status: Some(ExecutionStatus::Unsat),
                        model: None,
                        reason_unknown: None,
                    }),
                    lean_output: (plan.kind == crate::CommandKind::Lean).then(|| {
                        crate::LeanOutput {
                            diagnostics: vec![crate::LeanDiagnostic {
                                file: Some("lean/KarpalVerify.lean".into()),
                                line: Some(self.failure_line),
                                column: Some(2),
                                severity: "error".into(),
                                message: "type mismatch".into(),
                                theorem_hits: Vec::new(),
                            }],
                            theorem_hits: Vec::new(),
                        }
                    }),
                }
            }
        }

        let report = execute_report(&bundle, &artifacts, &LocationRunner { failure_line });
        let left_distributivity = report
            .obligations
            .iter()
            .find(|entry| entry.obligation_name == "left_distributivity")
            .expect("left distributivity obligation should be present");
        assert_eq!(
            left_distributivity.lean_diagnostics,
            vec!["type mismatch".to_string()]
        );
        assert_eq!(
            report
                .lean_module
                .as_ref()
                .map(|module| module.theorem_failures.clone()),
            Some(vec!["KarpalVerify.left_distributivity".to_string()])
        );
    }
}
