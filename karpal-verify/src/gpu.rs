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
    use crate::{Origin, export_kani_bundle, export_lean_bundle, export_smt_bundle};

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
}
