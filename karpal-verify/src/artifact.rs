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

/// Result of preparing or writing a verification batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactBatch {
    pub root: String,
    pub records: Vec<ArtifactRecord>,
    pub plans: Vec<InvocationPlan>,
    pub lean_export: Option<LeanExport>,
    pub lean_project: Option<LeanProject>,
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
    fs::write(
        &manifest_path,
        render_lean_manifest_json(&lean_export, &lean_project),
    )?;
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
    }
}

#[cfg(feature = "std")]
fn render_lean_manifest_json(export: &LeanExport, project: &LeanProject) -> String {
    fn esc(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    }

    let import_entries = export
        .prelude
        .imports
        .iter()
        .map(|import| format!("\"{}\"", esc(&import.module)))
        .collect::<Vec<_>>()
        .join(",");

    let alias_entries = export
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

    let theorem_entries = export
        .theorems
        .iter()
        .map(|theorem| {
            format!(
                "{{\"obligation_name\":\"{}\",\"theorem_name\":\"{}\",\"witness_ref\":\"{}\"}}",
                esc(&theorem.obligation_name),
                esc(&theorem.theorem_name),
                esc(&theorem.witness_ref(&export.module_name))
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\"module_name\":\"{}\",\"project\":{{\"package_name\":\"{}\",\"toolchain\":\"{}\",\"requires_mathlib\":{}}},\"prelude\":{{\"imports\":[{}],\"aliases\":[{}]}},\"theorems\":[{}]}}",
        esc(&export.module_name),
        esc(&project.package_name),
        esc(&project.toolchain),
        project.requires_mathlib,
        import_entries,
        alias_entries,
        theorem_entries
    )
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
        assert!(temp.join("lakefile.lean").exists());
        assert!(temp.join("lean-toolchain").exists());

        let _ = fs::remove_dir_all(&temp);
    }
}
