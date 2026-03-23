use crate::{
    ArtifactLayout, DryRunner, LeanConfig, LocalProcessRunner, ObligationBundle, SmtConfig,
    VerificationReport, VerifierRunner, dry_run_bundle_artifacts, dry_run_report, execute_report,
    write_bundle_artifacts,
};
use std::{fs, io, path::Path, string::String};

/// Default file stem used for CI-oriented verification summaries.
pub const DEFAULT_REPORT_STEM: &str = "verification-report";

/// Paths written for CI-oriented report summaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportFiles {
    pub json_path: String,
    pub markdown_path: String,
}

/// End-to-end verification session configuration for a bundle.
#[derive(Debug, Clone)]
pub struct VerificationSession {
    bundle: ObligationBundle,
    layout: ArtifactLayout,
    lean_module_name: String,
    smt: SmtConfig,
    lean: LeanConfig,
    report_stem: String,
}

impl VerificationSession {
    pub fn new(
        bundle: ObligationBundle,
        layout: ArtifactLayout,
        lean_module_name: impl Into<String>,
    ) -> Self {
        Self {
            bundle,
            layout,
            lean_module_name: lean_module_name.into(),
            smt: SmtConfig::default(),
            lean: LeanConfig::default(),
            report_stem: DEFAULT_REPORT_STEM.into(),
        }
    }

    pub fn with_smt_config(mut self, smt: SmtConfig) -> Self {
        self.smt = smt;
        self
    }

    pub fn with_lean_config(mut self, lean: LeanConfig) -> Self {
        self.lean = lean;
        self
    }

    pub fn with_report_stem(mut self, report_stem: impl Into<String>) -> Self {
        self.report_stem = report_stem.into();
        self
    }

    pub fn bundle(&self) -> &ObligationBundle {
        &self.bundle
    }

    pub fn layout(&self) -> &ArtifactLayout {
        &self.layout
    }

    pub fn lean_module_name(&self) -> &str {
        &self.lean_module_name
    }

    pub fn report_stem(&self) -> &str {
        &self.report_stem
    }

    /// Build dry-run artifacts and return the attached verification report.
    pub fn dry_run_report(&self) -> VerificationReport {
        let artifacts = dry_run_bundle_artifacts(
            &self.bundle,
            &self.layout,
            &self.lean_module_name,
            &self.smt,
            &self.lean,
        );
        dry_run_report(&self.bundle, &artifacts)
    }

    /// Build artifacts, execute plans, and return the resulting verification report.
    pub fn verify_report(&self, runner: &impl VerifierRunner) -> io::Result<VerificationReport> {
        let artifacts = write_bundle_artifacts(
            &self.bundle,
            &self.layout,
            &self.lean_module_name,
            &self.smt,
            &self.lean,
        )?;
        Ok(execute_report(&self.bundle, &artifacts, runner))
    }

    /// Build artifacts, execute plans locally, and return the resulting verification report.
    pub fn verify_local_report(&self) -> io::Result<VerificationReport> {
        self.verify_report(&LocalProcessRunner)
    }

    /// Build artifacts, execute plans, and write JSON / Markdown summaries beside them.
    pub fn verify_with_ci_outputs(
        &self,
        runner: &impl VerifierRunner,
    ) -> io::Result<VerificationOutput> {
        let report = self.verify_report(runner)?;
        let report_files = write_report_files(&self.layout.root, &self.report_stem, &report)?;
        Ok(VerificationOutput {
            report,
            report_files,
        })
    }

    /// Build artifacts, execute plans locally, and write CI-oriented summaries.
    pub fn verify_local_with_ci_outputs(&self) -> io::Result<VerificationOutput> {
        self.verify_with_ci_outputs(&LocalProcessRunner)
    }

    /// Build artifacts via the real writer, then execute a dry-run runner.
    pub fn verify_with_dry_runner(&self) -> io::Result<VerificationReport> {
        self.verify_report(&DryRunner)
    }
}

/// Report plus the CI-oriented files written beside generated artifacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationOutput {
    pub report: VerificationReport,
    pub report_files: ReportFiles,
}

/// One-shot orchestration helper.
pub fn verify_bundle(
    bundle: &ObligationBundle,
    layout: &ArtifactLayout,
    lean_module_name: &str,
    smt: &SmtConfig,
    lean: &LeanConfig,
    runner: &impl VerifierRunner,
) -> io::Result<VerificationReport> {
    VerificationSession::new(bundle.clone(), layout.clone(), lean_module_name)
        .with_smt_config(smt.clone())
        .with_lean_config(lean.clone())
        .verify_report(runner)
}

/// One-shot orchestration helper that also writes CI-oriented report files.
pub fn verify_bundle_with_ci_outputs(
    bundle: &ObligationBundle,
    layout: &ArtifactLayout,
    lean_module_name: &str,
    smt: &SmtConfig,
    lean: &LeanConfig,
    runner: &impl VerifierRunner,
) -> io::Result<VerificationOutput> {
    VerificationSession::new(bundle.clone(), layout.clone(), lean_module_name)
        .with_smt_config(smt.clone())
        .with_lean_config(lean.clone())
        .verify_with_ci_outputs(runner)
}

fn write_report_files(
    root: impl AsRef<Path>,
    report_stem: &str,
    report: &VerificationReport,
) -> io::Result<ReportFiles> {
    let root = root.as_ref();
    fs::create_dir_all(root)?;

    let json_path = root.join(format!("{report_stem}.json"));
    let markdown_path = root.join(format!("{report_stem}.md"));
    fs::write(&json_path, report.to_json())?;
    fs::write(&markdown_path, report.to_markdown())?;

    Ok(ReportFiles {
        json_path: path_to_string(&json_path),
        markdown_path: path_to_string(&markdown_path),
    })
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AlgebraicSignature, ExecutionResult, ExecutionStatus, LeanDiagnostic, LeanOutput, Origin,
        SmtOutput, Sort,
    };

    fn sample_session(root: &Path) -> VerificationSession {
        let sig = AlgebraicSignature::monoid(Sort::Int, "combine", "e");
        let bundle = ObligationBundle::monoid(
            "sum_monoid",
            Origin::new("karpal-core", "Monoid for Sum<i32>"),
            &sig,
        );

        VerificationSession::new(bundle, ArtifactLayout::new(root), "KarpalVerify")
    }

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
                smt_output: Some(SmtOutput {
                    status: Some(ExecutionStatus::Unsat),
                    model: None,
                    reason_unknown: None,
                }),
                lean_output: (plan.kind == crate::CommandKind::Lean).then(|| LeanOutput {
                    diagnostics: vec![LeanDiagnostic {
                        file: Some("lean/KarpalVerify.lean".into()),
                        line: Some(4),
                        column: Some(2),
                        severity: "warning".into(),
                        message: "declaration uses sorry: associativity".into(),
                    }],
                    theorem_hits: vec!["associativity".into()],
                }),
            }
        }
    }

    #[test]
    fn dry_run_report_orchestrates_artifacts_and_report() {
        let session = sample_session(Path::new("target/karpal-verify-session-dry-run"));
        let report = session.dry_run_report();

        assert_eq!(report.obligation_count(), 3);
        assert_eq!(report.root, "target/karpal-verify-session-dry-run");
        assert!(
            report
                .obligations
                .iter()
                .all(|entry| entry.status() == Some(ExecutionStatus::DryRun))
        );
    }

    #[test]
    fn verify_report_builds_artifacts_runs_plans_and_returns_report() {
        let temp = std::env::temp_dir().join("karpal_verify_session_report_test");
        if temp.exists() {
            let _ = fs::remove_dir_all(&temp);
        }

        let session = sample_session(&temp);
        let report = session
            .verify_report(&SuccessRunner)
            .expect("session verification should succeed");
        assert!(report.is_success());
        assert!(temp.join("smt").exists());
        assert!(temp.join("lean").exists());

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn verify_with_ci_outputs_writes_reports_beside_artifacts() {
        let temp = std::env::temp_dir().join("karpal_verify_session_ci_output_test");
        if temp.exists() {
            let _ = fs::remove_dir_all(&temp);
        }

        let output = sample_session(&temp)
            .with_report_stem("summary")
            .verify_with_ci_outputs(&SuccessRunner)
            .expect("ci outputs should be written");

        assert!(output.report.is_success());
        assert!(Path::new(&output.report_files.json_path).exists());
        assert!(Path::new(&output.report_files.markdown_path).exists());
        assert!(output.report_files.json_path.ends_with("summary.json"));
        assert!(output.report_files.markdown_path.ends_with("summary.md"));

        let _ = fs::remove_dir_all(&temp);
    }
}
