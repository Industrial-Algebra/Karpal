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

/// How Lean verification should be executed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeanDriver {
    Direct,
    LakeEnv,
}

/// Command-line configuration for Lean 4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanConfig {
    pub executable: String,
    pub args: Vec<String>,
    pub driver: LeanDriver,
    pub lake_executable: String,
    pub lake_args: Vec<String>,
}

impl Default for LeanConfig {
    fn default() -> Self {
        Self {
            executable: "lean".into(),
            args: Vec::new(),
            driver: LeanDriver::Direct,
            lake_executable: "lake".into(),
            lake_args: Vec::new(),
        }
    }
}

impl LeanConfig {
    pub fn new(executable: impl Into<String>) -> Self {
        Self {
            executable: executable.into(),
            ..Self::default()
        }
    }

    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn with_driver(mut self, driver: LeanDriver) -> Self {
        self.driver = driver;
        self
    }

    pub fn with_lake_executable(mut self, executable: impl Into<String>) -> Self {
        self.lake_executable = executable.into();
        self
    }

    pub fn with_lake_arg(mut self, arg: impl Into<String>) -> Self {
        self.lake_args.push(arg.into());
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
        match config.driver {
            LeanDriver::Direct => {
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
            LeanDriver::LakeEnv => {
                let lean_dir = module.parent().unwrap_or_else(|| Path::new("."));
                let root = lean_dir.parent().unwrap_or(lean_dir);
                let relative_module = module
                    .strip_prefix(root)
                    .unwrap_or(&module)
                    .to_string_lossy()
                    .into_owned();
                let mut args = config.lake_args.clone();
                args.push("env".into());
                args.push(config.executable.clone());
                args.extend(config.args.iter().cloned());
                args.push(relative_module);
                Self {
                    kind: CommandKind::Lean,
                    executable: config.lake_executable.clone(),
                    args,
                    working_directory: Some(path_to_string(root)),
                    input_files: vec![
                        path_to_string(&module),
                        path_to_string(&root.join("lakefile.lean")),
                        path_to_string(&root.join("lean-toolchain")),
                    ],
                }
            }
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

    #[cfg(feature = "std")]
    #[test]
    fn lean_project_plan_uses_lake_from_project_root() {
        let plan = InvocationPlan::lean(
            &LeanConfig::default().with_driver(LeanDriver::LakeEnv),
            std::path::PathBuf::from("target/verify/lean/KarpalVerify.lean"),
        );
        assert_eq!(plan.kind, CommandKind::Lean);
        assert_eq!(plan.executable, "lake");
        assert_eq!(plan.working_directory.as_deref(), Some("target/verify"));
        assert_eq!(plan.args, vec!["env", "lean", "lean/KarpalVerify.lean"]);
        assert!(
            plan.input_files
                .iter()
                .any(|path| path.ends_with("lakefile.lean"))
        );
        assert!(
            plan.input_files
                .iter()
                .any(|path| path.ends_with("lean-toolchain"))
        );
    }
}
