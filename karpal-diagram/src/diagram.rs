use alloc::{boxed::Box, string::String, vec::Vec};

/// Runtime string-diagram representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagram {
    pub kind: DiagramKind,
    pub input_arity: usize,
    pub output_arity: usize,
}

/// Individual rewrite rules applied during normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationRule {
    FlattenSequence,
    FlattenParallel,
    ElideIdentitySequenceStage,
    CollapseIdentityParallel,
    CancelAdjacentSwaps,
}

/// Trace of rewrite rules applied while normalizing a diagram.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizationTrace {
    pub normalized: Diagram,
    pub rules: Vec<NormalizationRule>,
}

impl NormalizationTrace {
    pub fn applied(&self, rule: NormalizationRule) -> bool {
        self.rules.contains(&rule)
    }
}

/// Primitive diagram nodes and combinators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagramKind {
    Identity,
    Box { label: String },
    Sequence(Box<Diagram>, Box<Diagram>),
    Parallel(Box<Diagram>, Box<Diagram>),
    Swap { left: usize, right: usize },
}

impl Diagram {
    pub fn identity(arity: usize) -> Self {
        Self {
            kind: DiagramKind::Identity,
            input_arity: arity,
            output_arity: arity,
        }
    }

    pub fn box_(label: impl Into<String>, input_arity: usize, output_arity: usize) -> Self {
        Self {
            kind: DiagramKind::Box {
                label: label.into(),
            },
            input_arity,
            output_arity,
        }
    }

    pub fn swap(left: usize, right: usize) -> Self {
        Self {
            kind: DiagramKind::Swap { left, right },
            input_arity: left + right,
            output_arity: left + right,
        }
    }

    pub fn then(self, next: Self) -> Self {
        assert_eq!(
            self.output_arity, next.input_arity,
            "sequential diagrams must agree on middle arity"
        );
        Self {
            input_arity: self.input_arity,
            output_arity: next.output_arity,
            kind: DiagramKind::Sequence(Box::new(self), Box::new(next)),
        }
    }

    pub fn parallel(self, other: Self) -> Self {
        Self {
            input_arity: self.input_arity + other.input_arity,
            output_arity: self.output_arity + other.output_arity,
            kind: DiagramKind::Parallel(Box::new(self), Box::new(other)),
        }
    }

    pub fn normalize(&self) -> Self {
        self.normalize_with_trace().normalized
    }

    pub fn normalize_with_trace(&self) -> NormalizationTrace {
        let mut rules = Vec::new();
        let normalized = self.normalize_into(&mut rules);
        NormalizationTrace { normalized, rules }
    }

    fn normalize_into(&self, rules: &mut Vec<NormalizationRule>) -> Self {
        match &self.kind {
            DiagramKind::Identity => Self::identity(self.input_arity),
            DiagramKind::Box { label } => {
                Self::box_(label.clone(), self.input_arity, self.output_arity)
            }
            DiagramKind::Swap { left, right } => Self::swap(*left, *right),
            DiagramKind::Sequence(_, _) => {
                rules.push(NormalizationRule::FlattenSequence);
                Self::normalize_sequence(
                    self.sequence_chain()
                        .into_iter()
                        .map(|diagram| diagram.normalize_into(rules))
                        .collect(),
                    self.input_arity,
                    self.output_arity,
                    rules,
                )
            }
            DiagramKind::Parallel(_, _) => {
                rules.push(NormalizationRule::FlattenParallel);
                Self::normalize_parallel(
                    self.parallel_chain()
                        .into_iter()
                        .map(|diagram| diagram.normalize_into(rules))
                        .collect(),
                    self.input_arity,
                    self.output_arity,
                    rules,
                )
            }
        }
    }

    pub fn equivalent_to(&self, other: &Self) -> bool {
        self.normalize() == other.normalize()
    }

    pub fn render_text(&self) -> String {
        crate::render::TextRenderer::render(self)
    }

    pub fn render_svg(&self) -> String {
        crate::render::SvgRenderer::render(self)
    }

    pub(crate) fn sequence_chain(&self) -> Vec<&Diagram> {
        let mut out = Vec::new();
        self.collect_sequence_chain(&mut out);
        out
    }

    pub(crate) fn parallel_chain(&self) -> Vec<&Diagram> {
        let mut out = Vec::new();
        self.collect_parallel_chain(&mut out);
        out
    }

    fn collect_sequence_chain<'a>(&'a self, out: &mut Vec<&'a Diagram>) {
        match &self.kind {
            DiagramKind::Sequence(left, right) => {
                left.collect_sequence_chain(out);
                right.collect_sequence_chain(out);
            }
            _ => out.push(self),
        }
    }

    fn collect_parallel_chain<'a>(&'a self, out: &mut Vec<&'a Diagram>) {
        match &self.kind {
            DiagramKind::Parallel(left, right) => {
                left.collect_parallel_chain(out);
                right.collect_parallel_chain(out);
            }
            _ => out.push(self),
        }
    }

    fn normalize_sequence(
        stages: Vec<Self>,
        input_arity: usize,
        output_arity: usize,
        rules: &mut Vec<NormalizationRule>,
    ) -> Self {
        let mut reduced: Vec<Self> = Vec::new();

        for stage in stages {
            if stage.kind == DiagramKind::Identity {
                rules.push(NormalizationRule::ElideIdentitySequenceStage);
                continue;
            }

            if let Some(previous) = reduced.last()
                && previous.cancels_with(&stage)
            {
                rules.push(NormalizationRule::CancelAdjacentSwaps);
                reduced.pop();
                continue;
            }

            reduced.push(stage);
        }

        Self::rebuild_sequence(reduced, input_arity, output_arity)
    }

    fn normalize_parallel(
        branches: Vec<Self>,
        input_arity: usize,
        output_arity: usize,
        rules: &mut Vec<NormalizationRule>,
    ) -> Self {
        if branches
            .iter()
            .all(|branch| branch.kind == DiagramKind::Identity)
        {
            rules.push(NormalizationRule::CollapseIdentityParallel);
            return Self::identity(input_arity);
        }

        Self::rebuild_parallel(branches, input_arity, output_arity)
    }

    fn rebuild_sequence(stages: Vec<Self>, input_arity: usize, output_arity: usize) -> Self {
        let mut iter = stages.into_iter();
        let Some(first) = iter.next() else {
            debug_assert_eq!(input_arity, output_arity);
            return Self::identity(input_arity);
        };

        iter.fold(first, Self::then)
    }

    fn rebuild_parallel(branches: Vec<Self>, input_arity: usize, output_arity: usize) -> Self {
        let mut iter = branches.into_iter();
        let Some(first) = iter.next() else {
            debug_assert_eq!(input_arity, output_arity);
            return Self::identity(input_arity);
        };

        iter.fold(first, Self::parallel)
    }

    fn cancels_with(&self, other: &Self) -> bool {
        match (&self.kind, &other.kind) {
            (
                DiagramKind::Swap {
                    left: left_left,
                    right: left_right,
                },
                DiagramKind::Swap {
                    left: right_left,
                    right: right_right,
                },
            ) => left_left == right_right && left_right == right_left,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_requires_matching_arities() {
        let left = Diagram::box_("double", 1, 1);
        let right = Diagram::box_("pair", 2, 1);
        let result = std::panic::catch_unwind(|| left.then(right));
        assert!(result.is_err());
    }

    #[test]
    fn normalization_elides_identity() {
        let diagram = Diagram::identity(1).then(Diagram::box_("inc", 1, 1));
        assert_eq!(diagram.normalize(), Diagram::box_("inc", 1, 1));
    }

    #[test]
    fn double_swap_normalizes_to_identity() {
        let diagram = Diagram::swap(1, 2).then(Diagram::swap(2, 1));
        assert_eq!(diagram.normalize(), Diagram::identity(3));
    }

    #[test]
    fn sequence_associativity_normalizes_to_canonical_form() {
        let left =
            Diagram::box_("f", 1, 1).then(Diagram::box_("g", 1, 1).then(Diagram::box_("h", 1, 1)));
        let right = Diagram::box_("f", 1, 1)
            .then(Diagram::box_("g", 1, 1))
            .then(Diagram::box_("h", 1, 1));

        assert_eq!(left.normalize(), right.normalize());
        assert!(left.equivalent_to(&right));
    }

    #[test]
    fn parallel_associativity_normalizes_to_canonical_form() {
        let left = Diagram::box_("f", 1, 1)
            .parallel(Diagram::box_("g", 1, 1).parallel(Diagram::box_("h", 1, 1)));
        let right = Diagram::box_("f", 1, 1)
            .parallel(Diagram::box_("g", 1, 1))
            .parallel(Diagram::box_("h", 1, 1));

        assert_eq!(left.normalize(), right.normalize());
        assert!(left.equivalent_to(&right));
    }

    #[test]
    fn sequence_normalization_cancels_swaps_inside_longer_chain() {
        let diagram = Diagram::box_("f", 3, 3)
            .then(Diagram::swap(1, 2))
            .then(Diagram::swap(2, 1))
            .then(Diagram::box_("g", 3, 3));

        assert_eq!(
            diagram.normalize(),
            Diagram::box_("f", 3, 3).then(Diagram::box_("g", 3, 3))
        );
    }

    #[test]
    fn normalization_trace_records_sequence_rules() {
        let trace = Diagram::identity(2)
            .then(Diagram::swap(1, 1))
            .then(Diagram::swap(1, 1))
            .normalize_with_trace();

        assert_eq!(trace.normalized, Diagram::identity(2));
        assert!(trace.applied(NormalizationRule::FlattenSequence));
        assert!(trace.applied(NormalizationRule::ElideIdentitySequenceStage));
        assert!(trace.applied(NormalizationRule::CancelAdjacentSwaps));
    }

    #[test]
    fn normalization_trace_records_parallel_rules() {
        let trace = Diagram::identity(1)
            .parallel(Diagram::identity(2))
            .normalize_with_trace();

        assert_eq!(trace.normalized, Diagram::identity(3));
        assert!(trace.applied(NormalizationRule::FlattenParallel));
        assert!(trace.applied(NormalizationRule::CollapseIdentityParallel));
    }
}
