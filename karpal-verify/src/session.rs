use crate::{
    ArtifactLayout, DryRunner, LeanConfig, LeanManifestReportFiles, LocalProcessRunner,
    ObligationBundle, SmtConfig, VerificationReport, VerifierRunner, dry_run_bundle_artifacts,
    dry_run_report, execute_report, write_bundle_artifacts,
};
#[cfg(feature = "amari")]
use crate::{StatisticalVerification, ThreeTierVerificationReport, three_tier_report};
use std::{fs, io, path::Path, string::String};

/// Schema version for CI-oriented verification sidecar/report-file metadata.
pub const VERIFICATION_SIDECAR_SCHEMA_VERSION: &str = "1";

/// Default file stem used for CI-oriented verification summaries.
pub const DEFAULT_REPORT_STEM: &str = "verification-report";

/// Paths written for CI-oriented report summaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportFiles {
    pub json_path: String,
    pub markdown_path: String,
    pub lean_diagnostics_json_path: Option<String>,
    pub lean_manifest_path: Option<String>,
    pub three_tier_json_path: Option<String>,
    pub three_tier_markdown_path: Option<String>,
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
    #[cfg(feature = "amari")]
    statistical_verifications: Vec<StatisticalVerification>,
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
            #[cfg(feature = "amari")]
            statistical_verifications: Vec::new(),
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

    #[cfg(feature = "amari")]
    pub fn with_statistical_verification(mut self, verification: StatisticalVerification) -> Self {
        self.statistical_verifications.push(verification);
        self
    }

    #[cfg(feature = "amari")]
    pub fn with_statistical_verifications(
        mut self,
        verifications: impl IntoIterator<Item = StatisticalVerification>,
    ) -> Self {
        self.statistical_verifications.extend(verifications);
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
        let artifacts = write_bundle_artifacts(
            &self.bundle,
            &self.layout,
            &self.lean_module_name,
            &self.smt,
            &self.lean,
        )?;
        let report = execute_report(&self.bundle, &artifacts, runner);
        let report_files = write_report_files(
            &self.layout.root,
            &self.report_stem,
            &self.bundle,
            &report,
            artifacts.lean_manifest.as_ref(),
            #[cfg(feature = "amari")]
            &self.statistical_verifications,
        )?;
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
    bundle: &ObligationBundle,
    report: &VerificationReport,
    lean_manifest: Option<&crate::artifact::LeanManifest>,
    #[cfg(feature = "amari")] statistical_verifications: &[StatisticalVerification],
) -> io::Result<ReportFiles> {
    #[cfg(not(feature = "amari"))]
    let _ = bundle;
    let root = root.as_ref();
    fs::create_dir_all(root)?;

    let lean_manifest_path = report.lean_module.as_ref().map(|module| {
        path_to_string(
            &root
                .join("lean")
                .join(format!("{}.manifest.json", module.module_name)),
        )
    });

    let lean_diagnostics_json_path = report
        .lean_module
        .as_ref()
        .map(|_| -> io::Result<String> {
            let path = root.join(format!("{report_stem}.lean-diagnostics.json"));
            fs::write(&path, render_lean_diagnostics_json(report))?;
            Ok(path_to_string(&path))
        })
        .transpose()?;

    #[cfg(feature = "amari")]
    let three_tier = (!statistical_verifications.is_empty())
        .then(|| three_tier_report(bundle, statistical_verifications, Some(report)));
    #[cfg(feature = "amari")]
    let three_tier_json_path = if let Some(three_tier) = &three_tier {
        let path = root.join(format!("{report_stem}.three-tier.json"));
        fs::write(
            &path,
            render_three_tier_json_with_links(three_tier, report_stem, root),
        )?;
        Some(path_to_string(&path))
    } else {
        None
    };
    #[cfg(feature = "amari")]
    let three_tier_markdown_path = if let Some(three_tier) = &three_tier {
        let path = root.join(format!("{report_stem}.three-tier.md"));
        fs::write(
            &path,
            render_three_tier_markdown_with_links(three_tier, report_stem, root),
        )?;
        Some(path_to_string(&path))
    } else {
        None
    };
    #[cfg(not(feature = "amari"))]
    let three_tier_json_path = None;
    #[cfg(not(feature = "amari"))]
    let three_tier_markdown_path = None;

    let json_path = root.join(format!("{report_stem}.json"));
    let markdown_path = root.join(format!("{report_stem}.md"));
    let report_files = ReportFiles {
        json_path: path_to_string(&json_path),
        markdown_path: path_to_string(&markdown_path),
        lean_diagnostics_json_path,
        lean_manifest_path,
        three_tier_json_path,
        three_tier_markdown_path,
    };
    fs::write(
        &json_path,
        render_report_json_with_links(report, &report_files),
    )?;
    fs::write(
        &markdown_path,
        render_report_markdown_with_links(report, &report_files),
    )?;
    if let (Some(path), Some(manifest)) = (&report_files.lean_manifest_path, lean_manifest) {
        write_lean_manifest_with_report_links(path, manifest, &report_files)?;
    }

    Ok(report_files)
}

fn render_report_json_with_links(report: &VerificationReport, files: &ReportFiles) -> String {
    let mut json = report.to_json();
    if json.ends_with('}') {
        json.pop();
        json.push_str(",\"report_files\":{");
        json.push_str(&format!(
            "\"schema_version\":\"{}\",\"json_path\":\"{}\",\"markdown_path\":\"{}\"",
            VERIFICATION_SIDECAR_SCHEMA_VERSION,
            escape_json(&files.json_path),
            escape_json(&files.markdown_path)
        ));
        if let Some(path) = &files.lean_diagnostics_json_path {
            json.push_str(&format!(
                ",\"lean_diagnostics_json_path\":\"{}\"",
                escape_json(path)
            ));
        }
        if let Some(path) = &files.lean_manifest_path {
            json.push_str(&format!(
                ",\"lean_manifest_path\":\"{}\"",
                escape_json(path)
            ));
        }
        if let Some(path) = &files.three_tier_json_path {
            json.push_str(&format!(
                ",\"three_tier_json_path\":\"{}\"",
                escape_json(path)
            ));
        }
        if let Some(path) = &files.three_tier_markdown_path {
            json.push_str(&format!(
                ",\"three_tier_markdown_path\":\"{}\"",
                escape_json(path)
            ));
        }
        json.push_str("}}");
    }
    json
}

fn render_report_markdown_with_links(report: &VerificationReport, files: &ReportFiles) -> String {
    let mut markdown = report.to_markdown();
    markdown.push_str("\nReport files:\n");
    markdown.push_str(&format!(
        "- Schema version: `{}`\n",
        VERIFICATION_SIDECAR_SCHEMA_VERSION
    ));
    markdown.push_str(&format!("- JSON: `{}`\n", files.json_path));
    markdown.push_str(&format!("- Markdown: `{}`\n", files.markdown_path));
    if let Some(path) = &files.lean_diagnostics_json_path {
        markdown.push_str(&format!("- Lean diagnostics JSON: `{}`\n", path));
    }
    if let Some(path) = &files.lean_manifest_path {
        markdown.push_str(&format!("- Lean manifest: `{}`\n", path));
    }
    if let Some(path) = &files.three_tier_json_path {
        markdown.push_str(&format!("- Three-tier JSON: `{}`\n", path));
    }
    if let Some(path) = &files.three_tier_markdown_path {
        markdown.push_str(&format!("- Three-tier Markdown: `{}`\n", path));
    }
    markdown
}

#[cfg(feature = "amari")]
fn render_three_tier_json_with_links(
    report: &ThreeTierVerificationReport,
    report_stem: &str,
    root: &Path,
) -> String {
    let mut json = report.to_json();
    if json.ends_with('}') {
        json.pop();
        json.push_str(",\"report_files\":{");
        json.push_str(&format!(
            "\"schema_version\":\"{}\",\"json_path\":\"{}\",\"markdown_path\":\"{}\"",
            VERIFICATION_SIDECAR_SCHEMA_VERSION,
            escape_json(&path_to_string(
                &root.join(format!("{report_stem}.three-tier.json"))
            )),
            escape_json(&path_to_string(
                &root.join(format!("{report_stem}.three-tier.md"))
            ))
        ));
        json.push_str("}}");
    }
    json
}

#[cfg(feature = "amari")]
fn render_three_tier_markdown_with_links(
    report: &ThreeTierVerificationReport,
    report_stem: &str,
    root: &Path,
) -> String {
    let mut markdown = report.to_markdown();
    markdown.push_str("\nReport files:\n");
    markdown.push_str(&format!(
        "- Schema version: `{}`\n",
        VERIFICATION_SIDECAR_SCHEMA_VERSION
    ));
    markdown.push_str(&format!(
        "- JSON: `{}`\n",
        path_to_string(&root.join(format!("{report_stem}.three-tier.json")))
    ));
    markdown.push_str(&format!(
        "- Markdown: `{}`\n",
        path_to_string(&root.join(format!("{report_stem}.three-tier.md")))
    ));
    markdown
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn write_lean_manifest_with_report_links(
    manifest_path: &str,
    manifest: &crate::artifact::LeanManifest,
    files: &ReportFiles,
) -> io::Result<()> {
    let manifest = manifest.clone().with_report_files({
        let report_files =
            LeanManifestReportFiles::new(files.json_path.clone(), files.markdown_path.clone());
        match &files.lean_diagnostics_json_path {
            Some(path) => report_files.with_lean_diagnostics_json_path(path.clone()),
            None => report_files,
        }
    });
    fs::write(manifest_path, manifest.to_json())
}

fn render_lean_diagnostics_json(report: &VerificationReport) -> String {
    fn esc(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    }

    let module = match &report.lean_module {
        Some(module) => module,
        None => return "null".into(),
    };

    let obligation_entries = report
        .obligations
        .iter()
        .filter(|obligation| {
            obligation.lean_theorem_ref.is_some() || !obligation.lean_diagnostics.is_empty()
        })
        .map(|obligation| {
            let diagnostics = obligation
                .lean_diagnostics
                .iter()
                .map(|diagnostic| format!("\"{}\"", esc(diagnostic)))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"obligation_name\":\"{}\",\"theorem_ref\":{},\"diagnostics\":[{}]}}",
                esc(&obligation.obligation_name),
                obligation
                    .lean_theorem_ref
                    .as_ref()
                    .map(|theorem_ref| format!("\"{}\"", esc(theorem_ref)))
                    .unwrap_or_else(|| "null".into()),
                diagnostics
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let module_diagnostics = module
        .diagnostics
        .iter()
        .map(|diagnostic| format!("\"{}\"", esc(diagnostic)))
        .collect::<Vec<_>>()
        .join(",");
    let theorem_failures = module
        .theorem_failures
        .iter()
        .map(|failure| format!("\"{}\"", esc(failure)))
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\"schema_version\":\"{}\",\"module_name\":\"{}\",\"module_diagnostics\":[{}],\"theorem_failures\":[{}],\"obligations\":[{}]}}",
        VERIFICATION_SIDECAR_SCHEMA_VERSION,
        esc(&module.module_name),
        module_diagnostics,
        theorem_failures,
        obligation_entries
    )
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
    #[cfg(feature = "amari")]
    use crate::{StatisticalBound, verify_rare_event};

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
                        theorem_hits: vec!["associativity".into()],
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
        assert_eq!(
            report
                .lean_module
                .as_ref()
                .map(|module| module.theorem_failures.clone()),
            Some(vec!["KarpalVerify.associativity".into()])
        );

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
        assert!(
            Path::new(
                output
                    .report_files
                    .lean_diagnostics_json_path
                    .as_deref()
                    .expect("lean diagnostics sidecar should be written")
            )
            .exists()
        );
        assert!(
            Path::new(
                output
                    .report_files
                    .lean_manifest_path
                    .as_deref()
                    .expect("lean manifest path should be recorded")
            )
            .exists()
        );
        assert!(output.report_files.json_path.ends_with("summary.json"));
        assert!(output.report_files.markdown_path.ends_with("summary.md"));
        #[cfg(not(feature = "amari"))]
        {
            assert!(output.report_files.three_tier_json_path.is_none());
            assert!(output.report_files.three_tier_markdown_path.is_none());
        }
        assert!(
            output
                .report_files
                .lean_diagnostics_json_path
                .as_deref()
                .unwrap()
                .ends_with("summary.lean-diagnostics.json")
        );
        assert!(
            output
                .report_files
                .lean_manifest_path
                .as_deref()
                .unwrap()
                .ends_with("lean/KarpalVerify.manifest.json")
        );

        let json = fs::read_to_string(&output.report_files.json_path)
            .expect("summary json should be readable");
        let markdown = fs::read_to_string(&output.report_files.markdown_path)
            .expect("summary markdown should be readable");
        let sidecar = fs::read_to_string(
            output
                .report_files
                .lean_diagnostics_json_path
                .as_deref()
                .expect("lean diagnostics sidecar path should be present"),
        )
        .expect("lean diagnostics sidecar should be readable");
        let manifest = fs::read_to_string(
            output
                .report_files
                .lean_manifest_path
                .as_deref()
                .expect("lean manifest path should be present"),
        )
        .expect("lean manifest should be readable");
        assert!(json.contains("\"schema_version\":\"1\""));
        assert!(json.contains("\"report_files\""));
        assert!(json.contains("\"lean_manifest_path\""));
        assert!(json.contains("\"lean_diagnostics_json_path\""));
        assert!(json.contains("\"schema_version\":\"1\",\"json_path\""));
        assert!(markdown.contains("Report files:"));
        assert!(markdown.contains("Schema version: `1`"));
        assert!(markdown.contains("Lean diagnostics JSON"));
        assert!(markdown.contains("Lean manifest"));
        assert!(sidecar.contains("\"schema_version\":\"1\""));
        assert!(manifest.contains("\"schema_version\":\"1\""));
        assert!(manifest.contains("\"report_files\""));
        assert!(manifest.contains("\"json_path\""));
        assert!(manifest.contains("\"markdown_path\""));
        assert!(manifest.contains("\"lean_diagnostics_json_path\""));

        let _ = fs::remove_dir_all(&temp);
    }

    #[cfg(feature = "amari")]
    #[test]
    fn verify_with_ci_outputs_writes_three_tier_sidecars_when_statistical_evidence_exists() {
        let temp = std::env::temp_dir().join("karpal_verify_session_three_tier_test");
        if temp.exists() {
            let _ = fs::remove_dir_all(&temp);
        }

        let bundle = sample_session(&temp).bundle().clone();
        let statistical = verify_rare_event(
            &bundle.obligations()[0],
            &StatisticalBound::new(0.05).with_samples(128),
            || false,
        );

        let output = sample_session(&temp)
            .with_report_stem("summary")
            .with_statistical_verification(statistical)
            .verify_with_ci_outputs(&DryRunner)
            .expect("ci outputs should be written");

        let three_tier_json_path = output
            .report_files
            .three_tier_json_path
            .as_deref()
            .expect("three-tier json should be written");
        let three_tier_markdown_path = output
            .report_files
            .three_tier_markdown_path
            .as_deref()
            .expect("three-tier markdown should be written");
        assert!(Path::new(three_tier_json_path).exists());
        assert!(Path::new(three_tier_markdown_path).exists());
        assert!(three_tier_json_path.ends_with("summary.three-tier.json"));
        assert!(three_tier_markdown_path.ends_with("summary.three-tier.md"));

        let json = fs::read_to_string(three_tier_json_path).expect("three-tier json readable");
        let markdown =
            fs::read_to_string(three_tier_markdown_path).expect("three-tier markdown readable");
        let main_json =
            fs::read_to_string(&output.report_files.json_path).expect("main report json readable");
        let main_markdown = fs::read_to_string(&output.report_files.markdown_path)
            .expect("main report markdown readable");

        assert!(json.contains("\"impossible_count\":1"));
        assert!(json.contains("\"external_count\":2"));
        assert!(json.contains("\"report_files\""));
        assert!(markdown.contains("Three-Tier Verification Report"));
        assert!(markdown.contains("Schema version: `1`"));
        assert!(main_json.contains("\"three_tier_json_path\""));
        assert!(main_json.contains("\"three_tier_markdown_path\""));
        assert!(main_markdown.contains("Three-tier JSON"));
        assert!(main_markdown.contains("Three-tier Markdown"));

        let _ = fs::remove_dir_all(&temp);
    }
}
