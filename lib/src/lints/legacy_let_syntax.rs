use crate::{Metadata, Report, Rule, Suggestion, make, session::SessionInfo};
use rowan::ast::AstNode;

use if_chain::if_chain;
use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind,
    ast::{HasEntry, LegacyLet},
};

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
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        if_chain! {
            if let NodeOrToken::Node(node) = node;
            if let Some(legacy_let) = LegacyLet::cast(node.clone());

            if legacy_let
                .entries()
                .filter_map(|entry| match entry {
                    rnix::ast::Entry::AttrpathValue(kv) => kv.attrpath(),
                    _ => None,
                })
                .any(|key_name| key_is_ident(&key_name, "body"));

            then {
                let inherits = legacy_let.inherits();
                let entries = legacy_let.entries();
                let attrset = make::attrset(inherits, entries, true);
                let parenthesized = make::parenthesize(attrset.syntax());
                let selected = make::select(parenthesized.syntax(), make::ident("body").syntax());

                let at = node.text_range();
                let message = "Prefer `rec` over undocumented `let` syntax";
                let replacement = selected.syntax().clone();

                Some(self.report().suggest(at, message, Suggestion::new(at, replacement)))
            } else {
                None
            }
        }
    }
}

fn key_is_ident(key_path: &rnix::ast::Attrpath, ident: &str) -> bool {
    key_path
        .attrs()
        .next()
        .and_then(|attr| match attr {
            rnix::ast::Attr::Ident(ident) => Some(ident),
            _ => None,
        })
        .is_some_and(|key| key.to_string() == ident)
}
