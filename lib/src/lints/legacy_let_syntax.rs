use crate::{Metadata, Report, Rule, Suggestion, make};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    ast::{Attr, Entry, HasEntry, LegacyLet},
};
use rowan::ast::AstNode as _;

/// ## What it does
/// Checks for legacy-let syntax that was never formalized.
///
/// ## Why is this bad?
/// This syntax construct is undocumented, refrain from using it.
///
/// ## Example
///
/// Legacy let syntax makes use of an attribute set annotated with
/// `let` and expects a `body` attribute.
/// ```nix
/// let {
///   body = x + y;
///   x = 2;
///   y = 3;
/// }
/// ```
///
/// This is trivially representible via `rec`, which is documented
/// and more widely known:
///
/// ```nix
/// rec {
///   body = x + y;
///   x = 2;
///   y = 3;
/// }.body
/// ```
#[lint(
    name = "legacy_let_syntax",
    note = "Using undocumented `let` syntax",
    code = 5,
    match_with = SyntaxKind::NODE_LEGACY_LET
)]
struct ManualInherit;

impl Rule for ManualInherit {
    fn validate(&self, node: &SyntaxElement) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let legacy_let = LegacyLet::cast(node.clone())?;

        if !legacy_let
            .entries()
            .filter_map(|entry| {
                let Entry::AttrpathValue(attrpath_value) = entry else {
                    return None;
                };

                let first_component = attrpath_value.attrpath()?.attrs().next()?;

                let Attr::Ident(ident) = first_component else {
                    return None;
                };

                Some(ident)
            })
            .any(|ident| ident.to_string() == "body")
        {
            return None;
        }

        let inherits = legacy_let.inherits();
        let entries = legacy_let.entries();
        let attrset = make::attrset(inherits, entries, true);
        let parenthesized = make::parenthesize(attrset.syntax());
        let selected = make::select(parenthesized.syntax(), make::ident("body").syntax());

        let at = node.text_range();
        let message = "Prefer `rec` over undocumented `let` syntax";
        let replacement = selected.syntax().clone();

        Some(
            self.report()
                .suggest(at, message, Suggestion::with_replacement(at, replacement)),
        )
    }
}
