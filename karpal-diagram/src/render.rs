use alloc::{format, string::String};

use crate::diagram::{Diagram, DiagramKind, NormalizationRule, NormalizationTrace};

pub struct TextRenderer;
pub struct SvgRenderer;

impl TextRenderer {
    pub fn render(diagram: &Diagram) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "diagram {} -> {}\n",
            diagram.input_arity, diagram.output_arity
        ));
        for (index, stage) in diagram.sequence_chain().iter().enumerate() {
            out.push_str(&format!("{index}: {}\n", Self::stage(stage)));
        }
        out
    }

    pub fn render_trace(trace: &NormalizationTrace) -> String {
        let mut out = String::new();
        out.push_str("normalization trace\n");
        out.push_str("rules:\n");
        if trace.rules.is_empty() {
            out.push_str("- none\n");
        } else {
            for rule in &trace.rules {
                out.push_str(&format!("- {}\n", Self::rule_name(*rule)));
            }
        }
        out.push_str("normalized:\n");
        out.push_str(&Self::render(&trace.normalized));
        out
    }

    fn stage(diagram: &Diagram) -> String {
        match &diagram.kind {
            DiagramKind::Identity => format!("id[{}]", diagram.input_arity),
            DiagramKind::Box { label } => format!(
                "box({label}) {} -> {}",
                diagram.input_arity, diagram.output_arity
            ),
            DiagramKind::Parallel(left, right) => {
                format!("parallel({}, {})", Self::stage(left), Self::stage(right))
            }
            DiagramKind::Swap { left, right } => format!("swap[{left}|{right}]"),
            DiagramKind::Sequence(_, _) => {
                unreachable!("sequence chains are flattened before stage rendering")
            }
        }
    }

    fn rule_name(rule: NormalizationRule) -> &'static str {
        match rule {
            NormalizationRule::FlattenSequence => "flatten-sequence",
            NormalizationRule::FlattenParallel => "flatten-parallel",
            NormalizationRule::ElideIdentitySequenceStage => "elide-identity-sequence-stage",
            NormalizationRule::CollapseIdentityParallel => "collapse-identity-parallel",
            NormalizationRule::CancelAdjacentSwaps => "cancel-adjacent-swaps",
        }
    }
}

impl SvgRenderer {
    pub fn render(diagram: &Diagram) -> String {
        let stages = diagram.sequence_chain();
        let width = 180 * stages.len().max(1);
        let height = 80 + 40 * diagram.input_arity.max(diagram.output_arity).max(1);
        let mut out =
            format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\">");
        out.push_str("<style>text{font-family:monospace;font-size:12px}rect{fill:#1f2937;stroke:#94a3b8}line{stroke:#64748b;stroke-width:2}</style>");

        for wire in 0..diagram.input_arity.max(1) {
            let y = 40 + wire * 30;
            out.push_str(&format!(
                "<line x1=\"10\" y1=\"{y}\" x2=\"{}\" y2=\"{y}\" />",
                width - 10
            ));
        }

        for (index, stage) in stages.iter().enumerate() {
            let x = 30 + index * 160;
            out.push_str(&format!(
                "<rect x=\"{x}\" y=\"20\" width=\"120\" height=\"40\" rx=\"6\" />"
            ));
            out.push_str(&format!(
                "<text x=\"{}\" y=\"45\">{}</text>",
                x + 10,
                TextRenderer::stage(stage)
            ));
        }

        out.push_str("</svg>");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagram::Diagram;

    #[test]
    fn text_renderer_includes_swap_and_boxes() {
        let diagram = Diagram::box_("double", 1, 1)
            .parallel(Diagram::box_("increment", 1, 1))
            .then(Diagram::swap(1, 1));
        let rendered = TextRenderer::render(&diagram);
        assert!(rendered.contains("parallel"));
        assert!(rendered.contains("swap[1|1]"));
    }

    #[test]
    fn svg_renderer_emits_svg_tag() {
        let diagram = Diagram::box_("double", 1, 1);
        let rendered = SvgRenderer::render(&diagram);
        assert!(rendered.starts_with("<svg"));
        assert!(rendered.contains("box(double)"));
    }

    #[test]
    fn text_renderer_trace_lists_applied_rules() {
        let trace = Diagram::identity(2)
            .then(Diagram::swap(1, 1))
            .then(Diagram::swap(1, 1))
            .normalize_with_trace();

        let rendered = TextRenderer::render_trace(&trace);
        assert!(rendered.contains("normalization trace"));
        assert!(rendered.contains("flatten-sequence"));
        assert!(rendered.contains("cancel-adjacent-swaps"));
        assert!(rendered.contains("diagram 2 -> 2"));
    }
}
