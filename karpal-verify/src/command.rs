#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{path::Path, string::String, vec::Vec};

/// External prover backend command kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKind {
    Smt,
    Lean,
}

/// Command-line configuration for an SMT solver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmtConfig {
    pub executable: String,
    pub args: Vec<String>,
}

impl Default for SmtConfig {
    fn default() -> Self {
        Self {
            executable: "z3".into(),
            args: vec!["-smt2".into()],
        }
    }
}

impl SmtConfig {
    pub fn new(executable: impl Into<String>) -> Self {
        Self {
            executable: executable.into(),
            args: Vec::new(),
        }
    }

    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }
}

/// Command-line configuration for Lean 4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanConfig {
    pub executable: String,
    pub args: Vec<String>,
}

impl Default for LeanConfig {
    fn default() -> Self {
        Self {
            executable: "lean".into(),
            args: Vec::new(),
        }
    }
}

impl LeanConfig {
    pub fn new(executable: impl Into<String>) -> Self {
        Self {
            executable: executable.into(),
            args: Vec::new(),
        }
    }

    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }
}

/// Dry-run invocation plan for an external tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvocationPlan {
    pub kind: CommandKind,
    pub executable: String,
    pub args: Vec<String>,
    pub working_directory: Option<String>,
    pub input_files: Vec<String>,
}

impl InvocationPlan {
    pub fn render_shell(&self) -> String {
        let joined = self
            .args
            .iter()
            .map(|arg| shell_escape(arg))
            .collect::<Vec<_>>()
            .join(" ");
        if joined.is_empty() {
            self.executable.clone()
        } else {
            format!("{} {}", self.executable, joined)
        }
    }
}

#[cfg(feature = "std")]
impl InvocationPlan {
    pub fn smt(config: &SmtConfig, script: impl AsRef<Path>) -> Self {
        let script = script.as_ref().to_path_buf();
        let mut args = config.args.clone();
        args.push(script.to_string_lossy().into_owned());
        Self {
            kind: CommandKind::Smt,
            executable: config.executable.clone(),
            args,
            working_directory: script.parent().map(path_to_string),
            input_files: vec![path_to_string(&script)],
        }
    }

    pub fn lean(config: &LeanConfig, module: impl AsRef<Path>) -> Self {
        let module = module.as_ref().to_path_buf();
        let mut args = config.args.clone();
        args.push(module.to_string_lossy().into_owned());
        Self {
            kind: CommandKind::Lean,
            executable: config.executable.clone(),
            args,
            working_directory: module.parent().map(path_to_string),
            input_files: vec![path_to_string(&module)],
        }
    }
}

#[cfg(feature = "std")]
fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn shell_escape(arg: &str) -> String {
    if arg
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '_' | '.' | ':'))
    {
        arg.to_string()
    } else {
        format!("'{}'", arg.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_render_quotes_complex_args() {
        let plan = InvocationPlan {
            kind: CommandKind::Lean,
            executable: "lean".into(),
            args: vec!["My File.lean".into()],
            working_directory: None,
            input_files: vec!["My File.lean".into()],
        };
        assert!(plan.render_shell().contains("'My File.lean'"));
    }

    #[cfg(feature = "std")]
    #[test]
    fn smt_plan_includes_script_path() {
        let plan = InvocationPlan::smt(
            &SmtConfig::default(),
            std::path::PathBuf::from("out/test.smt2"),
        );
        assert_eq!(plan.kind, CommandKind::Smt);
        assert!(plan.input_files[0].ends_with("test.smt2"));
    }
}
