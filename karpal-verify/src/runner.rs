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

/// Result captured from a verifier invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionResult {
    pub plan: InvocationPlan,
    pub status: ExecutionStatus,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

impl ExecutionResult {
    pub fn is_success(&self) -> bool {
        matches!(
            self.status,
            ExecutionStatus::Success | ExecutionStatus::Unsat
        )
    }

    pub fn certificate_for_obligation(&self, obligation: &str) -> Option<Certificate> {
        if !self.is_success() {
            return None;
        }

        let backend = match self.plan.kind {
            CommandKind::Smt => SmtCertificate::NAME,
            CommandKind::Lean => LeanCertificate::NAME,
        };

        let witness = match self.plan.kind {
            CommandKind::Smt => format!("{}:unsat", self.plan.executable),
            CommandKind::Lean => format!("{}:ok", self.plan.executable),
        };

        let artifact_path = self.plan.input_files.first().cloned();
        let mut cert = Certificate::new(backend, obligation, witness);
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

        match command.output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                let status = classify_status(plan.kind, output.status.success(), &stdout);
                ExecutionResult {
                    plan: plan.clone(),
                    status,
                    stdout,
                    stderr,
                    exit_code: output.status.code(),
                }
            }
            Err(err) => ExecutionResult {
                plan: plan.clone(),
                status: ExecutionStatus::Failure,
                stdout: String::new(),
                stderr: err.to_string(),
                exit_code: None,
            },
        }
    }
}

fn classify_status(kind: CommandKind, process_success: bool, stdout: &str) -> ExecutionStatus {
    match kind {
        CommandKind::Smt => parse_smt_status(stdout).unwrap_or(if process_success {
            ExecutionStatus::Success
        } else {
            ExecutionStatus::Failure
        }),
        CommandKind::Lean => {
            if process_success {
                ExecutionStatus::Success
            } else {
                ExecutionStatus::Failure
            }
        }
    }
}

/// Parse the first SMT solver status token from stdout.
pub fn parse_smt_status(stdout: &str) -> Option<ExecutionStatus> {
    for line in stdout.lines() {
        match line.trim() {
            "sat" => return Some(ExecutionStatus::Sat),
            "unsat" => return Some(ExecutionStatus::Unsat),
            "unknown" => return Some(ExecutionStatus::Unknown),
            _ => {}
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
        };
        let cert = result
            .certificate_for_obligation("karpal-core::Semigroup for i32 [associativity]")
            .expect("successful result should yield certificate");
        assert_eq!(cert.backend, "smtlib2");
        assert_eq!(cert.artifact_path.as_deref(), Some("input"));
    }
}
