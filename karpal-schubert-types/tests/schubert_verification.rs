// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use karpal_schubert_types::verification::{schubert_bundle, verify_schubert};

#[test]
fn schubert_bundle_contains_three_obligations() {
    let bundle = schubert_bundle();
    assert_eq!(
        bundle.obligations().len(),
        3,
        "should have LR consistency, partition validity, and intersection emptiness"
    );
}

#[test]
fn schubert_bundle_contains_lr_consistency() {
    let bundle = schubert_bundle();
    let names: Vec<&str> = bundle
        .obligations()
        .iter()
        .map(|o| o.name.as_str())
        .collect();
    assert!(
        names.iter().any(|n| n.contains("lr_consistency")),
        "should contain LR consistency obligation"
    );
}

#[test]
fn schubert_bundle_contains_partition_validity() {
    let bundle = schubert_bundle();
    let names: Vec<&str> = bundle
        .obligations()
        .iter()
        .map(|o| o.name.as_str())
        .collect();
    assert!(
        names.iter().any(|n| n.contains("partition_validity")),
        "should contain partition validity obligation"
    );
}

#[test]
fn verify_schubert_produces_report() {
    let report = verify_schubert();
    assert_eq!(
        report.bundle_name, "schubert_calculus",
        "report should be for schubert calculus"
    );
    assert_eq!(report.obligations.len(), 3);
}

#[test]
fn verify_schubert_all_obligations_have_results() {
    let report = verify_schubert();
    for obl_report in &report.obligations {
        assert!(
            obl_report.certificate.is_some(),
            "every obligation should have a certificate"
        );
    }
}
