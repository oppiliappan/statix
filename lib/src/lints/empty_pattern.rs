use crate::{Metadata, Report, Rule, Suggestion, make, session::SessionInfo};

use macros::lint;
use rnix::{
    NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode,
    types::{AttrSet, EntryHolder, Lambda, Pattern, TypedNode},
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
        if let NodeOrToken::Node(node) = node
            && let Some(lambda_expr) = Lambda::cast(node.clone())
            && let Some(arg) = lambda_expr.arg()
            && let Some(body) = lambda_expr.body()
            && let Some(pattern) = Pattern::cast(arg)
            // no patterns within `{ }`
            && pattern.entries().count() == 0
            // pattern is not bound
            && pattern.at().is_none()
            // not a nixos module
            && !is_module(&body)
        {
            let at = pattern.node().text_range();
            let message = "This pattern is empty, use `_` instead";
            let replacement = make::ident("_").node().clone();
            Some(
                self.report()
                    .suggest(at, message, Suggestion::new(at, replacement)),
            )
        } else {
            None
        }
    }
}

fn is_module(body: &SyntaxNode) -> bool {
    if let Some(attr_set) = AttrSet::cast(body.clone())
        && attr_set
            .entries()
            .filter_map(|e| e.key())
            .any(|k| k.node().to_string() == "imports")
    {
        true
    } else {
        false
    }
}
