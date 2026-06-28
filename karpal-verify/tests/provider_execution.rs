// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use std::{fs, process::Command};

use karpal_proof::Property;
use karpal_verify::{
    ExecutionStatus, InvocationPlan, KaniConfig, LeanConfig, LocalProcessRunner, Obligation,
    Origin, Term, VerificationTier, VerifierRunner, export_kani_harness,
};

struct ProviderSmoke;
impl Property for ProviderSmoke {
    const NAME: &'static str = "ProviderSmoke";
}

fn command_available(command: &str, args: &[&str]) -> bool {
    Command::new(command)
        .args(args)
        .output()
        .is_ok_and(|output| output.status.success())
}

fn smoke_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("karpal_verify_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("smoke directory should be created");
    dir
}

#[test]
fn lean_provider_executes_direct_smoke_module() {
    if !command_available("lean", &["--version"]) {
        eprintln!("lean unavailable; skipping provider execution smoke test");
        return;
    }

    let dir = smoke_dir("lean_provider");
    let module = dir.join("KarpalSmoke.lean");
    fs::write(
        &module,
        "namespace KarpalSmoke\n\ntheorem smoke : True := by\n  trivial\n\nend KarpalSmoke\n",
    )
    .expect("Lean smoke module should be written");

    let plan = InvocationPlan::lean(&LeanConfig::default(), &module);
    let result = LocalProcessRunner.run(&plan);

    assert_eq!(result.status, ExecutionStatus::Success, "{result:#?}");
    assert_eq!(result.exit_code, Some(0), "{result:#?}");
}

#[test]
fn kani_provider_executes_generated_bool_harness() {
    if !command_available("kani", &["--version"]) {
        eprintln!("kani unavailable; skipping provider execution smoke test");
        return;
    }

    let obligation = Obligation::for_property::<ProviderSmoke>(
        "kani_provider_smoke",
        Origin::new("karpal-verify", "provider smoke"),
        VerificationTier::External,
        Term::bool(true),
    );
    let harness = export_kani_harness(&obligation);

    let dir = smoke_dir("kani_provider");
    let harness_path = dir.join(format!("{}.rs", harness.harness_name));
    fs::write(&harness_path, harness.source).expect("Kani smoke harness should be written");

    let plan = InvocationPlan::kani(&KaniConfig::default(), &harness_path, &harness.harness_name);
    let result = LocalProcessRunner.run(&plan);

    assert_eq!(result.status, ExecutionStatus::Success, "{result:#?}");
    assert_eq!(result.exit_code, Some(0), "{result:#?}");
}
