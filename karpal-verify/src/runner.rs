use crate::{
    Certificate, CommandKind, InvocationPlan, LeanCertificate, SmtCertificate, VerificationBackend,
};

#[cfg(feature = "std")]
use std::process::Command;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

/// Outcome of an external verification run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    Success,
    Failure,
    Sat,
    Unsat,
    Unknown,
    DryRun,
}

/// Parsed SMT output details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmtOutput {
    pub status: Option<ExecutionStatus>,
    pub model: Option<String>,
    pub reason_unknown: Option<String>,
}

/// One parsed Lean diagnostic line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanDiagnostic {
    pub file: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub severity: String,
    pub message: String,
    pub theorem_hits: Vec<String>,
}

/// Parsed Lean output details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanOutput {
    pub diagnostics: Vec<LeanDiagnostic>,
    pub theorem_hits: Vec<String>,
}

/// Backend-specific verification policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerificationPolicy {
    pub kind: CommandKind,
    pub success_status: ExecutionStatus,
    pub witness_suffix: &'static str,
}

impl VerificationPolicy {
    pub fn for_kind(kind: CommandKind) -> Self {
        match kind {
            CommandKind::Smt => Self {
                kind,
                success_status: ExecutionStatus::Unsat,
                witness_suffix: "unsat",
            },
            CommandKind::Lean => Self {
                kind,
                success_status: ExecutionStatus::Success,
                witness_suffix: "ok",
            },
        }
    }

    pub fn accepts(self, status: ExecutionStatus) -> bool {
        status == self.success_status
    }
}

/// Result captured from a verifier invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionResult {
    pub plan: InvocationPlan,
    pub status: ExecutionStatus,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub backend_version: Option<String>,
    pub smt_output: Option<SmtOutput>,
    pub lean_output: Option<LeanOutput>,
}

impl ExecutionResult {
    pub fn verification_policy(&self) -> VerificationPolicy {
        VerificationPolicy::for_kind(self.plan.kind)
    }

    pub fn is_success(&self) -> bool {
        self.verification_policy().accepts(self.status)
    }

    pub fn certificate_for_obligation(&self, obligation: &str) -> Option<Certificate> {
        if !self.is_success() {
            return None;
        }

        let backend = match self.plan.kind {
            CommandKind::Smt => SmtCertificate::NAME,
            CommandKind::Lean => LeanCertificate::NAME,
        };

        let witness = format!(
            "{}:{}",
            self.plan.executable,
            self.verification_policy().witness_suffix
        );

        let artifact_path = self.plan.input_files.first().cloned();
        let mut cert = Certificate::new(backend, obligation, witness);
        if let Some(version) = &self.backend_version {
            cert = cert.with_backend_version(version.clone());
        }
        if let Some(path) = artifact_path {
            cert = cert.with_artifact_path(path);
        }
        Some(cert)
    }
}

/// Runner abstraction for dry-run and local-process verification.
pub trait VerifierRunner {
    fn run(&self, plan: &InvocationPlan) -> ExecutionResult;

    fn run_all(&self, plans: &[InvocationPlan]) -> Vec<ExecutionResult> {
        plans.iter().map(|plan| self.run(plan)).collect()
    }
}

/// Dry-run runner that never spawns processes.
pub struct DryRunner;

impl VerifierRunner for DryRunner {
    fn run(&self, plan: &InvocationPlan) -> ExecutionResult {
        ExecutionResult {
            plan: plan.clone(),
            status: ExecutionStatus::DryRun,
            stdout: plan.render_shell(),
            stderr: String::new(),
            exit_code: None,
            backend_version: None,
            smt_output: None,
            lean_output: None,
        }
    }
}

/// Local process runner using `std::process::Command`.
#[cfg(feature = "std")]
pub struct LocalProcessRunner;

#[cfg(feature = "std")]
impl VerifierRunner for LocalProcessRunner {
    fn run(&self, plan: &InvocationPlan) -> ExecutionResult {
        let mut command = Command::new(&plan.executable);
        command.args(&plan.args);
        if let Some(dir) = &plan.working_directory {
            command.current_dir(dir);
        }

        let backend_version = probe_backend_version(&plan.executable);

        match command.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                let smt_output = match plan.kind {
                    CommandKind::Smt => Some(parse_smt_output(&stdout)),
                    CommandKind::Lean => None,
                };
                let lean_output = match plan.kind {
                    CommandKind::Smt => None,
                    CommandKind::Lean => Some(parse_lean_output(&stdout, &stderr)),
                };
                let status = classify_status(plan.kind, output.status.success(), &stdout, &stderr);
                ExecutionResult {
                    plan: plan.clone(),
                    status,
                    stdout,
                    stderr,
                    exit_code: output.status.code(),
                    backend_version,
                    smt_output,
                    lean_output,
                }
            }
            Err(err) => ExecutionResult {
                plan: plan.clone(),
                status: ExecutionStatus::Failure,
                stdout: String::new(),
                stderr: err.to_string(),
                exit_code: None,
                backend_version,
                smt_output: None,
                lean_output: None,
            },
        }
    }
}

fn classify_status(
    kind: CommandKind,
    process_success: bool,
    stdout: &str,
    stderr: &str,
) -> ExecutionStatus {
    match kind {
        CommandKind::Smt => parse_smt_output(stdout)
            .status
            .unwrap_or(if process_success {
                ExecutionStatus::Success
            } else {
                ExecutionStatus::Failure
            }),
        CommandKind::Lean => {
            let parsed = parse_lean_output(stdout, stderr);
            if process_success && parsed.error_count() == 0 {
                ExecutionStatus::Success
            } else {
                ExecutionStatus::Failure
            }
        }
    }
}

/// Parse SMT solver output into structured details.
pub fn parse_smt_output(stdout: &str) -> SmtOutput {
    let mut status = None;
    let mut model_lines = Vec::new();
    let mut reason_unknown = None;
    let mut capture_model = false;

    for line in stdout.lines() {
        let trimmed = line.trim();
        match trimmed {
            "sat" => {
                status = Some(ExecutionStatus::Sat);
                capture_model = true;
                continue;
            }
            "unsat" => {
                status = Some(ExecutionStatus::Unsat);
                continue;
            }
            "unknown" => {
                status = Some(ExecutionStatus::Unknown);
                continue;
            }
            _ => {}
        }

        if let Some(rest) = trimmed.strip_prefix("(:reason-unknown") {
            reason_unknown = Some(
                rest.trim()
                    .trim_end_matches(')')
                    .trim()
                    .trim_matches('"')
                    .to_string(),
            );
            continue;
        }

        if capture_model && !trimmed.is_empty() {
            model_lines.push(trimmed.to_string());
        }
    }

    SmtOutput {
        status,
        model: (!model_lines.is_empty()).then(|| model_lines.join("\n")),
        reason_unknown,
    }
}

/// Parse the first SMT solver status token from stdout.
pub fn parse_smt_status(stdout: &str) -> Option<ExecutionStatus> {
    parse_smt_output(stdout).status
}

impl LeanOutput {
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == "error")
            .count()
    }

    pub fn theorem_diagnostics<'a>(&'a self, theorem_name: &str) -> Vec<&'a LeanDiagnostic> {
        self.diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic
                    .theorem_hits
                    .iter()
                    .any(|hit| hit == theorem_name)
            })
            .collect()
    }

    pub fn has_theorem_failure(&self, theorem_name: &str) -> bool {
        self.theorem_hits.iter().any(|hit| hit == theorem_name)
            || self.diagnostics.iter().any(|diagnostic| {
                diagnostic
                    .theorem_hits
                    .iter()
                    .any(|hit| hit == theorem_name)
            })
    }
}

/// Parse Lean stdout/stderr into structured diagnostics and theorem references.
pub fn parse_lean_output(stdout: &str, stderr: &str) -> LeanOutput {
    let mut diagnostics = Vec::new();
    let mut theorem_hits = Vec::new();

    for line in stdout.lines().chain(stderr.lines()) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(diagnostic) = parse_lean_diagnostic_line(trimmed) {
            theorem_hits.extend(diagnostic.theorem_hits.iter().cloned());
            diagnostics.push(diagnostic);
            continue;
        }

        theorem_hits.extend(extract_theorem_hits(trimmed));
    }

    theorem_hits.sort();
    theorem_hits.dedup();

    LeanOutput {
        diagnostics,
        theorem_hits,
    }
}

fn parse_lean_diagnostic_line(line: &str) -> Option<LeanDiagnostic> {
    let (location, rest) = line.split_once(": ")?;
    let severity = ["error", "warning", "info"]
        .into_iter()
        .find(|severity| rest.starts_with(severity))?;
    let message = rest[severity.len()..]
        .trim_start_matches(':')
        .trim()
        .to_string();
    let theorem_hits = extract_theorem_hits(&message);

    let mut location_parts = location.split(':');
    let file = location_parts.next()?.to_string();
    let line_num = location_parts.next()?.parse().ok();
    let column = location_parts.next()?.parse().ok();

    Some(LeanDiagnostic {
        file: Some(file),
        line: line_num,
        column,
        severity: severity.into(),
        message,
        theorem_hits,
    })
}

fn extract_theorem_hits(text: &str) -> Vec<String> {
    fn normalize(token: &str) -> String {
        token
            .trim_matches(|c: char| matches!(c, '`' | '"' | '\''))
            .trim_end_matches(':')
            .to_string()
    }

    let tokens = text
        .split(|c: char| c.is_whitespace() || matches!(c, ',' | ';' | '(' | ')' | '[' | ']'))
        .map(normalize)
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();

    let mut hits = Vec::new();

    for window in tokens.windows(2) {
        if let [head, candidate] = window
            && matches!(head.as_str(), "theorem" | "declaration" | "sorry")
            && candidate
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        {
            hits.push(candidate.clone());
        }
    }

    if hits.is_empty() {
        for token in tokens {
            if token
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
                && token.contains('_')
            {
                hits.push(token);
            }
        }
    }

    hits.sort();
    hits.dedup();
    hits
}

#[cfg(feature = "std")]
fn probe_backend_version(executable: &str) -> Option<String> {
    let probes = [["--version"], ["-version"]];
    for args in probes {
        if let Ok(output) = Command::new(executable).args(args).output()
            && output.status.success()
        {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !text.is_empty() {
                return Some(text.lines().next().unwrap_or_default().to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_plan(kind: CommandKind) -> InvocationPlan {
        InvocationPlan {
            kind,
            executable: "tool".into(),
            args: vec!["input".into()],
            working_directory: None,
            input_files: vec!["input".into()],
        }
    }

    #[test]
    fn parses_smt_statuses() {
        assert_eq!(
            parse_smt_status("unsat\n(model ...)"),
            Some(ExecutionStatus::Unsat)
        );
        assert_eq!(parse_smt_status("sat"), Some(ExecutionStatus::Sat));
        assert_eq!(parse_smt_status("unknown"), Some(ExecutionStatus::Unknown));
        assert_eq!(parse_smt_status("noise"), None);

        let parsed = parse_smt_output("sat\n(model\n  (define-fun x () Int 1)\n)");
        assert_eq!(parsed.status, Some(ExecutionStatus::Sat));
        assert!(parsed.model.as_deref().unwrap().contains("define-fun x"));

        let parsed = parse_smt_output("unknown\n(:reason-unknown \"incomplete\")");
        assert_eq!(parsed.reason_unknown.as_deref(), Some("incomplete"));
    }

    #[test]
    fn dry_runner_returns_dry_run_result() {
        let result = DryRunner.run(&sample_plan(CommandKind::Lean));
        assert_eq!(result.status, ExecutionStatus::DryRun);
        assert!(result.stdout.contains("tool input"));
    }

    #[test]
    fn successful_smt_result_can_yield_certificate() {
        let result = ExecutionResult {
            plan: sample_plan(CommandKind::Smt),
            status: ExecutionStatus::Unsat,
            stdout: "unsat".into(),
            stderr: String::new(),
            exit_code: Some(0),
            backend_version: Some("Z3 4.13.0".into()),
            smt_output: Some(parse_smt_output("unsat")),
            lean_output: None,
        };
        let cert = result
            .certificate_for_obligation("karpal-core::Semigroup for i32 [associativity]")
            .expect("successful result should yield certificate");
        assert_eq!(cert.backend, "smtlib2");
        assert_eq!(cert.artifact_path.as_deref(), Some("input"));
        assert_eq!(cert.backend_version.as_deref(), Some("Z3 4.13.0"));
    }

    #[test]
    fn verification_policy_is_backend_specific() {
        assert!(VerificationPolicy::for_kind(CommandKind::Smt).accepts(ExecutionStatus::Unsat));
        assert!(!VerificationPolicy::for_kind(CommandKind::Smt).accepts(ExecutionStatus::Success));
        assert!(VerificationPolicy::for_kind(CommandKind::Lean).accepts(ExecutionStatus::Success));
        assert!(!VerificationPolicy::for_kind(CommandKind::Lean).accepts(ExecutionStatus::Unsat));
    }

    #[test]
    fn parses_lean_diagnostics_and_theorem_hits() {
        let parsed = parse_lean_output(
            "",
            "lean/KarpalVerify.lean:7:2: error: unsolved goals in theorem associativity\nlean/KarpalVerify.lean:12:4: warning: declaration uses sorry: left_inverse",
        );

        assert_eq!(parsed.error_count(), 1);
        assert_eq!(parsed.diagnostics.len(), 2);
        assert_eq!(parsed.diagnostics[0].line, Some(7));
        assert_eq!(
            parsed.diagnostics[0].theorem_hits,
            vec!["associativity".to_string()]
        );
        assert!(parsed.theorem_hits.iter().any(|hit| hit == "associativity"));
        assert!(parsed.theorem_hits.iter().any(|hit| hit == "left_inverse"));
        assert_eq!(parsed.theorem_diagnostics("associativity").len(), 1);
        assert!(parsed.has_theorem_failure("left_inverse"));
    }
}
