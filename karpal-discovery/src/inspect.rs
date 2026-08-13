// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! The project inspector (Phase 19-C, slice 1) — a bounded, **read-only**
//! Cargo.toml metadata extractor producing a [`ProjectSnapshot`].
//!
//! Mirrors the catalog's design: a real parser (`toml`), typed records,
//! `BTreeMap`/sorted collections for deterministic serialization, and a
//! content hash. Where the catalog ( [`crate::extract`] ) walks the `syn` AST
//! of *source* to record the public API surface, the inspector walks *Cargo
//! manifests* and records project structure: workspace membership, per-crate
//! package metadata, dependencies (with source discrimination — registry /
//! path / git / workspace-inherited), feature flags, and explicit targets.
//!
//! ## Read-only guarantee
//!
//! The inspector performs filesystem reads and TOML parsing **only** — it
//! never spawns `cargo`, runs build scripts, resolves the lockfile, or
//! mutates the project. That keeps it inside the "bounded, no arbitrary
//! execution" envelope required of the discovery vertical (ROADMAP §"Project
//! inspection"). Resolved dependencies from `Cargo.lock`, conventional
//! (auto-detected) targets, and imported-symbol analysis arrive in later
//! slices.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

/// A read-only snapshot of a Cargo project/workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    /// The `[workspace]` table, present on the workspace-root manifest.
    pub workspace: Option<WorkspaceMeta>,
    /// Per-crate snapshots, keyed by package name (deterministic order).
    pub crates: BTreeMap<String, CrateSnapshot>,
    /// SHA-256 over the canonical `(workspace, crates)` serialization. Stable
    /// across runs over identical inputs — the drift-detection primitive.
    pub content_hash: String,
}

impl ProjectSnapshot {
    /// Number of catalogued crates.
    #[must_use]
    pub fn crate_count(&self) -> usize {
        self.crates.len()
    }

    /// Look up a crate snapshot by package name.
    #[must_use]
    pub fn crate_snapshot(&self, name: &str) -> Option<&CrateSnapshot> {
        self.crates.get(name)
    }
}

/// The `[workspace]` table of a workspace-root manifest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceMeta {
    /// Declared workspace members (may include globs, captured verbatim).
    pub members: Vec<String>,
    /// The optional `default-members` list.
    pub default_members: Option<Vec<String>>,
    /// The `exclude` list.
    pub exclude: Vec<String>,
    /// The `resolver` version (`"2"` / `"3"`), if declared.
    pub resolver: Option<String>,
}

/// A single crate's manifest, as captured by the inspector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateSnapshot {
    /// The `[package]` table.
    pub package: PackageMeta,
    /// All declared dependencies (normal + dev + build), keyed by name.
    pub dependencies: BTreeMap<String, DepRecord>,
    /// The `[features]` table: feature name → the entries it enables.
    pub features: BTreeMap<String, Vec<String>>,
    /// Explicit targets (`[lib]`, `[[bin]]`, `[[example]]`, `[[test]]`,
    /// `[[bench]]`). Conventional auto-targets are not inferred here.
    pub targets: TargetsRecord,
}

/// The `[package]` table of a crate manifest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageMeta {
    /// Package name.
    pub name: String,
    /// Package version (as written — may be workspace-inherited).
    pub version: String,
    /// The `edition`, if declared.
    pub edition: Option<String>,
    /// The `rust-version` (MSRV), if declared.
    pub rust_version: Option<String>,
    /// The `description`, if declared.
    pub description: Option<String>,
    /// The `license`, if declared.
    pub license: Option<String>,
}

/// The dependency graph edge kind: which `[*-dependencies]` table a dep came
/// from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DepKind {
    /// `[dependencies]`.
    #[default]
    Normal,
    /// `[dev-dependencies]`.
    Dev,
    /// `[build-dependencies]`.
    Build,
}

/// Where a dependency is sourced from. Mutually exclusive in a real manifest
/// (a dep carries exactly one of version / path / git, or is
/// workspace-inherited), so this is a single tagged enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum DepSource {
    /// `version = "…"` (or a bare string) — the crates.io registry.
    Registry {
        /// The version requirement, as written.
        version_req: String,
    },
    /// `path = "…"` — a local path dependency.
    Path {
        /// The path, as written (relative to the manifest).
        rel_path: String,
    },
    /// `git = "…"` — a git dependency.
    Git {
        /// The repository URL.
        url: String,
        /// The pinned `rev`, if any.
        rev: Option<String>,
        /// The pinned `branch`, if any.
        branch: Option<String>,
        /// The pinned `tag`, if any.
        tag: Option<String>,
    },
    /// `workspace = true` — inherited from the workspace manifest.
    Workspace,
}

/// A single declared dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepRecord {
    /// Which dependencies table this came from.
    pub kind: DepKind,
    /// Where the dependency is sourced from.
    pub source: DepSource,
    /// Whether `optional = true` was set.
    pub optional: bool,
    /// Whether default features are enabled (`true` unless
    /// `default-features = false`).
    pub default_features: bool,
    /// Feature flags activated by this dep declaration.
    pub features: Vec<String>,
}

/// Explicit targets declared in a manifest.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TargetsRecord {
    /// The `[lib]` target, if any.
    pub lib: Option<LibTarget>,
    /// `[[bin]]` targets.
    pub bins: Vec<NamedTarget>,
    /// `[[example]]` targets.
    pub examples: Vec<NamedTarget>,
    /// `[[test]]` targets (integration tests).
    pub integration_tests: Vec<NamedTarget>,
    /// `[[bench]]` targets.
    pub benches: Vec<NamedTarget>,
}

/// The `[lib]` target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibTarget {
    /// The library name (defaults to the package name with `-` → `_`).
    pub name: String,
    /// The source path, if declared (or `src/lib.rs` when auto-discovered).
    pub path: Option<String>,
    /// Explicit `crate-type` values, if declared.
    pub crate_types: Option<Vec<String>>,
    /// `true` when inferred from `src/lib.rs` by convention rather than
    /// declared explicitly via `[lib]`.
    pub auto_discovered: bool,
}

/// A named target (`[[bin]]` / `[[example]]` / `[[test]]` / `[[bench]]`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedTarget {
    /// The target name.
    pub name: String,
    /// The source path, if declared or conventionally inferred.
    pub path: Option<String>,
    /// `true` when inferred from the filesystem layout by convention (e.g.
    /// `src/bin/<name>.rs`) rather than declared explicitly.
    pub auto_discovered: bool,
}

/// Extract a read-only project snapshot from a workspace root.
///
/// Walks the tree (skipping `target/`, `.git`, and hidden dirs), parses every
/// `Cargo.toml` with a real TOML parser, and assembles a [`ProjectSnapshot`].
/// The first manifest carrying a `[workspace]` table supplies the workspace
/// meta; every manifest with a `[package]` table contributes a crate
/// snapshot. Unreadable or unparseable manifests are skipped rather than
/// fatal — the inspector degrades gracefully over a partial checkout.
pub fn inspect_workspace(root: &Path) -> ProjectSnapshot {
    let mut workspace: Option<WorkspaceMeta> = None;
    let mut crates: BTreeMap<String, CrateSnapshot> = BTreeMap::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !is_ignored_dir(e))
    {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_file() || entry.file_name() != "Cargo.toml" {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        let Ok(manifest): Result<toml::Table, _> = toml::from_str(&text) else {
            continue;
        };

        if workspace.is_none() {
            workspace = manifest
                .get("workspace")
                .and_then(toml::Value::as_table)
                .map(parse_workspace);
        }
        if let Some(pkg) = manifest.get("package").and_then(toml::Value::as_table) {
            let crate_dir = entry
                .path()
                .parent()
                .expect("a Cargo.toml always has a parent directory");
            let mut snapshot = parse_crate(&manifest, pkg);
            discover_auto_targets(
                &mut snapshot.targets,
                pkg,
                &snapshot.package.name,
                crate_dir,
            );
            crates.insert(snapshot.package.name.clone(), snapshot);
        }
    }

    let content_hash = compute_hash(&workspace, &crates);
    ProjectSnapshot {
        workspace,
        crates,
        content_hash,
    }
}

/// Whether a `walkdir` entry is a directory the inspector should not descend
/// into (build output, VCS metadata, hidden dirs). The walk root itself is
/// never ignored — a temporary or dotted workspace root must still be walked.
fn is_ignored_dir(entry: &walkdir::DirEntry) -> bool {
    if !entry.file_type().is_dir() || entry.depth() == 0 {
        return false;
    }
    let name = entry.file_name().to_string_lossy();
    name == "target" || name == ".git" || name == "node_modules" || name.starts_with('.')
}

fn parse_workspace(ws: &toml::Table) -> WorkspaceMeta {
    WorkspaceMeta {
        members: string_array(ws, "members"),
        default_members: optional_string_array(ws, "default-members"),
        exclude: string_array(ws, "exclude"),
        resolver: ws
            .get("resolver")
            .and_then(toml::Value::as_str)
            .map(String::from),
    }
}

fn parse_crate(manifest: &toml::Table, pkg: &toml::Table) -> CrateSnapshot {
    let package = PackageMeta {
        name: pkg
            .get("name")
            .and_then(toml::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        version: pkg
            .get("version")
            .and_then(toml::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        edition: pkg
            .get("edition")
            .and_then(toml::Value::as_str)
            .map(String::from),
        rust_version: pkg
            .get("rust-version")
            .and_then(toml::Value::as_str)
            .map(String::from),
        description: pkg
            .get("description")
            .and_then(toml::Value::as_str)
            .map(String::from),
        license: pkg
            .get("license")
            .and_then(toml::Value::as_str)
            .map(String::from),
    };

    let mut dependencies = BTreeMap::new();
    for (kind, section) in [
        (DepKind::Normal, "dependencies"),
        (DepKind::Dev, "dev-dependencies"),
        (DepKind::Build, "build-dependencies"),
    ] {
        if let Some(deps) = manifest.get(section).and_then(toml::Value::as_table) {
            for (name, spec) in deps {
                dependencies.insert(name.clone(), parse_dep(kind, spec));
            }
        }
    }

    let features = manifest
        .get("features")
        .and_then(toml::Value::as_table)
        .map(|t| {
            t.iter()
                .map(|(k, v)| (k.clone(), string_array_value(v)))
                .collect()
        })
        .unwrap_or_default();

    let targets = parse_targets(manifest);

    CrateSnapshot {
        package,
        dependencies,
        features,
        targets,
    }
}

/// Parse one dependency spec — a bare version string or a detail table —
/// into a [`DepRecord`] of the given kind.
fn parse_dep(kind: DepKind, spec: &toml::Value) -> DepRecord {
    match spec {
        toml::Value::String(version_req) => DepRecord {
            kind,
            source: DepSource::Registry {
                version_req: version_req.clone(),
            },
            optional: false,
            default_features: true,
            features: Vec::new(),
        },
        toml::Value::Table(t) => {
            let source = if t
                .get("workspace")
                .and_then(toml::Value::as_bool)
                .unwrap_or(false)
            {
                DepSource::Workspace
            } else if let Some(rel_path) = t.get("path").and_then(toml::Value::as_str) {
                DepSource::Path {
                    rel_path: rel_path.to_string(),
                }
            } else if let Some(url) = t.get("git").and_then(toml::Value::as_str) {
                DepSource::Git {
                    url: url.to_string(),
                    rev: t.get("rev").and_then(toml::Value::as_str).map(String::from),
                    branch: t
                        .get("branch")
                        .and_then(toml::Value::as_str)
                        .map(String::from),
                    tag: t.get("tag").and_then(toml::Value::as_str).map(String::from),
                }
            } else {
                DepSource::Registry {
                    version_req: t
                        .get("version")
                        .and_then(toml::Value::as_str)
                        .unwrap_or("*")
                        .to_string(),
                }
            };
            DepRecord {
                kind,
                source,
                optional: t
                    .get("optional")
                    .and_then(toml::Value::as_bool)
                    .unwrap_or(false),
                default_features: t
                    .get("default-features")
                    .and_then(toml::Value::as_bool)
                    .unwrap_or(true),
                features: string_array_value(
                    t.get("features").unwrap_or(&toml::Value::Array(Vec::new())),
                ),
            }
        }
        // A dependency spec that is neither string nor table is malformed;
        // record it as an unconstrained registry dep so it is still visible.
        _ => DepRecord {
            kind,
            source: DepSource::Registry {
                version_req: "*".to_string(),
            },
            optional: false,
            default_features: true,
            features: Vec::new(),
        },
    }
}

fn parse_targets(manifest: &toml::Table) -> TargetsRecord {
    TargetsRecord {
        lib: manifest
            .get("lib")
            .and_then(toml::Value::as_table)
            .map(parse_lib),
        bins: target_array(manifest, "bin"),
        examples: target_array(manifest, "example"),
        integration_tests: target_array(manifest, "test"),
        benches: target_array(manifest, "bench"),
    }
}

fn parse_lib(t: &toml::Table) -> LibTarget {
    LibTarget {
        name: t
            .get("name")
            .and_then(toml::Value::as_str)
            .map(String::from)
            .unwrap_or_default(),
        path: t
            .get("path")
            .and_then(toml::Value::as_str)
            .map(String::from),
        crate_types: t
            .get("crate-type")
            .map(string_array_value)
            .filter(|v| !v.is_empty()),
        auto_discovered: false,
    }
}

fn target_array(manifest: &toml::Table, key: &str) -> Vec<NamedTarget> {
    manifest
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|arr| arr.iter().filter_map(named_target).collect())
        .unwrap_or_default()
}

fn named_target(value: &toml::Value) -> Option<NamedTarget> {
    let t = value.as_table()?;
    Some(NamedTarget {
        name: t.get("name")?.as_str()?.to_string(),
        path: t
            .get("path")
            .and_then(toml::Value::as_str)
            .map(String::from),
        auto_discovered: false,
    })
}

/// Infer conventional (auto-detected) targets from the crate's filesystem
/// layout and merge them into `targets`, deduplicated against the explicit
/// targets already recorded. Mirrors Cargo's auto-discovery rules: `src/lib.rs`
/// → lib; `src/main.rs` + `src/bin/*` → bins; `examples/*`, `tests/*`,
/// `benches/*` → their kinds. The `autobins`/`autoexamples`/`autotests`/
/// `autobenches` `[package]` flags (default true) gate each kind.
fn discover_auto_targets(
    targets: &mut TargetsRecord,
    pkg: &toml::Table,
    package_name: &str,
    crate_dir: &Path,
) {
    // Lib: conventionally inferred from `src/lib.rs` when no `[lib]` exists.
    if targets.lib.is_none() && crate_dir.join("src/lib.rs").is_file() {
        targets.lib = Some(LibTarget {
            name: cargo_name(package_name),
            path: Some("src/lib.rs".to_string()),
            crate_types: None,
            auto_discovered: true,
        });
    }

    if auto_flag(pkg, "autobins") {
        let explicit = explicit_paths(&targets.bins, "src/bin");
        // `src/main.rs` → a bin named after the package (unless an explicit
        // bin already claims that path).
        if crate_dir.join("src/main.rs").is_file() && !explicit.contains("src/main.rs") {
            targets.bins.push(NamedTarget {
                name: cargo_name(package_name),
                path: Some("src/main.rs".to_string()),
                auto_discovered: true,
            });
        }
        discover_dir_targets(crate_dir, "src/bin", &explicit, &mut targets.bins);
    }
    if auto_flag(pkg, "autoexamples") {
        let explicit = explicit_paths(&targets.examples, "examples");
        discover_dir_targets(crate_dir, "examples", &explicit, &mut targets.examples);
    }
    if auto_flag(pkg, "autotests") {
        let explicit = explicit_paths(&targets.integration_tests, "tests");
        discover_dir_targets(
            crate_dir,
            "tests",
            &explicit,
            &mut targets.integration_tests,
        );
    }
    if auto_flag(pkg, "autobenches") {
        let explicit = explicit_paths(&targets.benches, "benches");
        discover_dir_targets(crate_dir, "benches", &explicit, &mut targets.benches);
    }

    // Restore determinism: `read_dir` order is platform-dependent.
    for vec in [
        &mut targets.bins,
        &mut targets.examples,
        &mut targets.integration_tests,
        &mut targets.benches,
    ] {
        vec.sort_by(|a, b| a.name.cmp(&b.name));
    }
}

/// Translate a package name to a Cargo target name (`-` → `_`).
fn cargo_name(package_name: &str) -> String {
    package_name.replace('-', "_")
}

/// Read an `auto*` `[package]` flag, defaulting to `true` (auto-discovery on).
fn auto_flag(pkg: &toml::Table, key: &str) -> bool {
    pkg.get(key).and_then(toml::Value::as_bool).unwrap_or(true)
}

/// Effective source paths of explicit named targets, for dedup: the declared
/// path, or the conventional default `<subdir>/<name>.rs` for a name-only
/// declaration.
fn explicit_paths(explicit: &[NamedTarget], subdir: &str) -> BTreeSet<String> {
    explicit
        .iter()
        .map(|t| match &t.path {
            Some(p) => p.clone(),
            None => format!("{subdir}/{}.rs", t.name),
        })
        .collect()
}

/// Scan `<crate_dir>/<subdir>` for conventional targets — `<name>.rs` files
/// and `<name>/main.rs` subdirectories — appending any not already claimed by
/// an explicit target path.
fn discover_dir_targets(
    crate_dir: &Path,
    subdir: &str,
    explicit: &BTreeSet<String>,
    out: &mut Vec<NamedTarget>,
) {
    let Ok(entries) = std::fs::read_dir(crate_dir.join(subdir)) else {
        return;
    };
    for entry in entries.flatten() {
        let Some(file_name) = entry.file_name().to_str().map(String::from) else {
            continue;
        };
        if let Some(name) = file_name.strip_suffix(".rs") {
            let rel_path = format!("{subdir}/{file_name}");
            if !explicit.contains(&rel_path) {
                out.push(NamedTarget {
                    name: name.to_string(),
                    path: Some(rel_path),
                    auto_discovered: true,
                });
            }
        } else if entry.path().join("main.rs").is_file() {
            // `<subdir>/<name>/main.rs` form.
            let rel_path = format!("{subdir}/{file_name}/main.rs");
            if !explicit.contains(&rel_path) {
                out.push(NamedTarget {
                    name: file_name,
                    path: Some(rel_path),
                    auto_discovered: true,
                });
            }
        }
    }
}

fn string_array(table: &toml::Table, key: &str) -> Vec<String> {
    table.get(key).map(string_array_value).unwrap_or_default()
}

fn optional_string_array(table: &toml::Table, key: &str) -> Option<Vec<String>> {
    table
        .get(key)
        .map(string_array_value)
        .filter(|v| !v.is_empty())
}

fn string_array_value(value: &toml::Value) -> Vec<String> {
    value
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|e| e.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn compute_hash(
    workspace: &Option<WorkspaceMeta>,
    crates: &BTreeMap<String, CrateSnapshot>,
) -> String {
    let bytes = serde_json::to_vec(&(workspace, crates)).expect("snapshot fragment serializes");
    hex::encode(Sha256::digest(&bytes))
}
