#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeSet, format, string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{collections::BTreeSet, format, string::String, vec::Vec};

use crate::{
    Obligation, ObligationBundle,
    obligation::{Sort, Term},
};

/// Marker for the Lean 4 backend.
pub struct Lean4;

/// A Lean module import.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LeanImport {
    pub module: String,
}

impl LeanImport {
    pub fn new(module: impl Into<String>) -> Self {
        Self {
            module: module.into(),
        }
    }
}

/// A Lean-safe alias for a raw symbol name from the obligation IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanAlias {
    pub alias: String,
    pub target: String,
}

impl LeanAlias {
    pub fn new(alias: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            alias: alias.into(),
            target: target.into(),
        }
    }
}

/// Prelude metadata emitted ahead of theorem declarations.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LeanPrelude {
    pub imports: Vec<LeanImport>,
    pub aliases: Vec<LeanAlias>,
}

impl LeanPrelude {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_import(mut self, module: impl Into<String>) -> Self {
        let import = LeanImport::new(module);
        if !self.imports.contains(&import) {
            self.imports.push(import);
        }
        self
    }

    pub fn with_alias(mut self, alias: impl Into<String>, target: impl Into<String>) -> Self {
        let alias = LeanAlias::new(alias, target);
        if !self
            .aliases
            .iter()
            .any(|existing| existing.target == alias.target)
        {
            self.aliases.push(alias);
        }
        self
    }

    pub fn for_obligations(obligations: &[Obligation]) -> Self {
        let mut prelude = Self::new();
        let mut symbols = BTreeSet::new();

        for obligation in obligations {
            collect_term_symbols(&obligation.conclusion, &mut symbols);
            for assumption in &obligation.assumptions {
                collect_term_symbols(assumption, &mut symbols);
            }
        }

        for symbol in symbols {
            if !is_lean_identifier(&symbol) {
                prelude = prelude.with_alias(alias_name(&symbol), symbol);
            }
        }

        prelude
    }

    pub fn symbol_name(&self, target: &str) -> String {
        self.aliases
            .iter()
            .find(|alias| alias.target == target)
            .map(|alias| alias.alias.clone())
            .unwrap_or_else(|| render_identifier(target))
    }
}

/// Structured Lean theorem metadata derived from an obligation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanTheorem {
    pub obligation_name: String,
    pub theorem_name: String,
    pub property: String,
    pub origin_summary: String,
    pub declaration_start_line: usize,
    pub declaration_end_line: usize,
}

impl LeanTheorem {
    pub fn witness_ref(&self, module_name: &str) -> String {
        format!("{module_name}.{}", self.theorem_name)
    }

    pub fn contains_line(&self, line: usize) -> bool {
        (self.declaration_start_line..=self.declaration_end_line).contains(&line)
    }

    pub fn with_line_span(mut self, start_line: usize, end_line: usize) -> Self {
        self.declaration_start_line = start_line;
        self.declaration_end_line = end_line.max(start_line);
        self
    }
}

/// Structured Lean export result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanExport {
    pub module_name: String,
    pub source: String,
    pub prelude: LeanPrelude,
    pub theorems: Vec<LeanTheorem>,
}

/// Generated Lean package metadata for writing a runnable project scaffold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeanProject {
    pub package_name: String,
    pub toolchain: String,
    pub requires_mathlib: bool,
}

impl LeanProject {
    pub fn new(package_name: impl Into<String>) -> Self {
        Self {
            package_name: package_name.into(),
            toolchain: "leanprover/lean4:stable".into(),
            requires_mathlib: false,
        }
    }

    pub fn for_export(export: &LeanExport) -> Self {
        let mut project = Self::new(sanitize(&export.module_name).to_lowercase());
        project.requires_mathlib = export
            .prelude
            .imports
            .iter()
            .any(|import| import.module == "Mathlib");
        project
    }

    pub fn with_toolchain(mut self, toolchain: impl Into<String>) -> Self {
        self.toolchain = toolchain.into();
        self
    }

    pub fn with_mathlib(mut self, requires_mathlib: bool) -> Self {
        self.requires_mathlib = requires_mathlib;
        self
    }

    pub fn render_lakefile(&self) -> String {
        let mut lines = vec![
            format!("package {}", render_identifier(&self.package_name)),
            String::new(),
        ];

        if self.requires_mathlib {
            lines.push("require mathlib from git".into());
            lines.push("  \"https://github.com/leanprover-community/mathlib4.git\"".into());
            lines.push(String::new());
        }

        lines.push(format!(
            "@[default_target]\nlean_lib {}",
            render_identifier(&self.package_name)
        ));
        lines.join("\n")
    }

    pub fn render_toolchain(&self) -> String {
        self.toolchain.clone()
    }
}

impl LeanExport {
    pub fn theorem_for_obligation(&self, obligation_name: &str) -> Option<&LeanTheorem> {
        self.theorems
            .iter()
            .find(|theorem| theorem.obligation_name == obligation_name)
    }

    pub fn theorem_names(&self) -> Vec<String> {
        self.theorems
            .iter()
            .map(|theorem| theorem.theorem_name.clone())
            .collect()
    }

    pub fn project(&self) -> LeanProject {
        LeanProject::for_export(self)
    }
}

impl Lean4 {
    /// Export a list of obligations as a Lean 4 module skeleton.
    pub fn export_module(module_name: &str, obligations: &[Obligation]) -> String {
        export_module(module_name, obligations)
    }

    /// Export a list of obligations as a Lean 4 module skeleton with explicit prelude metadata.
    pub fn export_module_with_prelude(
        module_name: &str,
        obligations: &[Obligation],
        prelude: LeanPrelude,
    ) -> String {
        export_with_prelude(module_name, obligations, prelude).source
    }

    /// Export a list of obligations as structured Lean metadata plus source.
    pub fn export(module_name: &str, obligations: &[Obligation]) -> LeanExport {
        export(module_name, obligations)
    }

    /// Export a list of obligations as structured Lean metadata plus source with explicit prelude metadata.
    pub fn export_with_prelude(
        module_name: &str,
        obligations: &[Obligation],
        prelude: LeanPrelude,
    ) -> LeanExport {
        export_with_prelude(module_name, obligations, prelude)
    }

    /// Export a bundle of obligations as a Lean 4 module skeleton.
    pub fn export_bundle(module_name: &str, bundle: &ObligationBundle) -> String {
        export_module(module_name, bundle.obligations())
    }

    /// Export a bundle of obligations as structured Lean metadata plus source.
    pub fn export_bundle_structured(module_name: &str, bundle: &ObligationBundle) -> LeanExport {
        export(module_name, bundle.obligations())
    }

    /// Derive Lean package metadata from a structured export.
    pub fn project(export: &LeanExport) -> LeanProject {
        LeanProject::for_export(export)
    }
}

pub fn export(module_name: &str, obligations: &[Obligation]) -> LeanExport {
    export_with_prelude(
        module_name,
        obligations,
        LeanPrelude::for_obligations(obligations),
    )
}

pub fn export_with_prelude(
    module_name: &str,
    obligations: &[Obligation],
    prelude: LeanPrelude,
) -> LeanExport {
    let mut theorems = Vec::new();
    let mut lines = Vec::new();

    for import in &prelude.imports {
        lines.push(format!("import {}", import.module));
    }
    if !prelude.imports.is_empty() {
        lines.push(String::new());
    }

    lines.push(format!("namespace {}", module_name));
    lines.push(String::new());

    for alias in &prelude.aliases {
        lines.push(format!(
            "abbrev {} := {}",
            alias.alias,
            render_identifier(&alias.target)
        ));
    }
    if !prelude.aliases.is_empty() {
        lines.push(String::new());
    }

    for obligation in obligations {
        let theorem = LeanTheorem::from(obligation);
        let theorem_start_line = lines.len() + 3;
        let theorem_text = render_theorem(obligation, &theorem, &prelude);
        let theorem_line_count = theorem_text.lines().count();
        let theorem_end_line = theorem_start_line + theorem_line_count.saturating_sub(1);

        lines.push(format!("-- property: {}", obligation.property));
        lines.push(format!("-- origin: {}", obligation.summary()));
        lines.push(theorem_text);
        lines.push(String::new());

        theorems.push(theorem.with_line_span(theorem_start_line, theorem_end_line));
    }

    lines.push(format!("end {}", module_name));

    LeanExport {
        module_name: module_name.into(),
        source: lines.join("\n"),
        prelude,
        theorems,
    }
}

pub fn export_module(module_name: &str, obligations: &[Obligation]) -> String {
    export(module_name, obligations).source
}

pub fn export_module_with_prelude(
    module_name: &str,
    obligations: &[Obligation],
    prelude: LeanPrelude,
) -> String {
    export_with_prelude(module_name, obligations, prelude).source
}

fn render_theorem(obligation: &Obligation, theorem: &LeanTheorem, prelude: &LeanPrelude) -> String {
    let binders = obligation
        .declarations
        .iter()
        .map(|decl| {
            format!(
                "({} : {})",
                render_identifier(&decl.name),
                render_sort(&decl.sort)
            )
        })
        .collect::<Vec<_>>()
        .join(" ");

    let assumptions = obligation
        .assumptions
        .iter()
        .enumerate()
        .map(|(idx, term)| format!("(h{} : {})", idx + 1, render_term(term, prelude)))
        .collect::<Vec<_>>()
        .join(" ");

    let mut header = format!("theorem {}", theorem.theorem_name);
    if !binders.is_empty() {
        header.push(' ');
        header.push_str(&binders);
    }
    if !assumptions.is_empty() {
        header.push(' ');
        header.push_str(&assumptions);
    }
    header.push_str(&format!(
        " : {} := by",
        render_term(&obligation.conclusion, prelude)
    ));
    header.push_str("\n  sorry");
    header
}

fn render_sort(sort: &Sort) -> String {
    match sort {
        Sort::Bool => "Bool".to_string(),
        Sort::Int => "Int".to_string(),
        Sort::Real => "Rat".to_string(),
        Sort::Named(name) => render_identifier(name),
    }
}

fn render_term(term: &Term, prelude: &LeanPrelude) -> String {
    match term {
        Term::Var(name) => render_identifier(name),
        Term::Bool(true) => "True".to_string(),
        Term::Bool(false) => "False".to_string(),
        Term::Int(value) => value.to_string(),
        Term::App { function, args } => {
            let function = prelude.symbol_name(function);
            if args.is_empty() {
                function
            } else {
                format!(
                    "({} {})",
                    function,
                    args.iter()
                        .map(|term| render_term(term, prelude))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }
        }
        Term::Eq(left, right) => format!(
            "{} = {}",
            render_term(left, prelude),
            render_term(right, prelude)
        ),
        Term::And(terms) => match terms.as_slice() {
            [] => "True".to_string(),
            [term] => render_term(term, prelude),
            _ => terms
                .iter()
                .map(|term| render_term(term, prelude))
                .collect::<Vec<_>>()
                .join(" ∧ "),
        },
        Term::Or(terms) => match terms.as_slice() {
            [] => "False".to_string(),
            [term] => render_term(term, prelude),
            _ => terms
                .iter()
                .map(|term| render_term(term, prelude))
                .collect::<Vec<_>>()
                .join(" ∨ "),
        },
        Term::Not(inner) => format!("¬{}", parenthesize(inner, prelude)),
        Term::Implies(lhs, rhs) => format!(
            "{} → {}",
            parenthesize(lhs, prelude),
            render_term(rhs, prelude)
        ),
    }
}

fn parenthesize(term: &Term, prelude: &LeanPrelude) -> String {
    match term {
        Term::Var(_) | Term::Bool(_) | Term::Int(_) | Term::App { .. } => {
            render_term(term, prelude)
        }
        _ => format!("({})", render_term(term, prelude)),
    }
}

fn collect_term_symbols(term: &Term, symbols: &mut BTreeSet<String>) {
    match term {
        Term::Var(_) | Term::Bool(_) | Term::Int(_) => {}
        Term::App { function, args } => {
            symbols.insert(function.clone());
            for arg in args {
                collect_term_symbols(arg, symbols);
            }
        }
        Term::Eq(left, right) | Term::Implies(left, right) => {
            collect_term_symbols(left, symbols);
            collect_term_symbols(right, symbols);
        }
        Term::And(terms) | Term::Or(terms) => {
            for term in terms {
                collect_term_symbols(term, symbols);
            }
        }
        Term::Not(inner) => collect_term_symbols(inner, symbols),
    }
}

fn render_identifier(name: &str) -> String {
    if is_lean_identifier(name) {
        name.to_string()
    } else {
        format!("«{}»", name.replace('»', "_"))
    }
}

fn alias_name(name: &str) -> String {
    let mut alias = String::from("sym_");
    alias.push_str(&sanitize(name));
    alias
}

fn is_lean_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }

    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '\'')
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

impl From<&Obligation> for LeanTheorem {
    fn from(obligation: &Obligation) -> Self {
        Self {
            obligation_name: obligation.name.clone(),
            theorem_name: sanitize(&obligation.name),
            property: obligation.property.into(),
            origin_summary: obligation.summary(),
            declaration_start_line: 0,
            declaration_end_line: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obligation::{Origin, VerificationTier};
    use karpal_proof::IsCommutative;

    #[test]
    fn exports_theorem_skeleton() {
        let obligation = Obligation::for_property::<IsCommutative>(
            "sum/commutative",
            Origin::new("karpal-algebra", "AbelianGroup for i16"),
            VerificationTier::External,
            Term::eq(
                Term::app("combine", [Term::var("a"), Term::var("b")]),
                Term::app("combine", [Term::var("b"), Term::var("a")]),
            ),
        )
        .with_decl("a", Sort::Int)
        .with_decl("b", Sort::Int);

        let text = export_module("KarpalVerify", &[obligation]);
        assert!(text.contains("namespace KarpalVerify"));
        assert!(text.contains("theorem sum_commutative"));
        assert!(text.contains("sorry"));
    }

    #[test]
    fn structured_export_tracks_theorem_identity() {
        let obligation = Obligation::associativity(
            "sum-assoc",
            Origin::new("karpal-core", "Semigroup for i32"),
            Sort::Int,
            "combine",
        );

        let export = export("KarpalVerify", &[obligation]);
        assert_eq!(export.module_name, "KarpalVerify");
        assert_eq!(export.theorems.len(), 1);
        assert_eq!(export.theorems[0].theorem_name, "sum_assoc");
        assert_eq!(
            export.theorems[0].witness_ref("KarpalVerify"),
            "KarpalVerify.sum_assoc"
        );
        assert!(export.theorems[0].declaration_start_line > 0);
        assert!(
            export.theorems[0].declaration_end_line >= export.theorems[0].declaration_start_line
        );
        assert!(export.source.contains("theorem sum_assoc"));
    }

    #[test]
    fn derived_prelude_aliases_invalid_symbols() {
        let obligation = Obligation::for_property::<IsCommutative>(
            "sum/commutative",
            Origin::new("karpal-algebra", "AbelianGroup for i16"),
            VerificationTier::External,
            Term::eq(
                Term::app("combine-op", [Term::var("a"), Term::var("b")]),
                Term::app("combine-op", [Term::var("b"), Term::var("a")]),
            ),
        )
        .with_decl("a", Sort::Int)
        .with_decl("b", Sort::Int);

        let export = export("KarpalVerify", &[obligation]);
        assert_eq!(export.prelude.aliases.len(), 1);
        assert_eq!(export.prelude.aliases[0].alias, "sym_combine_op");
        assert_eq!(export.prelude.aliases[0].target, "combine-op");
        assert!(
            export
                .source
                .contains("abbrev sym_combine_op := «combine-op»")
        );
        assert!(
            export
                .source
                .contains("(sym_combine_op a b) = (sym_combine_op b a)")
        );
    }

    #[test]
    fn explicit_prelude_renders_imports_before_namespace() {
        let obligation = Obligation::associativity(
            "sum-assoc",
            Origin::new("karpal-core", "Semigroup for i32"),
            Sort::Int,
            "combine",
        );
        let prelude = LeanPrelude::new().with_import("Mathlib");

        let text = export_module_with_prelude("KarpalVerify", &[obligation], prelude);
        assert!(text.starts_with("import Mathlib\n\nnamespace KarpalVerify"));
    }

    #[test]
    fn project_derives_mathlib_requirement_from_prelude() {
        let obligation = Obligation::associativity(
            "sum-assoc",
            Origin::new("karpal-core", "Semigroup for i32"),
            Sort::Int,
            "combine",
        );
        let export = export_with_prelude(
            "KarpalVerify",
            &[obligation],
            LeanPrelude::new().with_import("Mathlib"),
        );
        let project = export.project();

        assert_eq!(project.package_name, "karpalverify");
        assert!(project.requires_mathlib);
        assert!(
            project
                .render_lakefile()
                .contains("require mathlib from git")
        );
        assert_eq!(project.render_toolchain(), "leanprover/lean4:stable");
    }
}
