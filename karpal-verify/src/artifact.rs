use crate::{
    InvocationPlan, LeanConfig, ObligationBundle, SmtConfig, export_lean_bundle, export_smt_bundle,
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

    let lean_path = layout.lean_dir.join(format!("{lean_module_name}.lean"));
    fs::write(&lean_path, export_lean_bundle(lean_module_name, bundle))?;
    plans.push(InvocationPlan::lean(lean, &lean_path));
    records.push(ArtifactRecord {
        name: lean_module_name.into(),
        path: path_to_string(&lean_path),
    });

    Ok(ArtifactBatch {
        root: path_to_string(&layout.root),
        records,
        plans,
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

    let lean_path = layout.lean_dir.join(format!("{lean_module_name}.lean"));
    plans.push(InvocationPlan::lean(lean, &lean_path));
    records.push(ArtifactRecord {
        name: lean_module_name.into(),
        path: path_to_string(&lean_path),
    });

    ArtifactBatch {
        root: path_to_string(&layout.root),
        records,
        plans,
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
            &LeanConfig::default(),
        );

        assert_eq!(batch.records.len(), 4);
        assert_eq!(batch.plans.len(), 4);
        assert!(
            batch
                .records
                .iter()
                .any(|r| r.path.ends_with("KarpalVerify.lean"))
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

        let _ = fs::remove_dir_all(&temp);
    }
}
