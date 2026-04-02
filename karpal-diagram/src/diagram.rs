use alloc::{boxed::Box, string::String, vec::Vec};

/// Runtime string-diagram representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagram {
    pub kind: DiagramKind,
    pub input_arity: usize,
    pub output_arity: usize,
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
        match &self.kind {
            DiagramKind::Identity => Self::identity(self.input_arity),
            DiagramKind::Box { label } => {
                Self::box_(label.clone(), self.input_arity, self.output_arity)
            }
            DiagramKind::Swap { left, right } => Self::swap(*left, *right),
            DiagramKind::Sequence(left, right) => {
                let left = left.normalize();
                let right = right.normalize();
                match (&left.kind, &right.kind) {
                    (DiagramKind::Identity, _) => right,
                    (_, DiagramKind::Identity) => left,
                    (
                        DiagramKind::Swap {
                            left: ll,
                            right: lr,
                        },
                        DiagramKind::Swap {
                            left: rl,
                            right: rr,
                        },
                    ) if ll == rr && lr == rl => Self::identity(left.input_arity),
                    _ => left.then(right),
                }
            }
            DiagramKind::Parallel(left, right) => {
                let left = left.normalize();
                let right = right.normalize();
                match (&left.kind, &right.kind) {
                    (DiagramKind::Identity, DiagramKind::Identity) => {
                        Self::identity(left.input_arity + right.input_arity)
                    }
                    _ => left.parallel(right),
                }
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

    fn collect_sequence_chain<'a>(&'a self, out: &mut Vec<&'a Diagram>) {
        match &self.kind {
            DiagramKind::Sequence(left, right) => {
                left.collect_sequence_chain(out);
                right.collect_sequence_chain(out);
            }
            _ => out.push(self),
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
}
