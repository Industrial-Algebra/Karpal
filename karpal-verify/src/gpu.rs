// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec, vec::Vec};
#[cfg(feature = "std")]
use std::{string::String, vec, vec::Vec};

use karpal_proof::Property;

use crate::{Declaration, Obligation, ObligationBundle, Origin, Sort, Term, VerificationTier};

pub struct IsBufferAlignedTo16;
impl Property for IsBufferAlignedTo16 {
    const NAME: &'static str = "IsBufferAlignedTo16";
}

pub struct IsWorkgroupSizeDivisible;
impl Property for IsWorkgroupSizeDivisible {
    const NAME: &'static str = "IsWorkgroupSizeDivisible";
}

pub struct IsDispatchWithinLimits;
impl Property for IsDispatchWithinLimits {
    const NAME: &'static str = "IsDispatchWithinLimits";
}

pub struct IsMSLKernelDeterministic;
impl Property for IsMSLKernelDeterministic {
    const NAME: &'static str = "IsMSLKernelDeterministic";
}

/// A linear GPU kernel produces the numerically correct output under the
/// DeepReinforce exact-match protocol.
///
/// The protocol restricts kernel inputs to binary `{0, 1}` values so that
/// floating-point associativity holds exactly within the FP16 integer range
/// `[0, 2048]`. A FP32 CPU reference is then compared against the GPU output
/// with bit-exact equality at every position where the reference value is at
/// or below the threshold.
///
/// This is a **runtime** verification property: the obligation records that
/// a numerical check has been specified, but the actual check runs in the
/// consumer crate (e.g., Borsalino's `verify_numerical()`).
///
/// # Limitations
///
/// Only applies to linear kernels. Non-linear operations (`log`, `exp`,
/// `tanh`) produce irrational outputs that cannot be checked with exact match.
///
/// Reference: <https://deep-reinforce.com/correctness_check.html>
pub struct IsNumericallyCorrect;
impl Property for IsNumericallyCorrect {
    const NAME: &'static str = "IsNumericallyCorrect";
}

/// Builder for GPU compute verification obligations.
#[derive(Debug, Clone)]
pub struct GpuObligationBundle {
    bundle: ObligationBundle,
}

impl GpuObligationBundle {
    pub fn metal_kernel(name: impl Into<String>, origin: Origin) -> Self {
        Self {
            bundle: ObligationBundle::new(name, origin),
        }
    }

    pub fn with_buffer_alignment(mut self, buffer: impl Into<String>, alignment: i64) -> Self {
        let buffer = buffer.into();
        let property = if alignment == 16 {
            IsBufferAlignedTo16::NAME
        } else {
            "IsBufferAligned"
        };
        self.bundle.push(Obligation {
            name: format_obligation_name(&buffer, "buffer_alignment"),
            property,
            declarations: vec![Declaration::new(buffer.clone(), Sort::named("MTLBuffer"))],
            assumptions: Vec::new(),
            conclusion: Term::app("aligned_to", [Term::var(buffer), Term::int(alignment)]),
            origin: self.bundle.origin.clone(),
            tier: VerificationTier::External,
        });
        self
    }

    pub fn with_workgroup_divisibility(mut self, symbol: impl Into<String>, divisor: i64) -> Self {
        let symbol = symbol.into();
        self.bundle.push(Obligation {
            name: format_obligation_name(&symbol, "workgroup_divisibility"),
            property: IsWorkgroupSizeDivisible::NAME,
            declarations: vec![Declaration::new(symbol.clone(), Sort::Int)],
            assumptions: Vec::new(),
            conclusion: Term::app("divisible_by", [Term::var(symbol), Term::int(divisor)]),
            origin: self.bundle.origin.clone(),
            tier: VerificationTier::External,
        });
        self
    }

    pub fn with_dispatch_limit(mut self, symbol: impl Into<String>, limit: i64) -> Self {
        let symbol = symbol.into();
        self.bundle.push(Obligation {
            name: format_obligation_name(&symbol, "dispatch_limit"),
            property: IsDispatchWithinLimits::NAME,
            declarations: vec![Declaration::new(symbol.clone(), Sort::Int)],
            assumptions: Vec::new(),
            conclusion: Term::app(
                "within_dispatch_limit",
                [Term::var(symbol), Term::int(limit)],
            ),
            origin: self.bundle.origin.clone(),
            tier: VerificationTier::External,
        });
        self
    }

    pub fn with_kernel_determinism(mut self, kernel: impl Into<String>) -> Self {
        let kernel = kernel.into();
        self.bundle.push(Obligation {
            name: format_obligation_name(&kernel, "kernel_determinism"),
            property: IsMSLKernelDeterministic::NAME,
            declarations: vec![Declaration::new(kernel.clone(), Sort::named("MSLKernel"))],
            assumptions: Vec::new(),
            conclusion: Term::app("deterministic_kernel", [Term::var(kernel)]),
            origin: self.bundle.origin.clone(),
            tier: VerificationTier::External,
        });
        self
    }

    /// Declare that this kernel has a numerical correctness check specified.
    ///
    /// Records that the kernel will be verified against a FP32 CPU reference
    /// using the DeepReinforce exact-match protocol. The actual runtime check
    /// runs in the consumer crate (e.g., Borsalino's `verify_numerical()`).
    ///
    /// # Parameters
    ///
    /// - `kernel`: The kernel symbol being verified.
    /// - `threshold`: The exact-match ceiling (2048 for FP16, the largest
    ///   integer exactly representable in FP16). Positions where the reference
    ///   output exceeds this value are ignored.
    /// - `trials`: Number of random binary-input trials to run.
    ///
    /// # Tier
    ///
    /// Uses [`VerificationTier::Emergent`] — runtime discovery, not
    /// compile-time phantom or external theorem prover.
    ///
    /// # Reference
    ///
    /// See [Towards a Reliable Kernel Correctness Check in Matrix
    /// Multiplication](https://deep-reinforce.com/correctness_check.html).
    pub fn with_numerical_correctness(
        mut self,
        kernel: impl Into<String>,
        threshold: i64,
        trials: u32,
    ) -> Self {
        let kernel = kernel.into();
        self.bundle.push(Obligation {
            name: format_obligation_name(&kernel, "numerical_correctness"),
            property: IsNumericallyCorrect::NAME,
            declarations: vec![
                Declaration::new("threshold", Sort::Int),
                Declaration::new("trials", Sort::Int),
            ],
            assumptions: vec![
                Term::app("le", [Term::int(0), Term::var("threshold")]),
                Term::app("le", [Term::int(1), Term::var("trials")]),
            ],
            conclusion: Term::app(
                "numerically_correct_under_exact_match",
                [
                    Term::var(kernel.clone()),
                    Term::int(threshold),
                    Term::int(i64::from(trials)),
                ],
            ),
            origin: self.bundle.origin.clone(),
            tier: VerificationTier::Emergent,
        });
        self
    }

    pub fn into_bundle(self) -> ObligationBundle {
        self.bundle
    }
}

fn format_obligation_name(symbol: &str, suffix: &str) -> String {
    let mut out = String::new();
    for ch in symbol.chars() {
        out.push(if ch.is_ascii_alphanumeric() || ch == '_' {
            ch
        } else {
            '_'
        });
    }
    out.push('_');
    out.push_str(suffix);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Origin, VerificationTier, export_kani_bundle, export_lean_bundle, export_smt_bundle,
    };

    #[test]
    fn gpu_bundle_contains_expected_obligations() {
        let bundle = GpuObligationBundle::metal_kernel(
            "borsalino_kernel",
            Origin::new("borsalino", "kernels::reduce"),
        )
        .with_buffer_alignment("input", 16)
        .with_workgroup_divisibility("threads_per_group", 32)
        .with_dispatch_limit("grid_size", 65_535)
        .with_kernel_determinism("reduce_kernel")
        .into_bundle();

        assert!(
            bundle
                .obligations()
                .iter()
                .any(|obligation| obligation.property == "IsBufferAlignedTo16")
        );
        assert!(
            bundle
                .obligations()
                .iter()
                .any(|obligation| obligation.property == "IsWorkgroupSizeDivisible")
        );
        assert!(
            bundle
                .obligations()
                .iter()
                .any(|obligation| obligation.property == "IsDispatchWithinLimits")
        );
        assert!(
            bundle
                .obligations()
                .iter()
                .any(|obligation| obligation.property == "IsMSLKernelDeterministic")
        );
    }

    #[test]
    fn gpu_bundle_exports_through_all_backends() {
        let bundle = GpuObligationBundle::metal_kernel(
            "borsalino_kernel",
            Origin::new("borsalino", "kernels::reduce"),
        )
        .with_buffer_alignment("input", 16)
        .with_workgroup_divisibility("threads_per_group", 32)
        .with_dispatch_limit("grid_size", 65_535)
        .with_kernel_determinism("reduce_kernel")
        .into_bundle();

        let smt = export_smt_bundle(&bundle);
        let lean = export_lean_bundle("GpuVerify", &bundle);
        let kani = export_kani_bundle(&bundle);

        assert_eq!(smt.len(), 4);
        assert!(lean.contains("deterministic_kernel"));
        assert_eq!(kani.len(), 4);
        assert!(kani[0].source.contains("kani::assert"));
    }

    // ── Numerical correctness (v0.6.0) ───────────────────────────
    //
    // Based on the DeepReinforce exact-match protocol:
    // https://deep-reinforce.com/correctness_check.html
    //
    // Linear kernels are checked by restricting inputs to binary {0, 1}
    // values and requiring bit-exact equality against an FP32 CPU
    // reference at positions where the reference value <= 2048 (the FP16
    // exact-integer ceiling).

    #[test]
    fn numerical_correctness_property_has_name() {
        assert_eq!(IsNumericallyCorrect::NAME, "IsNumericallyCorrect");
    }

    #[test]
    fn with_numerical_correctness_adds_obligation() {
        let bundle = GpuObligationBundle::metal_kernel(
            "borsalino_matmul",
            Origin::new("borsalino", "kernels::matmul"),
        )
        .with_numerical_correctness("matmul_kernel", 2048, 16)
        .into_bundle();

        let obligations = bundle.obligations();
        assert_eq!(obligations.len(), 1);
        assert_eq!(obligations[0].property, IsNumericallyCorrect::NAME);
        assert_eq!(obligations[0].tier, VerificationTier::Emergent);
    }

    #[test]
    fn numerical_correctness_exports_through_all_backends() {
        let bundle = GpuObligationBundle::metal_kernel(
            "borsalino_matmul",
            Origin::new("borsalino", "kernels::matmul"),
        )
        .with_buffer_alignment("input_a", 16)
        .with_buffer_alignment("input_b", 16)
        .with_numerical_correctness("matmul_kernel", 2048, 16)
        .into_bundle();

        let smt = export_smt_bundle(&bundle);
        let lean = export_lean_bundle("GpuVerify", &bundle);
        let kani = export_kani_bundle(&bundle);

        assert_eq!(smt.len(), 3);
        assert!(
            lean.contains("numerically_correct_under_exact_match"),
            "Lean export should contain the numerical correctness predicate"
        );
        assert_eq!(kani.len(), 3);
        assert!(
            kani.iter()
                .any(|h| h.source.contains("numerically_correct_under_exact_match")),
            "Kani export should contain the numerical correctness predicate"
        );
    }

    #[test]
    fn numerical_correctness_records_threshold_and_trials() {
        let bundle = GpuObligationBundle::metal_kernel(
            "borsalino_gp",
            Origin::new("borsalino", "kernels::geometric_product"),
        )
        .with_numerical_correctness("gp_kernel", 2048, 32)
        .into_bundle();

        let obligation = &bundle.obligations()[0];

        // The conclusion should reference the threshold and trials.
        let conclusion_str = format!("{:?}", obligation.conclusion);
        assert!(
            conclusion_str.contains("2048"),
            "Conclusion should contain the threshold value: {conclusion_str}"
        );
        assert!(
            conclusion_str.contains("32"),
            "Conclusion should contain the trials value: {conclusion_str}"
        );
    }
}
