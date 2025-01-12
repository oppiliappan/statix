use crate::{make, session::SessionInfo, Metadata, Report, Rule, Suggestion};
use rnix::ast::HasEntry;
use rowan::ast::AstNode;

use macros::lint;
use rnix::{
    ast::{AttrSet, Lambda, Pattern},
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
};

/// ## What it does
/// Checks for an empty variadic pattern: `{...}`, in a function
/// argument.
///
/// ## Why is this bad?
/// The intention with empty patterns is not instantly obvious. Prefer
/// an underscore identifier instead, to indicate that the argument
/// is being ignored.
///
/// ## Example
///
/// ```nix
/// client = { ... }: {
///   services.irmaseal-pkg.enable = true;
/// };
/// ```
///
/// Replace the empty variadic pattern with `_` to indicate that you
/// intend to ignore the argument:
///
/// ```nix
/// client = _: {
///   services.irmaseal-pkg.enable = true;
/// };
/// ```
#[lint(
    name = "empty_pattern",
    note = "Found empty pattern in function argument",
    code = 10,
    match_with = SyntaxKind::NODE_LAMBDA
)]
struct EmptyPattern;

impl Rule for EmptyPattern {
    fn validate(&self, node: &SyntaxElement, _sess: &SessionInfo) -> Option<Report> {
        let NodeOrToken::Node(node) = node else {
            return None;
        };
        let lambda_expr = Lambda::cast(node.clone())?;
        let arg = lambda_expr.param()?;
        let body = lambda_expr.body()?;

        let pattern = Pattern::cast(arg.syntax().clone())?;

        // no patterns within `{ }`
        if pattern.pat_entries().count() != 0 {
            return None;
        };
        // pattern is not bound
        if pattern.pat_bind().is_some() {
            return None;
        };

        // not a nixos module
        if is_module(body.syntax()) {
            return None;
        };

        let at = pattern.syntax().text_range();
        let message = "This pattern is empty, use `_` instead";
        let replacement = make::ident("_").syntax().clone();
        Some(
            self.report()
                .suggest(at, message, Suggestion::new(at, replacement)),
        )
    }
}

fn is_module(body: &SyntaxNode) -> bool {
    AttrSet::cast(body.clone())
        .into_iter()
        .flat_map(|attr_set| attr_set.entries())
        .filter_map(|entry| match entry {
            rnix::ast::Entry::AttrpathValue(attrpath_value) => attrpath_value.attrpath(),
            _ => None,
        })
        .any(|k| k.to_string() == "imports")
}
