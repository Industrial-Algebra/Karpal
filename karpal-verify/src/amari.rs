use amari_flynn::{
    backend::{
        monte_carlo::MonteCarloVerifier,
        smt::{self},
    },
    contracts::{EventVerification, RareEvent},
};

#[cfg(feature = "std")]
use crate::VerificationReport;
use crate::{Obligation, ObligationBundle, VerificationTier};

pub use amari_flynn::{
    backend::{
        monte_carlo::MonteCarloVerifier as AmariMonteCarloVerifier,
        smt::{
            ObligationKind as AmariObligationKind, SmtProofObligation as AmariSmtProofObligation,
        },
    },
    contracts::{
        StatisticalProperty as AmariStatisticalProperty,
        VerificationResult as AmariVerificationResult,
    },
};

/// Probability bound and Monte Carlo configuration for rare-event verification.
#[derive(Debug, Clone, PartialEq)]
pub struct StatisticalBound {
    pub probability: f64,
    pub samples: usize,
    pub rare_threshold: f64,
}

impl Default for StatisticalBound {
    fn default() -> Self {
        Self {
            probability: 0.05,
            samples: 10_000,
            rare_threshold: 0.01,
        }
    }
}

impl StatisticalBound {
    pub fn new(probability: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&probability),
            "probability bound must be in [0, 1]"
        );
        Self {
            probability,
            ..Self::default()
        }
    }

    pub fn with_samples(mut self, samples: usize) -> Self {
        assert!(samples > 0, "sample count must be greater than zero");
        self.samples = samples;
        self
    }

    pub fn with_rare_threshold(mut self, rare_threshold: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&rare_threshold),
            "rare threshold must be in [0, 1]"
        );
        self.rare_threshold = rare_threshold;
        self
    }
}

/// Statistical verification outcome backed by amari-flynn Monte Carlo estimation.
#[derive(Debug, Clone, PartialEq)]
pub struct StatisticalVerification {
    pub obligation_name: String,
    pub property: String,
    pub origin_summary: String,
    pub samples: usize,
    pub bound: f64,
    pub rare_threshold: f64,
    pub estimated_probability: f64,
    pub lower_probability: f64,
    pub upper_probability: f64,
    pub status: AmariVerificationResult,
    pub classification: EventVerification,
}

impl StatisticalVerification {
    pub fn is_verified(&self) -> bool {
        self.status == AmariVerificationResult::Verified
    }

    pub fn tier(&self) -> VerificationTier {
        classify_tier(self.estimated_probability, self.rare_threshold)
    }

    pub fn rare_event<T>(&self) -> Option<RareEvent<T>> {
        if self.classification != EventVerification::Rare {
            return None;
        }

        let probability = self.estimated_probability;
        if !(0.0..1.0).contains(&probability) {
            return None;
        }

        Some(RareEvent::new(
            probability,
            format!("{} [{}]", self.obligation_name, self.property),
        ))
    }
}

/// Map an estimated probability into Karpal's verification tiers.
pub fn classify_tier(probability: f64, rare_threshold: f64) -> VerificationTier {
    match EventVerification::classify(probability, rare_threshold) {
        EventVerification::Impossible => VerificationTier::Impossible,
        EventVerification::Rare => VerificationTier::Rare,
        EventVerification::Probable => VerificationTier::Emergent,
    }
}

/// Verify a law-violation predicate as a statistically rare event.
///
/// The predicate should return `true` when a sample exhibits the violating event.
pub fn verify_rare_event<F>(
    obligation: &Obligation,
    bound: &StatisticalBound,
    predicate: F,
) -> StatisticalVerification
where
    F: Fn() -> bool,
{
    let verifier = MonteCarloVerifier::new(bound.samples);
    let status = verifier.verify_probability_bound(&predicate, bound.probability);
    let (estimate, lower, upper) = verifier.estimate_probability(predicate);

    StatisticalVerification {
        obligation_name: obligation.name.clone(),
        property: obligation.property.into(),
        origin_summary: format_origin(obligation),
        samples: bound.samples,
        bound: bound.probability,
        rare_threshold: bound.rare_threshold,
        estimated_probability: estimate,
        lower_probability: lower,
        upper_probability: upper,
        status,
        classification: EventVerification::classify(estimate, bound.rare_threshold),
    }
}

/// Build an amari-flynn precondition obligation from a Karpal obligation.
pub fn precondition_obligation_for(
    obligation: &Obligation,
    condition_desc: impl Into<String>,
    probability: f64,
) -> AmariSmtProofObligation {
    smt::precondition_obligation(
        amari_name(obligation, "precondition"),
        condition_desc,
        probability,
    )
}

/// Build an amari-flynn postcondition obligation from a Karpal obligation.
pub fn postcondition_obligation_for(
    obligation: &Obligation,
    condition_desc: impl Into<String>,
    probability: f64,
) -> AmariSmtProofObligation {
    smt::postcondition_obligation(
        amari_name(obligation, "postcondition"),
        condition_desc,
        probability,
    )
}

/// Build an amari-flynn Hoeffding concentration obligation from a Karpal obligation.
pub fn concentration_obligation_for(
    obligation: &Obligation,
    samples: usize,
    epsilon: f64,
    delta: f64,
) -> AmariSmtProofObligation {
    smt::hoeffding_obligation(amari_name(obligation, "hoeffding"), samples, epsilon, delta)
}

/// Build an amari-flynn expected-value obligation from a Karpal obligation.
pub fn expected_value_obligation_for(
    obligation: &Obligation,
    expected: f64,
    epsilon: f64,
    samples: usize,
) -> AmariSmtProofObligation {
    smt::expected_value_obligation(
        amari_name(obligation, "expected-value"),
        expected,
        epsilon,
        samples,
    )
}

/// Combined three-tier view over Karpal obligations.
#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq)]
pub struct ThreeTierObligationReport {
    pub obligation_name: String,
    pub summary: String,
    pub declared_tier: VerificationTier,
    pub evidence_tier: VerificationTier,
    pub external_success: Option<bool>,
    pub statistical_verification: Option<StatisticalVerification>,
}

#[cfg(feature = "std")]
impl ThreeTierObligationReport {
    pub fn is_verified(&self) -> bool {
        self.external_success.unwrap_or(false)
            || self
                .statistical_verification
                .as_ref()
                .map(StatisticalVerification::is_verified)
                .unwrap_or(false)
    }
}

/// Aggregate three-tier report for a bundle.
#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq)]
pub struct ThreeTierVerificationReport {
    pub bundle_name: String,
    pub obligations: Vec<ThreeTierObligationReport>,
}

#[cfg(feature = "std")]
impl ThreeTierVerificationReport {
    pub fn count_for(&self, tier: VerificationTier) -> usize {
        self.obligations
            .iter()
            .filter(|obligation| obligation.evidence_tier == tier)
            .count()
    }

    pub fn impossible_count(&self) -> usize {
        self.count_for(VerificationTier::Impossible)
    }

    pub fn rare_count(&self) -> usize {
        self.count_for(VerificationTier::Rare)
    }

    pub fn emergent_count(&self) -> usize {
        self.count_for(VerificationTier::Emergent)
    }

    pub fn external_count(&self) -> usize {
        self.count_for(VerificationTier::External)
    }

    pub fn verified_count(&self) -> usize {
        self.obligations
            .iter()
            .filter(|obligation| obligation.is_verified())
            .count()
    }

    pub fn to_json(&self) -> String {
        fn esc(s: &str) -> String {
            s.replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
        }

        let obligations = self
            .obligations
            .iter()
            .map(|obligation| {
                let statistical = obligation
                    .statistical_verification
                    .as_ref()
                    .map(|verification| {
                        format!(
                            "{{\"status\":\"{:?}\",\"classification\":\"{:?}\",\"estimated_probability\":{},\"bound\":{},\"samples\":{}}}",
                            verification.status,
                            verification.classification,
                            verification.estimated_probability,
                            verification.bound,
                            verification.samples
                        )
                    })
                    .unwrap_or_else(|| "null".into());
                format!(
                    "{{\"name\":\"{}\",\"summary\":\"{}\",\"declared_tier\":\"{:?}\",\"evidence_tier\":\"{:?}\",\"external_success\":{},\"statistical\":{}}}",
                    esc(&obligation.obligation_name),
                    esc(&obligation.summary),
                    obligation.declared_tier,
                    obligation.evidence_tier,
                    obligation
                        .external_success
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "null".into()),
                    statistical
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        format!(
            "{{\"bundle_name\":\"{}\",\"impossible_count\":{},\"rare_count\":{},\"emergent_count\":{},\"external_count\":{},\"verified_count\":{},\"obligations\":[{}]}}",
            esc(&self.bundle_name),
            self.impossible_count(),
            self.rare_count(),
            self.emergent_count(),
            self.external_count(),
            self.verified_count(),
            obligations
        )
    }

    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("# Three-Tier Verification Report\n\n");
        out.push_str(&format!("- Bundle: `{}`\n", self.bundle_name));
        out.push_str(&format!("- Impossible: {}\n", self.impossible_count()));
        out.push_str(&format!("- Rare: {}\n", self.rare_count()));
        out.push_str(&format!("- Emergent: {}\n", self.emergent_count()));
        out.push_str(&format!("- External: {}\n", self.external_count()));
        out.push_str(&format!("- Verified: {}\n\n", self.verified_count()));
        out.push_str("| Obligation | Declared | Evidence | Verified | Statistical |\n");
        out.push_str("|---|---|---|---|---|\n");
        for obligation in &self.obligations {
            let statistical = obligation
                .statistical_verification
                .as_ref()
                .map(|verification| {
                    format!(
                        "`{:?}` at p={} (bound {})",
                        verification.classification,
                        verification.estimated_probability,
                        verification.bound
                    )
                })
                .unwrap_or_else(|| "-".into());
            out.push_str(&format!(
                "| `{}` | `{:?}` | `{:?}` | `{}` | {} |\n",
                obligation.obligation_name,
                obligation.declared_tier,
                obligation.evidence_tier,
                obligation.is_verified(),
                statistical
            ));
        }
        out
    }
}

/// Summarize a bundle across type-level, statistical, emergent, and external evidence.
#[cfg(feature = "std")]
pub fn three_tier_report(
    bundle: &ObligationBundle,
    statistical: &[StatisticalVerification],
    external: Option<&VerificationReport>,
) -> ThreeTierVerificationReport {
    let obligations = bundle
        .obligations()
        .iter()
        .map(|obligation| {
            let statistical_verification = statistical
                .iter()
                .find(|verification| verification.obligation_name == obligation.name)
                .cloned();
            let external_success = external.and_then(|report| {
                report
                    .obligations
                    .iter()
                    .find(|candidate| candidate.obligation_name == obligation.name)
                    .map(|candidate| candidate.succeeded())
            });
            let evidence_tier = if external_success == Some(true) {
                VerificationTier::External
            } else if let Some(verification) = &statistical_verification {
                verification.tier()
            } else {
                obligation.tier
            };
            ThreeTierObligationReport {
                obligation_name: obligation.name.clone(),
                summary: format_origin(obligation),
                declared_tier: obligation.tier,
                evidence_tier,
                external_success,
                statistical_verification,
            }
        })
        .collect();

    ThreeTierVerificationReport {
        bundle_name: bundle.name.clone(),
        obligations,
    }
}

fn amari_name(obligation: &Obligation, suffix: &str) -> String {
    format!("{}::{suffix}", obligation.name)
}

fn format_origin(obligation: &Obligation) -> String {
    format!(
        "{}::{} [{}]",
        obligation.origin.crate_name, obligation.origin.item_path, obligation.property
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CommandKind, ExecutionResult, ExecutionStatus, InvocationPlan, ObligationReport, Origin,
        Sort, Term, VerificationReport, prob_ensures, prob_requires,
    };
    use karpal_proof::Property;

    struct TestProperty;

    impl Property for TestProperty {
        const NAME: &'static str = "test property";
    }

    #[test]
    fn impossible_events_map_to_impossible_tier() {
        let obligation = Obligation::associativity(
            "sum_assoc",
            Origin::new("karpal-core", "Semigroup for Sum<i32>"),
            Sort::Int,
            "combine",
        );
        let verification = verify_rare_event(
            &obligation,
            &StatisticalBound::new(0.01).with_samples(128),
            || false,
        );

        assert_ne!(verification.status, AmariVerificationResult::Violated);
        assert_eq!(verification.estimated_probability, 0.0);
        assert_eq!(verification.tier(), VerificationTier::Impossible);
        assert!(verification.rare_event::<()>().is_none());
    }

    #[test]
    fn probable_events_map_to_emergent_tier() {
        let obligation = Obligation::for_property::<TestProperty>(
            "always_fails",
            Origin::new("karpal-test", "always_fails"),
            VerificationTier::Rare,
            Term::bool(false),
        );
        let verification = verify_rare_event(
            &obligation,
            &StatisticalBound::new(0.05).with_samples(128),
            || true,
        );

        assert_eq!(verification.status, AmariVerificationResult::Violated);
        assert_eq!(verification.classification, EventVerification::Probable);
        assert_eq!(verification.tier(), VerificationTier::Emergent);
    }

    #[test]
    fn bridges_to_amari_smt_obligations() {
        let obligation = Obligation::left_identity(
            "sum_left_identity",
            Origin::new("karpal-core", "Monoid for Sum<i32>"),
            Sort::Int,
            "combine",
            "e",
        );

        let pre = precondition_obligation_for(&obligation, "inputs are lawful", 0.99);
        let post = postcondition_obligation_for(&obligation, "result preserves law", 0.99);
        let hoeffding = concentration_obligation_for(&obligation, 1024, 0.05, 0.01);
        let expected = expected_value_obligation_for(&obligation, 0.0, 0.1, 1024);

        assert!(pre.to_smtlib2().contains("sum_left_identity::precondition"));
        assert!(
            post.to_smtlib2()
                .contains("sum_left_identity::postcondition")
        );
        assert!(
            hoeffding
                .to_smtlib2()
                .contains("sum_left_identity::hoeffding")
        );
        assert!(
            expected
                .to_smtlib2()
                .contains("sum_left_identity::expected-value")
        );
        assert!(matches!(
            expected.kind,
            AmariObligationKind::ExpectedValue { .. }
        ));
    }

    #[prob_requires(x >= 0.0, 1.0)]
    fn non_negative_identity(x: f64) -> f64 {
        x
    }

    #[prob_ensures(result >= 0.0, 1.0)]
    fn absolute_value(_value: f64) -> f64 {
        _value.abs()
    }

    #[test]
    fn amari_macros_generate_verification_helpers() {
        let pre = verify_non_negative_identity_precondition(|| 1.0, 64);
        let post = verify_absolute_value_postcondition(|| -3.0, 64);

        assert_eq!(non_negative_identity(1.0), 1.0);
        assert_eq!(absolute_value(-3.0), 3.0);
        assert!(matches!(
            pre,
            AmariVerificationResult::Verified | AmariVerificationResult::Inconclusive
        ));
        assert!(matches!(
            post,
            AmariVerificationResult::Verified | AmariVerificationResult::Inconclusive
        ));
    }

    #[test]
    fn three_tier_report_combines_declared_statistical_and_external_evidence() {
        let origin = Origin::new("karpal-test", "three_tier");
        let bundle = ObligationBundle::new("tiered_bundle", origin.clone())
            .with(Obligation::for_property::<TestProperty>(
                "type_safe",
                origin.clone(),
                VerificationTier::Impossible,
                Term::bool(true),
            ))
            .with(Obligation::for_property::<TestProperty>(
                "rare_failure",
                origin.clone(),
                VerificationTier::Rare,
                Term::bool(true),
            ))
            .with(Obligation::for_property::<TestProperty>(
                "externally_proved",
                origin,
                VerificationTier::External,
                Term::bool(true),
            ));

        let statistical = StatisticalVerification {
            obligation_name: "rare_failure".into(),
            property: TestProperty::NAME.into(),
            origin_summary: "karpal-test::three_tier [test property]".into(),
            samples: 4096,
            bound: 0.01,
            rare_threshold: 0.05,
            estimated_probability: 0.01,
            lower_probability: 0.0,
            upper_probability: 0.02,
            status: AmariVerificationResult::Verified,
            classification: EventVerification::Rare,
        };

        let external = VerificationReport {
            bundle_name: "tiered_bundle".into(),
            root: "target/three-tier".into(),
            obligations: vec![ObligationReport {
                obligation_name: "externally_proved".into(),
                summary: "karpal-test::three_tier [test property]".into(),
                artifact_path: None,
                lean_theorem_ref: None,
                lean_diagnostics: Vec::new(),
                result: Some(ExecutionResult {
                    plan: InvocationPlan {
                        kind: CommandKind::Smt,
                        executable: "z3".into(),
                        args: vec!["-smt2".into(), "externally_proved.smt2".into()],
                        working_directory: None,
                        input_files: vec!["externally_proved.smt2".into()],
                    },
                    status: ExecutionStatus::Unsat,
                    stdout: "unsat".into(),
                    stderr: String::new(),
                    exit_code: Some(0),
                    backend_version: None,
                    smt_output: None,
                    lean_output: None,
                }),
                certificate: None,
                lean_certificate: None,
            }],
            lean_module: None,
        };

        let report = three_tier_report(&bundle, &[statistical], Some(&external));

        assert_eq!(report.impossible_count(), 1);
        assert_eq!(report.rare_count(), 1);
        assert_eq!(report.external_count(), 1);
        assert_eq!(report.emergent_count(), 0);
        assert_eq!(report.verified_count(), 2);
        assert!(report.to_json().contains("\"rare_count\":1"));
        assert!(
            report
                .to_markdown()
                .contains("Three-Tier Verification Report")
        );
    }
}
