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
        let NodeOrToken::Node(node) = node else {
            return None;
        };

        let lambda_expr = Lambda::cast(node.clone())?;
        let pattern = Pattern::cast(lambda_expr.arg()?)?;

        // no patterns within `{ }`
        if pattern.entries().count() != 0 {
            return None;
        }

        // pattern is not bound
        if pattern.at().is_some() {
            return None;
        }

        if is_module(&lambda_expr.body()?) {
            return None;
        }

        Some(self.report().suggest(
            pattern.node().text_range(),
            "This pattern is empty, use `_` instead",
            Suggestion::new(pattern.node().text_range(), make::ident("_").node().clone()),
        ))
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
