use crate::{
    InvocationPlan, LeanConfig, LeanExport, LeanProject, ObligationBundle, SmtConfig,
    export_lean_bundle_structured, export_smt_bundle,
};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{
    fs,
    path::{Path, PathBuf},
    string::String,
    vec::Vec,
};

/// Written artifact metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactRecord {
    pub name: String,
    pub path: String,
}

/// Report file links attached back onto a generated Lean manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanManifestReportFiles {
    pub json_path: String,
    pub markdown_path: String,
    pub lean_diagnostics_json_path: Option<String>,
}

/// Lean package metadata serialized into the generated manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanManifestProject {
    pub package_name: String,
    pub toolchain: String,
    pub requires_mathlib: bool,
}

/// Lean import alias metadata serialized into the generated manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanManifestAlias {
    pub alias: String,
    pub target: String,
}

/// Lean prelude metadata serialized into the generated manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanManifestPrelude {
    pub imports: Vec<String>,
    pub aliases: Vec<LeanManifestAlias>,
}

/// Lean theorem metadata serialized into the generated manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanManifestTheorem {
    pub obligation_name: String,
    pub theorem_name: String,
    pub witness_ref: String,
    pub declaration_start_line: usize,
    pub declaration_end_line: usize,
}

/// Typed manifest model for generated Lean verification artifacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanManifest {
    pub module_name: String,
    pub project: LeanManifestProject,
    pub prelude: LeanManifestPrelude,
    pub theorems: Vec<LeanManifestTheorem>,
    pub report_files: Option<LeanManifestReportFiles>,
}

/// Result of preparing or writing a verification batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactBatch {
    pub root: String,
    pub records: Vec<ArtifactRecord>,
    pub plans: Vec<InvocationPlan>,
    pub lean_export: Option<LeanExport>,
    pub lean_project: Option<LeanProject>,
    pub lean_manifest: Option<LeanManifest>,
}

#[cfg(feature = "std")]
#[derive(Debug, Clone)]
pub struct ArtifactLayout {
    pub root: PathBuf,
    pub smt_dir: PathBuf,
    pub lean_dir: PathBuf,
}

#[cfg(feature = "std")]
impl ArtifactLayout {
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        Self {
            smt_dir: root.join("smt"),
            lean_dir: root.join("lean"),
            root,
        }
    }
}

impl LeanManifestReportFiles {
    pub fn new(json_path: impl Into<String>, markdown_path: impl Into<String>) -> Self {
        Self {
            json_path: json_path.into(),
            markdown_path: markdown_path.into(),
            lean_diagnostics_json_path: None,
        }
    }

    pub fn with_lean_diagnostics_json_path(mut self, path: impl Into<String>) -> Self {
        self.lean_diagnostics_json_path = Some(path.into());
        self
    }
}

impl LeanManifest {
    pub fn from_export(export: &LeanExport, project: &LeanProject) -> Self {
        Self {
            module_name: export.module_name.clone(),
            project: LeanManifestProject {
                package_name: project.package_name.clone(),
                toolchain: project.toolchain.clone(),
                requires_mathlib: project.requires_mathlib,
            },
            prelude: LeanManifestPrelude {
                imports: export
                    .prelude
                    .imports
                    .iter()
                    .map(|import| import.module.clone())
                    .collect(),
                aliases: export
                    .prelude
                    .aliases
                    .iter()
                    .map(|alias| LeanManifestAlias {
                        alias: alias.alias.clone(),
                        target: alias.target.clone(),
                    })
                    .collect(),
            },
            theorems: export
                .theorems
                .iter()
                .map(|theorem| LeanManifestTheorem {
                    obligation_name: theorem.obligation_name.clone(),
                    theorem_name: theorem.theorem_name.clone(),
                    witness_ref: theorem.witness_ref(&export.module_name),
                    declaration_start_line: theorem.declaration_start_line,
                    declaration_end_line: theorem.declaration_end_line,
                })
                .collect(),
            report_files: None,
        }
    }

    pub fn with_report_files(mut self, report_files: LeanManifestReportFiles) -> Self {
        self.report_files = Some(report_files);
        self
    }

    pub fn to_json(&self) -> String {
        fn esc(s: &str) -> String {
            s.replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
        }

        let import_entries = self
            .prelude
            .imports
            .iter()
            .map(|import| format!("\"{}\"", esc(import)))
            .collect::<Vec<_>>()
            .join(",");

        let alias_entries = self
            .prelude
            .aliases
            .iter()
            .map(|alias| {
                format!(
                    "{{\"alias\":\"{}\",\"target\":\"{}\"}}",
                    esc(&alias.alias),
                    esc(&alias.target)
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        let theorem_entries = self
            .theorems
            .iter()
            .map(|theorem| {
                format!(
                    "{{\"obligation_name\":\"{}\",\"theorem_name\":\"{}\",\"witness_ref\":\"{}\",\"declaration_start_line\":{},\"declaration_end_line\":{}}}",
                    esc(&theorem.obligation_name),
                    esc(&theorem.theorem_name),
                    esc(&theorem.witness_ref),
                    theorem.declaration_start_line,
                    theorem.declaration_end_line
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        let report_files = self
            .report_files
            .as_ref()
            .map(|report_files| {
                let mut json = format!(
                    "\"report_files\":{{\"json_path\":\"{}\",\"markdown_path\":\"{}\"",
                    esc(&report_files.json_path),
                    esc(&report_files.markdown_path)
                );
                if let Some(path) = &report_files.lean_diagnostics_json_path {
                    json.push_str(&format!(
                        ",\"lean_diagnostics_json_path\":\"{}\"",
                        esc(path)
                    ));
                }
                json.push('}');
                json
            })
            .unwrap_or_default();

        let mut json = format!(
            "{{\"module_name\":\"{}\",\"project\":{{\"package_name\":\"{}\",\"toolchain\":\"{}\",\"requires_mathlib\":{}}},\"prelude\":{{\"imports\":[{}],\"aliases\":[{}]}},\"theorems\":[{}]",
            esc(&self.module_name),
            esc(&self.project.package_name),
            esc(&self.project.toolchain),
            self.project.requires_mathlib,
            import_entries,
            alias_entries,
            theorem_entries
        );
        if !report_files.is_empty() {
            json.push(',');
            json.push_str(&report_files);
        }
        json.push('}');
        json
    }
}

#[cfg(feature = "std")]
pub fn write_bundle_artifacts(
    bundle: &ObligationBundle,
    layout: &ArtifactLayout,
    lean_module_name: &str,
    smt: &SmtConfig,
    lean: &LeanConfig,
) -> std::io::Result<ArtifactBatch> {
    fs::create_dir_all(&layout.smt_dir)?;
    fs::create_dir_all(&layout.lean_dir)?;

    let mut records = Vec::new();
    let mut plans = Vec::new();

    for (name, script) in export_smt_bundle(bundle) {
        let path = layout.smt_dir.join(format!("{name}.smt2"));
        fs::write(&path, script)?;
        plans.push(InvocationPlan::smt(smt, &path));
        records.push(ArtifactRecord {
            name,
            path: path_to_string(&path),
        });
    }

    let lean_export = export_lean_bundle_structured(lean_module_name, bundle);
    let lean_project = lean_export.project();
    let lean_manifest = LeanManifest::from_export(&lean_export, &lean_project);
    let lean_path = layout.lean_dir.join(format!("{lean_module_name}.lean"));
    fs::write(&lean_path, &lean_export.source)?;
    plans.push(InvocationPlan::lean(lean, &lean_path));
    records.push(ArtifactRecord {
        name: lean_module_name.into(),
        path: path_to_string(&lean_path),
    });

    let manifest_path = layout
        .lean_dir
        .join(format!("{lean_module_name}.manifest.json"));
    fs::write(&manifest_path, lean_manifest.to_json())?;
    records.push(ArtifactRecord {
        name: format!("{lean_module_name}_manifest"),
        path: path_to_string(&manifest_path),
    });

    let lakefile_path = layout.root.join("lakefile.lean");
    fs::write(&lakefile_path, lean_project.render_lakefile())?;
    records.push(ArtifactRecord {
        name: "lakefile".into(),
        path: path_to_string(&lakefile_path),
    });

    let toolchain_path = layout.root.join("lean-toolchain");
    fs::write(&toolchain_path, lean_project.render_toolchain())?;
    records.push(ArtifactRecord {
        name: "lean_toolchain".into(),
        path: path_to_string(&toolchain_path),
    });

    Ok(ArtifactBatch {
        root: path_to_string(&layout.root),
        records,
        plans,
        lean_export: Some(lean_export),
        lean_project: Some(lean_project),
        lean_manifest: Some(lean_manifest),
    })
}

#[cfg(feature = "std")]
pub fn dry_run_bundle_artifacts(
    bundle: &ObligationBundle,
    layout: &ArtifactLayout,
    lean_module_name: &str,
    smt: &SmtConfig,
    lean: &LeanConfig,
) -> ArtifactBatch {
    let mut records = Vec::new();
    let mut plans = Vec::new();

    for (name, _) in export_smt_bundle(bundle) {
        let path = layout.smt_dir.join(format!("{name}.smt2"));
        plans.push(InvocationPlan::smt(smt, &path));
        records.push(ArtifactRecord {
            name,
            path: path_to_string(&path),
        });
    }

    let lean_export = export_lean_bundle_structured(lean_module_name, bundle);
    let lean_project = lean_export.project();
    let lean_manifest = LeanManifest::from_export(&lean_export, &lean_project);
    let lean_path = layout.lean_dir.join(format!("{lean_module_name}.lean"));
    plans.push(InvocationPlan::lean(lean, &lean_path));
    records.push(ArtifactRecord {
        name: lean_module_name.into(),
        path: path_to_string(&lean_path),
    });

    let manifest_path = layout
        .lean_dir
        .join(format!("{lean_module_name}.manifest.json"));
    records.push(ArtifactRecord {
        name: format!("{lean_module_name}_manifest"),
        path: path_to_string(&manifest_path),
    });

    let lakefile_path = layout.root.join("lakefile.lean");
    records.push(ArtifactRecord {
        name: "lakefile".into(),
        path: path_to_string(&lakefile_path),
    });

    let toolchain_path = layout.root.join("lean-toolchain");
    records.push(ArtifactRecord {
        name: "lean_toolchain".into(),
        path: path_to_string(&toolchain_path),
    });

    ArtifactBatch {
        root: path_to_string(&layout.root),
        records,
        plans,
        lean_export: Some(lean_export),
        lean_project: Some(lean_project),
        lean_manifest: Some(lean_manifest),
    }
}

#[cfg(feature = "std")]
fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AlgebraicSignature, Origin, Sort};

    #[cfg(feature = "std")]
    #[test]
    fn dry_run_creates_expected_paths_and_plans() {
        let sig = AlgebraicSignature::monoid(Sort::Int, "combine", "e");
        let bundle = ObligationBundle::monoid("sum", Origin::new("karpal-core", "Sum<i32>"), &sig);
        let layout = ArtifactLayout::new("target/karpal-verify-test");
        let batch = dry_run_bundle_artifacts(
            &bundle,
            &layout,
            "KarpalVerify",
            &SmtConfig::default(),
            &LeanConfig::default().with_driver(crate::LeanDriver::LakeEnv),
        );

        assert_eq!(batch.records.len(), 7);
        assert_eq!(batch.plans.len(), 4);
        assert!(
            batch
                .records
                .iter()
                .any(|r| r.path.ends_with("KarpalVerify.lean"))
        );
        assert!(
            batch
                .records
                .iter()
                .any(|r| r.path.ends_with("KarpalVerify.manifest.json"))
        );
        assert_eq!(
            batch.lean_export.as_ref().unwrap().module_name,
            "KarpalVerify"
        );
        assert_eq!(
            batch.lean_project.as_ref().unwrap().package_name,
            "karpalverify"
        );
        assert_eq!(
            batch.lean_manifest.as_ref().unwrap().module_name,
            "KarpalVerify"
        );
        assert!(
            batch
                .records
                .iter()
                .any(|r| r.path.ends_with("lakefile.lean"))
        );
        assert!(
            batch
                .records
                .iter()
                .any(|r| r.path.ends_with("lean-toolchain"))
        );
        assert!(
            batch
                .plans
                .iter()
                .any(|plan| plan.kind == crate::CommandKind::Lean && plan.executable == "lake")
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn write_bundle_artifacts_writes_files() {
        let sig = AlgebraicSignature::semigroup(Sort::Int, "combine");
        let bundle =
            ObligationBundle::semigroup("sum", Origin::new("karpal-core", "Sum<i32>"), &sig);
        let temp = std::env::temp_dir().join("karpal_verify_artifacts_test");
        if temp.exists() {
            let _ = fs::remove_dir_all(&temp);
        }
        let layout = ArtifactLayout::new(&temp);

        let batch = write_bundle_artifacts(
            &bundle,
            &layout,
            "KarpalVerify",
            &SmtConfig::default(),
            &LeanConfig::default(),
        )
        .expect("artifact write should succeed");

        assert!(
            batch
                .records
                .iter()
                .all(|record| Path::new(&record.path).exists())
        );
        assert!(batch.lean_export.is_some());
        assert!(batch.lean_project.is_some());
        assert!(batch.lean_manifest.is_some());
        assert!(temp.join("lakefile.lean").exists());
        assert!(temp.join("lean-toolchain").exists());

        let _ = fs::remove_dir_all(&temp);
    }
}
